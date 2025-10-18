
//////
//
// Imports
//

// Standard library
use std::{error::Error, rc::Rc, borrow::Cow, path::{PathBuf, Path}, fmt::{Display, Formatter}};

// Anyhow library
use anyhow::anyhow;

// Serde library
use serde;

// Slang library
use shader_slang as slang;

// CRC64-fast library
use crc64fast_nvme as crc64;

// Local imports
use crate::*;
use crate::{compile::{SetEnvironmentError, AddModuleError}, slang::Program};



//////
//
// Errors
//

#[derive(Debug)]
pub enum LoadModuleError {
	CompilationError(String),
	InvalidModulePath(PathBuf),
	DuplicatePath(PathBuf)
}
impl Display for LoadModuleError {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::CompilationError(desc) => &format!("Compilation failed: {desc}"),
			Self::InvalidModulePath(path) => &format!("invalid module path: {}", path.display()),
			Self::DuplicatePath(path) => &format!("module already present at path: {}", path.display()),
		};
		write!(formatter, "LoadModuleError[{desc}]")
	}
}
impl Error for LoadModuleError {}



//////
//
// Enums
//

/// Indicates in what form a [`slang::Context`](Context) should enter modules into the active [`compile::Environment`]:
///
/// * `SourceCode` – The module should be stored as source code.
/// * `IR` – The module should be stored in *Slang*-IR form.
#[derive(Clone,Copy,serde::Serialize,serde::Deserialize)]
pub enum EnvironmentStorage {
	/// The module should be stored as source code.
	SourceCode,

	/// The module should be stored in *Slang*-IR form.
	IR
}

///
#[derive(Clone,serde::Serialize,serde::Deserialize)]
pub enum Module {
	/// The module should be stored as source code.
	SourceCode(String),

	/// The module should be stored in *Slang*-IR form.
	IR(Vec<u8>)
}
impl Module
{
	///
	#[inline(always)]
	fn fromSlangModule (slangModule: slang::Module) -> anyhow::Result<Self> {
		Ok(Self::IR(slangModule.serialize()?.as_slice().to_owned()))
	}

	///
	#[inline(always)]
	fn fromSlangSourceCode (sourceCode: &str) -> Self {
		Self::SourceCode(sourceCode.to_owned())
	}
}
impl compile::Module for Module {}



//////
//
// Structs
//

/// Helper struct for encapsulating [compatibility-relevant](Context::environmentCompatHash) Slang session options
#[derive(Default)]
struct CompatOptions {
	matrixLayoutColumn: bool,
	matrixLayoutRow: bool,
	optimize: bool
}
impl CompatOptions {
	pub fn matrixLayoutColumn(&mut self, enable: bool) -> bool {
		self.matrixLayoutColumn = enable;
		enable
	}
	pub fn matrixLayoutRow(&mut self, enable: bool) -> bool {
		self.matrixLayoutRow = enable;
		enable
	}
	pub fn optimize(&mut self, enable: bool) -> slang::OptimizationLevel {
		self.optimize = enable;
		if enable { slang::OptimizationLevel::Maximal } else { slang::OptimizationLevel::None }
	}
	pub fn digest (self) -> u64 {
		let mut digest = crc64::Digest::new();
		digest.write(util::slicify(&self));
		digest.sum64()
	}
}

/// Helper struct storing session configuration info in order to facilitate [Context::fork].
#[derive(Clone)]
struct SlangSessionConfig {
	searchPaths: Vec<std::ffi::CString>,
	target: CompilationTarget,
	compilerOptions: slang::CompilerOptions,
	profile: String,
}
impl SlangSessionConfig {
	pub fn searchPathsAsPointers(&self) -> Vec<*const i8> {
		self.searchPaths.iter().map(|p| p.as_ptr()).collect::<Vec<*const i8>>()
	}
}

