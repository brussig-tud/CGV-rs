
//////
//
// Imports
//

// Standard library
use std::{path::{PathBuf, Path}, borrow::Cow, ops::Deref, sync::{Mutex, LazyLock}};

// Anyhow library
use anyhow::anyhow;

// Slang library
use shader_slang as slang;

// Local imports
use crate::*;
use crate::{compile, slang::Program, slang::context::*};
use compile::{SetEnvironmentError, AddModuleError};



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

impl From<&slang::Module> for EnvModule {
	fn from (value: &shader_slang::Module) -> Self {
		Self::fromSlangIRBytes(
			value.serialize().expect("Slang failed to serialize a pre-compiled module").as_slice()
		)
	}
}

///
pub struct NativeGlobalSessionContainer {
	globalSession: Mutex<slang::GlobalSession>
}
impl Deref for NativeGlobalSessionContainer {
	type Target =  Mutex<slang::GlobalSession>;

	fn deref (&self) -> &Self::Target {
		&self.globalSession
	}
}
unsafe impl Send for NativeGlobalSessionContainer {
	// SAFETY:
	// According to the [*Slang* docs](https://shader-slang.org/slang/user-guide/compiling.html#multithreading), global
	// session methods may be called from any thread, as long as they don't do it concurrently.
}
unsafe impl Sync for NativeGlobalSessionContainer {
	// SAFETY:
	// According to the [*Slang* docs](https://shader-slang.org/slang/user-guide/compiling.html#multithreading), global
	// session methods are **NOT** re-entrant. However, clients promise they will only ever create instances inside a
	// `LazyLock`, which enforces mutual exclusion while the initialization methods are run, and our internal mutex
	// enforces serial access to the global session after initialization.
}

/// Helper struct storing session configuration info to facilitate [`compile::Environment`] compatibility checking.
#[derive(Clone)]
struct SlangSessionConfig {
	searchPaths: Vec<std::ffi::CString>,
	targets: util::ds::BTreeUniqueVec<compile::Target>,
	compilerOptions: slang::CompilerOptions,
	profile: slang::ProfileID,
}
impl SlangSessionConfig {
	pub fn searchPathsAsPointers(&self) -> Vec<*const i8> {
		self.searchPaths.iter().map(|p| p.as_ptr()).collect::<Vec<*const i8>>()
	}
}


///
pub struct ContextBuilder {
	searchPath:  util::ds::HashUniqueVec<PathBuf>,
	targets: util::ds::BTreeUniqueVec<compile::Target>,
}
impl ContextBuilder {
	#[inline(always)]
	pub fn withTarget (target: compile::Target) -> Self { Self {
		searchPath: util::ds::UniqueVec::new(),
		targets: vec![target].into()
	}}

	#[inline(always)]
	pub fn withTargets (targets: &[compile::Target]) -> Self { Self {
		searchPath: util::ds::UniqueVec::new(),
		targets: targets.into()
	}}

	#[inline(always)]
	pub fn addTarget (&mut self, target: compile::Target) -> &mut Self {
		self.targets.push(target);
		self
	}

	#[inline(always)]
	pub fn addTargets (&mut self, targets: &[compile::Target]) -> &mut Self {
		self.targets.extend(targets.iter().copied());
		self
	}

	#[inline(always)]
	pub fn addSearchPath (&mut self, path: impl AsRef<Path>) -> &mut Self {
		self.searchPath.push(path.as_ref().to_owned());
		self
	}

	#[inline(always)]
	pub fn addSearchPaths (&mut self, paths: &[impl AsRef<Path>]) -> &mut Self {
		self.searchPath.extend(paths.iter().map(|p| p.as_ref().to_owned()));
		self
	}

	#[inline(always)]
	pub fn buildWithGlobalSession<'gs, 'ctx> (self, globalSession: &'gs slang::GlobalSession)
		-> Result<Context<'ctx>, compile::CreateContextError>
	{
		// Finalize the Slang session configuration
		// - initialize compat-relevant settings
		let mut compatOptions = CompatOptions::default();
		// - compile flags
		let compilerOptions = slang::CompilerOptions::default()
			.matrix_layout_column(compatOptions.matrixLayoutColumn(true))
			.matrix_layout_row(compatOptions.matrixLayoutRow(false));
		let mut debug = false;
		let spirv = self.targets.iter().find(|t| {
			match t {
				compile::Target::SPIRV(spv_debug) => { debug = *spv_debug; true },
				_ => false
			}
		}).is_some();
		self.targets.iter().find(|t| {
			match t {
				compile::Target::DXIL(dxil_debug) => { debug = debug || *dxil_debug; true },
				_ => false
			}
		});
		let compilerOptions = if spirv {
			compilerOptions.emit_spirv_directly(true)
		}
		else { compilerOptions };
		let compilerOptions = compilerOptions
			.optimization(
				if debug { slang::OptimizationLevel::None } else { slang::OptimizationLevel::Maximal }
			)
			.debug_information(
				if debug { slang::DebugInfoLevel::Maximal } else { slang::DebugInfoLevel::None }
			);

		// - store
		let sessionConfig = SlangSessionConfig {
			targets:self.targets.into(), compilerOptions,
			searchPaths: self.searchPath.iter().map(|p| unsafe {
				std::ffi::CString::from_vec_unchecked(p.to_string_lossy().as_bytes().to_vec())
			}).collect::<Vec<std::ffi::CString>>(),
			profile:  globalSession.find_profile("glsl_460")
		};

		// Create the stateful Slang compiler session
		let session = Context::freshSession(&globalSession, &sessionConfig).map_err(|_|
			anyhow!("Failed to create Slang context")
		).map_err(
			|_| compile::CreateContextError::ImplementationDefined(CreateSessionError::Generic.into())
		)?;

		// Create the compatibility hash for our configuration
		let compatHash = compatOptions.digest();

		// Done!
		Ok(Context { sessionConfig, session, compatHash, environment: None, _phantomData: Default::default() })
	}

	#[inline(always)]
	pub fn build<'ctx> (self) -> Result<Context<'ctx>, compile::CreateContextError> {
		let gs = obtainGlobalSession().lock().unwrap();
		self.buildWithGlobalSession(&gs)
	}
}
impl Default for ContextBuilder {
	fn default () -> Self { Self::withTarget(compile::Target::SPIRV(cfg!(debug_assertions))) }
}


/// A *Slang* [compilation context](compile::EnvironmentEnabled).
pub struct Context<'this> {
	sessionConfig: SlangSessionConfig,

