
//////
//
// Imports
//

// Standard library
use std::path::Path;

// Anyhow library
use anyhow::anyhow;

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
// Structs
//

///
#[derive(Clone)]
pub struct Module {
	slangModule: slang::Module,
	irBlob: slang::Blob
}
impl Module {
	///
	pub fn fromSlangModule (slangModule: slang::Module) -> anyhow::Result<Self> {
		Ok(Self { irBlob: slangModule.serialize()?, slangModule })
	}

	///
	pub fn slangModule (&self) -> &slang::Module {
		&self.slangModule
	}

	///
	pub fn irBlob (&self) -> &slang::Blob {
		&self.irBlob
	}

	///
	pub fn irBytecode (&self) -> &[u8] {
		self.irBlob.as_slice()
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
	globalSession: slang::GlobalSession,

	pub(crate) session: slang::Session,

	pub compilationTarget: SourceType,

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
		// Convert search path for FFI
		// - create owned storage for the CStrings
		let searchPaths = searchPath.iter().map(|p| unsafe {
			std::ffi::CString::from_vec_unchecked(p.as_ref().to_string_lossy().as_bytes().to_vec())
		}).collect::<Vec<std::ffi::CString>>();
		// - build array of raw pointers required by the FFI
		let searchPaths = searchPaths.iter().map(|p|
			p.as_ptr()
		).collect::<Vec<*const i8>>();

		// Start a Slang global session
		let globalSession = slang::GlobalSession::new();
		let globalSession = if globalSession.is_some() {
			globalSession.unwrap()
		}
		else {
			return Err(anyhow!("Failed to create Slang global session"));
		};

		// Finalize the slang context with our CGV-rs specific options
		// - initialize compat-relevant settings
		let mut compatOptions = CompatOptions::default();
		// - compile flags
		let sessionOptions = slang::CompilerOptions::default()
			.matrix_layout_column(compatOptions.matrixLayoutColumn(true))
			.matrix_layout_row(compatOptions.matrixLayoutRow(false))
			.language(slang::SourceLanguage::Glsl);
		let sessionOptions = match target
		{
			CompilationTarget::SPIRV(debug) => sessionOptions
				.emit_spirv_directly(true)
				.optimization(
					if debug {compatOptions.optimize(false)} else { compatOptions.optimize(true)}
				)
				.debug_information(
					if debug { slang::DebugInfoLevel::Maximal } else { slang::DebugInfoLevel::None }
				),

			CompilationTarget::WGSL => sessionOptions
				.optimization(slang::OptimizationLevel::Maximal)
				.debug_information(slang::DebugInfoLevel::None)
		};
		// - output profile
		let compilationTarget;
		let targetDesc = slang::TargetDesc::default()
			.profile(globalSession.find_profile("glsl_460"));
		let targetDesc = match target {
			CompilationTarget::SPIRV(_) => {
				compilationTarget = SourceType::SPIRV;
				targetDesc.format(slang::CompileTarget::Spirv)
			},
			CompilationTarget::WGSL => {
				compilationTarget = SourceType::WGSL;
				targetDesc.format(slang::CompileTarget::Wgsl)
			}
		};

		let targets = &[targetDesc];
		// - the reusable compiler session
		let session = globalSession.create_session(&slang::SessionDesc::default()
			.targets(targets)
			.search_paths(searchPaths.as_slice())
			.options(&sessionOptions)
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
			globalSession, session, compilationTarget, compatHash, environment: None
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

	/// Build a shader program from the given *Slang* source file.
	///
	/// # Arguments
	///
	/// * `sourceFile` – The `.slang` file containing the shader source code.
	pub fn buildProgram (&self, sourceFile: impl AsRef<Path>) -> anyhow::Result<Program> {
		Program::fromSource(self, sourceFile)
	}
}

impl compile::Context<Module> for Context
{
	fn replaceEnvironment (&mut self, environment: Option<&compile::Environment<Module>>)
		-> std::result::Result<(), compile::SetEnvironmentError>
	{
		if let Some(_curEnv) = &self.environment
		{
			if let Some(newEnv) = environment
			{
				if self.compatHash != newEnv.compatHash() {
					return Err(compile::SetEnvironmentError::IncompatibleEnvironment)
				}
				todo!()
			}
			else {
				self.environment = None;
				todo!()
			}
		}
		else {
			todo!()
		}
	}

	fn environmentCompatHash (&self) -> u64 {
		self.compatHash
	}

	fn compileModule (&self, sourcefile: impl AsRef<Path>) -> Result<Module, compile::LoadModuleError>
	{
		// Let slang load and compile the module
		let module = self.session.load_module(
			sourcefile.as_ref().to_string_lossy().as_ref()
		).or_else(|err| Err(LoadModuleError::ImplementationSpecific(
			anyhow!("Compilation of `{}` failed:\n{}", sourcefile.as_ref().display(), err)
		)))?;

		// Wrap the Slang module in our compile::Module-compliant representation
		let module = Module::fromSlangModule(module).or_else(
			|err| Err(LoadModuleError::ImplementationSpecific(
				anyhow!("Compilation of `{}` failed:\n{}", sourcefile.as_ref().display(), err)
			))
		)?;

		// Done!
		Ok(module)
	}

	fn compileModuleFromMemory(&self, _source: &str) -> std::result::Result<Module, LoadModuleError> {
		todo!()
	}

	fn loadModule (&self, _filepath: impl AsRef<Path>) -> Result<Module, compile::LoadModuleError> {
		todo!()
	}

	fn loadModuleFromMemory (&self, _blob: &[u8]) -> Result<Module, compile::LoadModuleError> {
		todo!()
	}
}
