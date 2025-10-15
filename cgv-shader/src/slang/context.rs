
//////
//
// Imports
//

// Standard library
use std::{rc::Rc, path::Path};

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
use crate::compile::LoadModuleError;
use crate::slang::Program;



//////
//
// Enums
//

/// Indicates how a [`slang::Context`](Context) should reflect modules in the active [`compile::Environment`].
#[derive(Clone,serde::Serialize,serde::Deserialize)]
pub enum EnvironmentStorage {
	/// The module should be stored as source code.
	SourceCode,

	/// The module should be stored in *Slang*-IR form.
	IR
}



//////
//
// Structs
//

///
#[derive(Clone,serde::Serialize,serde::Deserialize)]
pub struct Module{
	kind: EnvironmentStorage,
	code: Vec<u8>
}
impl Module
{
	///
	#[inline]
	fn fromSlangModule (slangModule: slang::Module) -> anyhow::Result<Self> {
		Ok(Self {
			kind: EnvironmentStorage::IR,
			code: slangModule.serialize()?.as_slice().to_owned()
		})
	}

	///
	#[inline]
	fn fromSlangSourceCode (sourceCode: &str) -> Self {
		Self {
			kind: EnvironmentStorage::SourceCode,
			code: sourceCode.as_bytes().to_owned()
		}
	}
}
impl compile::Module for Module {}

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

		// Create the reusable Slang compiler session
		let targetDesc = constructTargetDesc(&globalSession, &sessionConfig);
		let session = globalSession.create_session(&slang::SessionDesc::default()
			.targets(&[targetDesc])
			.search_paths(sessionConfig.searchPathsAsPointers().as_slice())
			.options(&sessionConfig.compilerOptions)
		);
		let session = if session.is_some() {
			session.unwrap()
		}
		else {
			return Err(anyhow!("Failed to create Slang context"));
		};

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

	/// Create a new Slang context inheriting all settings and the [environment](compile::Context::replaceEnvironment).
	pub fn fork (&self) -> Self
	{
		// Re-create the reusable Slang compiler session from the stored configuration
		let targetDesc = constructTargetDesc(&self.globalSession, &self.sessionConfig);
		let session = self.globalSession.create_session(&slang::SessionDesc::default()
			.targets(&[targetDesc])
			.search_paths(self.sessionConfig.searchPathsAsPointers().as_slice())
			.options(&self.sessionConfig.compilerOptions)
		).expect(
			"Creating a Slang session identical to an existing one should never fail unless there are \
			 unrecoverable external circumstances (out-of-memory etc.)"
		);

		// Done!
		Self {
			session,
			globalSession: self.globalSession.clone(),
			sessionConfig: self.sessionConfig.clone(),
			compatHash: self.compatHash,
			environment: self.environment.clone()
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

	pub fn compile (&self, sourcefile: impl AsRef<Path>) -> Result<slang::Module, compile::LoadModuleError>
	{
		// We operate on a forked context to avoid polluting our session (only the loadModule... family of methods
		// should leave modules in the session for later reuse/import/specialization)
		let module = // Let slang load and compile the module
			self.session.load_module(
				sourcefile.as_ref().to_string_lossy().as_ref()
			).or_else(|err| Err(LoadModuleError::ImplementationSpecific(
				anyhow!("Compilation of `{}` failed:\n{}", sourcefile.as_ref().display(), err)
			)))?;

		/*// Wrap the Slang module in our compile::Module-compliant representation
		let module = Module::fromSlangModule(module).or_else(
			|err| Err(LoadModuleError::ImplementationSpecific(
				anyhow!("Compilation of `{}` failed:\n{}", sourcefile.as_ref().display(), err)
			))
		)?;*/

		// Done!
		Ok(module)
	}

	fn compileModuleFromMemory(&self, _source: &str) -> std::result::Result<slang::Module, LoadModuleError> {
		todo!()
	}

	fn loadModule (&self, _filepath: impl AsRef<Path>) -> Result<Module, compile::LoadModuleError> {
		todo!()
	}

	fn loadModuleFromMemory (&self, _blob: &[u8]) -> Result<Module, compile::LoadModuleError> {
		todo!()
	}
}

impl compile::Context<Module> for Context
{
	fn replaceEnvironment (&mut self, environment: Option<compile::Environment<Module>>)
		-> std::result::Result<Option<compile::Environment<Module>>, compile::SetEnvironmentError>
	{
		if self.environment.is_some()
		{
			if let Some(newEnv) = environment
			{
				if self.compatHash != newEnv.compatHash() {
					return Err(compile::SetEnvironmentError::IncompatibleEnvironment)
				}
				todo!()
			}
			else {
				// Take the old environment out of the context and start with a fresh session
				let oldEnv = self.environment.take().unwrap();
				self.environment = None;
				todo!(); // replace the session with a fresh one
				return Ok(Some(oldEnv))
			}
		}
		else {
			todo!()
		}
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
#[inline(always)]
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