	pub(crate)session: slang::Session,

	compatHash: u64,
	environment: Option<compile::Environment<EnvModule>>,

	_phantomData: std::marker::PhantomData<&'this ()>
}
impl Context<'_>
{
	/// Helper for obtaining a fresh *Slang* session.
	fn freshSession (globalSession: &slang::GlobalSession, sessionConfig: &SlangSessionConfig)
		-> Result<slang::Session, ()>
	{
		let targetDesc = constructTargetDescs(&sessionConfig);
		globalSession.create_session(&slang::SessionDesc::default()
			.targets(&targetDesc)
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
	pub fn forTarget (target: compile::Target, searchPath: &[impl AsRef<Path>])
		-> Result<Self, compile::CreateContextError>
	{
		// Sanity-check the target
		if !target.isWGSL() && !target.isSPIRV() {
			return Err(compile::CreateContextError::UnsupportedTarget(target));
		}

		// Setup builder for desired Context properties
		let mut builder = ContextBuilder::withTarget(target);
		builder.addSearchPaths(searchPath);

		// Done!
		builder.build()
	}

	/// Create a new *Slang* context for the *SPIR-V* target with the given module search path. The actual creation is
	/// delegated to [`Self::forTarget`] using the default shader compilation target, this function merely decides
	/// whether to enable debug information in the *SPIR-V* target based on `cfg!(debug_assertions)`.
	///
	/// # Arguments
	///
	/// * `searchPath` – The module search path for the *Slang* compiler.
	pub fn new (searchPath: &[impl AsRef<Path>]) -> Result<Self, compile::CreateContextError> {
		Self::forTarget(compile::Target::SPIRV(cfg!(debug_assertions)), searchPath)
	}

	///
	pub fn targetType (&self) -> Option<WgpuSourceType> {
		match self.sessionConfig.targets.first()? {
			compile::Target::SPIRV(_) => Some(WgpuSourceType::SPIRV),
			compile::Target::WGSL => Some(WgpuSourceType::WGSL),
			_ => None
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
		let module = EnvModule::fromSlangModule(self.compile(&filename)?).map_err(
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
			EnvironmentStorage::SourceCode => EnvModule::fromSlangSourceCode(sourceCode),
			EnvironmentStorage::IR => EnvModule::fromSlangModule(slangModule).map_err(
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
		storeInEnvironment(self.environment.as_mut(), targetPath, EnvModule::IR(bytes.to_owned())).map_err(
			|err| match err {
				AddModuleError::DuplicateModulePaths(path) => LoadModuleError::DuplicatePath(path)
			}
		)
	}
}
impl compile::EnvironmentEnabled<EnvModule> for Context<'_>
{
	fn replaceEnvironment (&mut self, environment: Option<compile::Environment<EnvModule>>)
		-> Result<Option<compile::Environment<EnvModule>>, compile::SetEnvironmentError>
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
					EnvModule::SourceCode(sourceCode) =>
						newSession.load_module_from_source_string(&path, "", sourceCode).or_else(|err|Err(
							SetEnvironmentError::ImplementationSpecific(
								LoadModuleError::CompilationError(format!("{err}")).into()
							)
						))?,

					EnvModule::IR(bytes) => {
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

	fn finishEnvironment (self) -> Option<compile::Environment<EnvModule>> {
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
	&SLANG_GLOBAL_SESSION
}

///
fn constructTargetDescs<'td> (sessionConfig: &SlangSessionConfig)
	-> Vec<slang::TargetDesc<'td>>
{
	sessionConfig.targets.iter().map(|target| {
		let targetDesc = slang::TargetDesc::default().profile(sessionConfig.profile);
		match target {
			compile::Target::SPIRV(_) => targetDesc.format(slang::CompileTarget::Spirv),
			compile::Target::WGSL => targetDesc.format(slang::CompileTarget::Wgsl),
			compile::Target::DXIL(_) => targetDesc.format(slang::CompileTarget::Dxil),
			compile::Target::GLSL => targetDesc.format(slang::CompileTarget::Glsl),
			compile::Target::HLSL => targetDesc.format(slang::CompileTarget::Hlsl),
			compile::Target::CudaCpp => targetDesc.format(slang::CompileTarget::CudaSource),
			compile::Target::Metal => targetDesc.format(slang::CompileTarget::Metal),
			_ => unimplemented!("unsupported target type")
		}
	}).collect()
}

///
#[inline]
fn encodeValidModulePath (targetPath: &Path) -> Cow<'_, str>
{
	targetPath.parent().ok_or(
		LoadModuleError::InvalidModulePath(targetPath.to_owned())
	).unwrap();
	targetPath.file_stem().ok_or(
		LoadModuleError::InvalidModulePath(targetPath.to_owned())
	).unwrap();

	targetPath.as_os_str().to_string_lossy()
}