/// # ToDos
///
/// *Slang* sessions are stateful. This is not necessarily something we always want. Consider using a fresh session in
/// the implementations of the [`compile...()`](compile::Context::compileModule) and
/// [`load...()`](compile::Context::loadModule) family of methods, as currently, they will cause the created module to
/// be embedded in the session, which can affect subsequent compilations of other modules in unexpected ways. The
/// [`compile::Context`] trait on the other hand has them take immutable references to `self` while very much _**not**_
/// implying any interior mutability.
pub struct Context {
	#[allow(dead_code)] // we need to keep this around as it dictates the lifetime of `session`
	globalSession: Rc<slang::GlobalSession>,

	sessionConfig: SlangSessionConfig,
	pub(crate) session: slang::Session,

	compatHash: u64,

	environment: Option<compile::Environment<Module>>
}
impl Context
{
	/// Helper for obtaining a fresh *Slang* session.
	fn freshSession (globalSession: &slang::GlobalSession, sessionConfig: &SlangSessionConfig)
	-> Result<slang::Session, ()>
	{
		let targetDesc = constructTargetDesc(&globalSession, &sessionConfig);
		globalSession.create_session(&slang::SessionDesc::default()
			.targets(&[targetDesc])
			.search_paths(sessionConfig.searchPathsAsPointers().as_slice())
			.options(&sessionConfig.compilerOptions)
		).ok_or(())
	}

	/// Create a new Slang context for the given compilation target using the given module search path.
	///
	/// # Arguments
	///
	/// * `target` – The target representation this `Context` will compile/transpile to.
	/// * `searchPath` – The module search path for the *Slang* compiler.
	pub fn forTarget (target: CompilationTarget, searchPath: &[impl AsRef<Path>]) -> anyhow::Result<Self>
	{
		// Start a Slang global session
		let globalSession = slang::GlobalSession::new();
		let globalSession = if globalSession.is_some() {
			globalSession.unwrap()
		}
		else {
			return Err(anyhow!("Failed to create Slang global session"));
		};

		// Finalize the Slang session configuration
		// - initialize compat-relevant settings
		let mut compatOptions = CompatOptions::default();
		// - compile flags
		let compilerOptions = slang::CompilerOptions::default()
			.matrix_layout_column(compatOptions.matrixLayoutColumn(true))
			.matrix_layout_row(compatOptions.matrixLayoutRow(false))
			.language(slang::SourceLanguage::Glsl);
		let compilerOptions = match target
		{
			CompilationTarget::SPIRV(debug) => compilerOptions
				.emit_spirv_directly(true)
				.optimization(
					if debug {compatOptions.optimize(false)} else { compatOptions.optimize(true)}
				)
				.debug_information(
					if debug { slang::DebugInfoLevel::Maximal } else { slang::DebugInfoLevel::None }
				),

			CompilationTarget::WGSL => compilerOptions
				.optimization(slang::OptimizationLevel::Maximal)
				.debug_information(slang::DebugInfoLevel::None)
		};
		// - store
		let sessionConfig = SlangSessionConfig {
			target, compilerOptions,
			searchPaths: searchPath.iter().map(|p| unsafe {
				std::ffi::CString::from_vec_unchecked(p.as_ref().to_string_lossy().as_bytes().to_vec())
			}).collect::<Vec<std::ffi::CString>>(),
			profile: "glsl_460".into()
		};

		// Create the stateful Slang compiler session
		let session = Self::freshSession(&globalSession, &sessionConfig).map_err(|_|
			anyhow!("Failed to create Slang context")
		)?;

		// Create the compatibility hash for our configuration
		let compatHash = compatOptions.digest();

		// Done!
		Ok(Self {
			globalSession: Rc::new(globalSession), sessionConfig, session, compatHash, environment: None
		})
	}

