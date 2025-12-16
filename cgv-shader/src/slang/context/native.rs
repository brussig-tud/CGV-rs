
//////
//
// Imports
//

// Standard library
use std::{sync::{Mutex, LazyLock}, path::{PathBuf, Path}};

// Anyhow library
use anyhow::anyhow;

// Slang library
use shader_slang as slang;

// Local imports
use crate::*;
use crate::{compile::{SetEnvironmentError, AddModuleError}, slang::Program, slang::context::*};



//////
//
// Globals
//

/// The singleton global session instance.
static SLANG_GLOBAL_SESSION: LazyLock<NativeGlobalSessionContainer> = LazyLock::new(
	|| NativeGlobalSessionContainer { globalSession: Mutex::new(slang::GlobalSession::new().unwrap()) }
);



//////
//
// Structs
//

impl From<&slang::Module> for Module {
	fn from (value: &shader_slang::Module) -> Self {
		Self::fromSlangIRBytes(
			value.serialize().expect("Slang failed to serialize a pre-compiled module").as_slice()
		)
	}
}

///
struct NativeGlobalSessionContainer {
	globalSession: Mutex<slang::GlobalSession>
}
unsafe impl Send for NativeGlobalSessionContainer {
	// SAFETY: According to the [*Slang* docs](https://shader-slang.org/slang/user-guide/compiling.html#multithreading),
	// global session methods may be called from any thread, as long as they don't do it concurrently.
}
unsafe impl Sync for NativeGlobalSessionContainer {
	// SAFETY: According to the [*Slang* docs](https://shader-slang.org/slang/user-guide/compiling.html#multithreading),
	// global session methods are **NOT** re-entrant. However, we only ever create a single global session inside a
	// LazyLock, which enforces mutual exclusion when initializing it, and our internal mutex enforces serial access to
	// the global session after initialization.
}

/// Helper struct storing session configuration info to facilitate [`compile::Environment`] compatibility checking.
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

/// A *Slang* [compilation context](compile::Context).
pub struct Context {
	sessionConfig: SlangSessionConfig,

	pub(crate)session: slang::Session,

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

	/// Create a new *Slang* context for the given compilation target using the given module search path.
	///
	/// # Arguments
	///
	/// * `target` – The target representation this `Context` will compile/transpile to.
	/// * `searchPath` – The module search path for the *Slang* compiler.
	pub fn forTarget (target: CompilationTarget, searchPath: &[impl AsRef<Path>]) -> anyhow::Result<Self>
	{
		// Obtain the global session
		let globalSession = obtainGlobalSession().lock().unwrap();

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
		Ok(Self { sessionConfig, session, compatHash, environment: None })
	}

	/// Create a new *Slang* context for the *SPIR-V* target with the given module search path. The actual creation is
	/// delegated to [`Self::forTarget`] using the default shader compilation target, this function merely decides
	/// whether to enable debug information in the *SPIR-V* target based on `cfg!(debug_assertions)`.
	///
	/// # Arguments
	///
	/// * `searchPath` – The module search path for the *Slang* compiler.
	pub fn new (searchPath: &[impl AsRef<Path>]) -> anyhow::Result<Self> {
		Self::forTarget(CompilationTarget::SPIRV(cfg!(debug_assertions)), searchPath)
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
		Program::fromSourceFile(self, sourceFile)
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
	pub fn compileFromNamedSource (&self, virtualFilepath: impl AsRef<Path>, sourceCode: &str)
	                               -> Result<slang::Module, LoadModuleError>
	{
		// Make sure we get a valid target path
		let targetPath = validateModulePath(virtualFilepath.as_ref())?;

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
		&mut self, envStorage: EnvironmentStorage, virtualFilepath: impl AsRef<Path>, sourceCode: &str
	) -> Result<(), LoadModuleError>
	{
		// Compile the source code inside the Slang session
		let slangModule = self.compileFromNamedSource(&virtualFilepath, sourceCode)?;
		let module = match envStorage {
			EnvironmentStorage::SourceCode => Module::fromSlangSourceCode(sourceCode),
			EnvironmentStorage::IR => Module::fromSlangModule(slangModule).map_err(
				|err| LoadModuleError::CompilationError(format!("{err}"))
			)?
		};

		// Store the module in the environment
		storeInEnvironment(self.environment.as_mut(), virtualFilepath, module).map_err(|err| match err {
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
		self.session.load_module_from_ir_blob(targetPath_str, targetPath_str, &irBlob).or_else(
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
		let newSession = Self::freshSession(&obtainGlobalSession().lock().unwrap(), &self.sessionConfig).expect(
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
						newSession.load_module_from_ir_blob(&path, "", &irBlob).or_else(|err|Err(
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

/// Obtain a reference to the singleton [`slang::GlobalSession`](slang::GlobalSession) from which actual, stateful
/// compiler sessions can be created.
#[inline(always)]
pub fn obtainGlobalSession () -> &'static Mutex<slang::GlobalSession> {
	&SLANG_GLOBAL_SESSION.globalSession
}

///
fn constructTargetDesc<'caller> (globalSession: &slang::GlobalSession, sessionConfig: &SlangSessionConfig)
	-> slang::TargetDesc<'caller>
{
	let targetDesc = slang::TargetDesc::default().profile(
		globalSession.find_profile(&sessionConfig.profile)
	);
	match sessionConfig.target {
		CompilationTarget::SPIRV(_) => targetDesc.format(slang::CompileTarget::Spirv),
		CompilationTarget::WGSL => targetDesc.format(slang::CompileTarget::Wgsl)
	}
}