	/// Create a new Slang context with the given module search path. The actual creation is delegated to
	/// [`Self::forTarget`] using the default shader compilation target for the current target platform.
	///
	/// # Arguments
	///
	/// * `searchPath` – The module search path for the *Slang* compiler.
	pub fn new (searchPath: &[impl AsRef<Path>]) -> anyhow::Result<Self>
	{
		#[cfg(not(target_arch="wasm32"))] {
			Self::forTarget(CompilationTarget::SPIRV(cfg!(debug_assertions)), searchPath)
		}
		#[cfg(target_arch="wasm32")] {
			Self::forTarget(CompilationTarget::WGSL, searchPath)
		}
	}

	///
	pub fn targetType (&self) -> WgpuSourceType {
		match self.sessionConfig.target {
			CompilationTarget::SPIRV(_) => WgpuSourceType::SPIRV,
			CompilationTarget::WGSL => WgpuSourceType::WGSL
		}
	}

	/// Build a shader program from the given *Slang* source file.
	///
	/// # Arguments
	///
	/// * `sourceFile` – The `.slang` file containing the shader source code.
	pub fn buildProgram (&self, sourceFile: impl AsRef<Path>) -> anyhow::Result<Program> {
		Program::fromSource(self, sourceFile)
	}

	///
	pub fn compile (&self, sourcefile: impl AsRef<Path>) -> Result<slang::Module, LoadModuleError>
	{
		// Let slang load and compile the module
		let module =  self.session.load_module(
			sourcefile.as_ref().to_string_lossy().as_ref()
		).or_else(|err|
			Err(LoadModuleError::CompilationError(format!("File {} – {err}", sourcefile.as_ref().display())))
		)?;

		// Done!
		Ok(module)
	}

	///
	#[inline]
	pub fn compileFromSource (&self, sourceCode: &str) -> Result<slang::Module, LoadModuleError> {
		let targetPath = PathBuf::from(format!("_unnamed__{}.slang", util::unique::uint32()));
		self.compileFromNamedSource(&targetPath, sourceCode)
	}

	///
	pub fn compileFromNamedSource (&self, targetPath: impl AsRef<Path>, sourceCode: &str)
	-> Result<slang::Module, LoadModuleError>
	{
		// Make sure we get a valid target path
		let targetPath = validateModulePath(targetPath.as_ref())?;

		// Let slang compile the module
		let module =  self.session.load_module_from_source_string(targetPath, targetPath, sourceCode)
			.or_else(|err| Err(LoadModuleError::CompilationError(format!("{err}"))))?;

		// Done!
		Ok(module)
	}

	///
	pub fn loadModule (&mut self, filename: impl AsRef<Path>) -> Result<(), LoadModuleError>
	{
		let module = Module::fromSlangModule(self.compile(&filename)?).map_err(
			|err| LoadModuleError::CompilationError(format!("{err}"))
		)?;
		storeInEnvironment(self.environment.as_mut(), filename, module).map_err(|err| match err {
			AddModuleError::DuplicateModulePaths(path) => LoadModuleError::DuplicatePath(path)
		})
	}

	///
	pub fn loadModuleFromSource (
		&mut self, envStorage: EnvironmentStorage, targetPath: impl AsRef<Path>, sourceCode: &str
	) -> Result<(), LoadModuleError>
	{
		// Compile the source code inside the Slang session
		let slangModule = self.compileFromNamedSource(&targetPath, sourceCode)?;
		let module = match envStorage {
			EnvironmentStorage::SourceCode => Module::fromSlangSourceCode(sourceCode),
			EnvironmentStorage::IR => Module::fromSlangModule(slangModule).map_err(
				|err| LoadModuleError::CompilationError(format!("{err}"))
			)?
		};

		// Store the module in the environment
		storeInEnvironment(self.environment.as_mut(), targetPath, module).map_err(|err| match err {
			AddModuleError::DuplicateModulePaths(path) => LoadModuleError::DuplicatePath(path)
		})
	}

	///
	pub fn loadModuleFromIR (&mut self, targetPath: impl AsRef<Path>, bytes: &[u8])
		-> Result<(), LoadModuleError>
	{
		// Make sure we get a valid target path
		let targetPath_str = validateModulePath(targetPath.as_ref())?;

		// Load the IR bytecode blob into the Slang session
		let irBlob = slang::ComPtr::new(slang::VecBlob::from_slice(bytes));
		self.session.load_module_from_ir_blob(targetPath_str, targetPath_str, &*irBlob).or_else(
			|err| Err(LoadModuleError::CompilationError(format!("{err}")))
		)?;

		// Store the IR module in the environment
		storeInEnvironment(self.environment.as_mut(), targetPath, Module::IR(bytes.to_owned())).map_err(
			|err| match err {
				AddModuleError::DuplicateModulePaths(path) => LoadModuleError::DuplicatePath(path)
			}
		)
	}
}

impl compile::Context<Module> for Context
{
	fn replaceEnvironment (&mut self, environment: Option<compile::Environment<Module>>)
		-> Result<Option<compile::Environment<Module>>, compile::SetEnvironmentError>
	{
		// Check if the new environment is compatible (in case it's `Some`)
		if let Some(newEnv) = &environment && self.compatHash != newEnv.compatHash() {
			return Err(compile::SetEnvironmentError::IncompatibleEnvironment)
		}

		// Start from a fresh session
		let newSession = Self::freshSession(&self.globalSession, &self.sessionConfig).expect(
			"Creating a Slang session identical to an existing one should never fail unless there are \
			unrecoverable external circumstances (out-of-memory etc.)"
		);

		// Apply the new environment to the new session
		if let Some(newEnv) = &environment
		{
			for module in newEnv.modules()
			{
				let path = encodeValidModulePath(&module.path);
				match &module.module
				{
					Module::SourceCode(sourceCode) =>
						newSession.load_module_from_source_string(&path, "", sourceCode).or_else(|err|Err(
							SetEnvironmentError::ImplementationSpecific(
								LoadModuleError::CompilationError(format!("{err}")).into()
							)
						))?,

					Module::IR(bytes) => {
						let irBlob = slang::ComPtr::new(slang::VecBlob::from_slice(bytes));
						newSession.load_module_from_ir_blob(&path, "", &*irBlob).or_else(|err|Err(
							SetEnvironmentError::ImplementationSpecific(
								LoadModuleError::CompilationError(format!("{err}")).into()
							)
						))?
					}
				};
			}
		}

		// Commit both
		let oldEnv = self.environment.take();
		self.session = newSession;
		self.environment = environment;

		// Done!
		Ok(oldEnv)
	}

	fn finishEnvironment (self) -> Option<compile::Environment<Module>> {
		self.environment
	}

	fn environmentCompatHash (&self) -> u64 {
		self.compatHash
	}
}



//////
//
// Functions
//

///
fn constructTargetDesc<'caller> (globalSesssion: &slang::GlobalSession, sessionConfig: &SlangSessionConfig)
	-> slang::TargetDesc<'caller>
{
	let targetDesc = slang::TargetDesc::default().profile(
		globalSesssion.find_profile(&sessionConfig.profile)
	);
	match sessionConfig.target {
		CompilationTarget::SPIRV(_) => targetDesc.format(slang::CompileTarget::Spirv),
		CompilationTarget::WGSL => targetDesc.format(slang::CompileTarget::Wgsl)
	}
}

///
fn validateModulePath (targetPath: &Path) -> Result<&str, LoadModuleError>
{
	targetPath.parent().ok_or(
		LoadModuleError::InvalidModulePath(targetPath.to_owned())
	)?;
	targetPath.file_stem().ok_or(
		LoadModuleError::InvalidModulePath(targetPath.to_owned())
	)?;

	Ok(targetPath.as_os_str().to_str().ok_or(
		LoadModuleError::InvalidModulePath(targetPath.to_owned())
	)?)
}

///
#[inline]
fn /*decompose*/encodeValidModulePath (targetPath: &Path) -> /*(*/Cow<'_, str>//, Cow<'_, str>)
{
	targetPath.parent().ok_or(
		LoadModuleError::InvalidModulePath(targetPath.to_owned())
	).unwrap();
	targetPath.file_stem().ok_or(
		LoadModuleError::InvalidModulePath(targetPath.to_owned())
	).unwrap();

	targetPath.as_os_str().to_string_lossy()
}

///
#[inline]
fn storeInEnvironment (
	environment: Option<&mut compile::Environment<Module>>, atPath: impl AsRef<Path>, module: Module
) -> Result<(), AddModuleError>
{
	if let Some(env) = environment {
		// If we got an environment, put the module in it
		env.addModule(atPath, module)
	}
	else {
		// No environment, nothing to do
		Ok(())
	}
}
