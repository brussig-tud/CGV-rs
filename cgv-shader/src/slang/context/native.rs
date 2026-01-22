
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
	|| NativeGlobalSessionContainer(Mutex::new(slang::GlobalSession::new().unwrap()))
);



//////
//
// Structs
//

///
pub struct NativeGlobalSessionContainer(Mutex<slang::GlobalSession>);
impl Deref for NativeGlobalSessionContainer {
	type Target =  Mutex<slang::GlobalSession>;

	fn deref (&self) -> &Self::Target {
		&self.0
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


///
pub(crate) struct Session<'this> {
	slangSession: slang::Session,
	_globalSession: &'this NativeGlobalSessionContainer
}
impl<'this> Session<'this> {
	fn new (slangSession: slang::Session, globalSession: &'this NativeGlobalSessionContainer) -> Self { Self {
		slangSession, _globalSession: globalSession
	}}
}
impl Deref for Session<'_> {
	type Target = slang::Session;
	fn deref (&self) -> &Self::Target {
		&self.slangSession
	}
}


///
pub struct EntryPoint(pub slang::EntryPoint);
impl compile::Component for EntryPoint {
	type Id = std::ptr::NonNull<std::ffi::c_void>;
	fn id (&self) -> Self::Id {
		extractSlangObjectInstancePointer(&self.0)
	}
}
impl compile::EntryPoint for EntryPoint {
	fn name (&self) -> &str {
		todo!()
	}
}

///
pub struct Module(pub slang::Module);
impl compile::Component for Module {
	type Id = std::ptr::NonNull<std::ffi::c_void>;
	fn id (&self) -> Self::Id {
		extractSlangObjectInstancePointer(&self.0)
	}
}
impl compile::Module<EntryPoint> for Module {
	fn virtualFilepath (&self) -> &Path {
		todo!()
	}

	fn entryPoint (&self, _name: &str) -> Option<&EntryPoint> {
		todo!()
	}

	fn entryPoints (&self) -> &[EntryPoint] {
		todo!()
	}
}

///
pub struct Composite(pub slang::ComponentType);
impl compile::Component for Composite {
	type Id = std::ptr::NonNull<std::ffi::c_void>;
	fn id (&self) -> Self::Id {
		extractSlangObjectInstancePointer(&self.0)
	}
}
impl compile::Composite for Composite {}

///
pub struct LinkedComposite;
impl compile::LinkedComposite for LinkedComposite {
	fn allEntryPointsCode (_target: compile::Target) -> Result<compile::ProgramCode, compile::TranslateError> {
		todo!()
	}

	fn entryPointCode (_target: compile::Target, _entryPointIdx: u32)
		-> Option<Result<compile::ProgramCode, compile::TranslateError>>
	{
		todo!()
	}
}

impl From<&slang::Module> for EnvModule {
	fn from (value: &shader_slang::Module) -> Self {
		Self::fromSlangIRBytes(
			value.serialize().expect("Slang failed to serialize a pre-compiled module").as_slice()
		)
	}
}

/// Helper struct storing session configuration info to facilitate [`compile::Environment`] compatibility checking.
#[derive(Clone)]
struct SessionConfig {
	searchPaths: Vec<std::ffi::CString>,
	targets: util::ds::BTreeUniqueVec<compile::Target>,
	compilerOptions: slang::CompilerOptions,
	profile: slang::ProfileID,
}
impl SessionConfig {
	pub fn searchPathsAsPointers(&self) -> Vec<*const i8> {
		self.searchPaths.iter().map(|p| p.as_ptr()).collect::<Vec<*const i8>>()
	}
}


///
pub struct ContextBuilder<'ctx> {
	targets: util::ds::BTreeUniqueVec<compile::Target>,
	debug: bool,
	searchPath: util::ds::HashUniqueVec<PathBuf>,
	lifetimePhantom: std::marker::PhantomData<&'ctx ()>
}
impl ContextBuilder<'_>
{
	#[inline(always)]
	pub fn buildWithGlobalSession (self, globalSession: &NativeGlobalSessionContainer)
		-> Result<Context<'_>, compile::CreateContextError>
	{
		// Finalize the Slang session configuration
		// - initialize compat-relevant settings
		let mut compatOptions = CompatOptions::default();
		// - compile flags
		let compilerOptions = slang::CompilerOptions::default()
			.matrix_layout_column(compatOptions.matrixLayoutColumn(true))
			.matrix_layout_row(compatOptions.matrixLayoutRow(false));
		let compilerOptions = if self.targets.contains(&compile::Target::SPIRV) {
			compilerOptions.emit_spirv_directly(true)
		}
		else { compilerOptions };
		let compilerOptions = compilerOptions
			.optimization(
				if self.debug { slang::OptimizationLevel::None } else { slang::OptimizationLevel::Maximal }
			)
			.debug_information(
				if self.debug { slang::DebugInfoLevel::Maximal } else { slang::DebugInfoLevel::None }
			);

		// - store
		let sessionConfig = SessionConfig {
			targets:self.targets.into(), compilerOptions,
			searchPaths: self.searchPath.iter().map(|p| unsafe {
				std::ffi::CString::from_vec_unchecked(p.to_string_lossy().as_bytes().to_vec())
			}).collect::<Vec<std::ffi::CString>>(),
			profile:  globalSession.lock().unwrap().find_profile("glsl_460")
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
		Ok(Context { sessionConfig, session, compatHash, environment: None })
	}
}
impl Default for ContextBuilder<'_> {
	fn default () -> Self { Self {
		targets: vec![compile::mostSuitableTarget()].into(),
		debug: cfg!(debug_assertions),
		searchPath: Default::default(),
		lifetimePhantom: Default::default()
	}}
}
impl<'ctx> compile::ContextBuilder for ContextBuilder<'ctx>
{
	type Context = Context<'ctx>;

	#[inline(always)]
	fn defaultForPlatform (platform: &util::meta::SupportedPlatform) -> Self { Self {
		targets: vec![compile::mostSuitableTargetForPlatform(platform)].into(),
		debug: platform.isDebug(),
		..Default::default()
	}}

	#[inline(always)]
	fn withTargets (targets: impl AsRef<[compile::Target]>) -> Self { Self {
		targets: targets.as_ref().into(),
		..Default::default()
	}}

	#[inline(always)]
	fn addTargets (mut self, targets: &[compile::Target]) -> Self {
		self.targets.extend(targets.iter().copied());
		self
	}

	#[inline(always)]
	fn build (self) -> Result<Self::Context, compile::CreateContextError> {
		self.buildWithGlobalSession(obtainGlobalSession())
	}
}
impl compile::WithFilesystemAccess for ContextBuilder<'_> {
	#[inline(always)]
	fn withSearchPaths (paths: &[impl AsRef<Path>]) -> Self { Self {
		searchPath: paths.iter().map(|p| p.as_ref().to_owned()).collect(),
		..Default::default()
	}}

	#[inline(always)]
	fn addSearchPaths (mut self, paths: &[impl AsRef<Path>]) -> Self {
		self.searchPath.extend(paths.iter().map(|p| p.as_ref().to_owned()));
		self
	}
}


/// A *Slang* [compilation context](compile::EnvironmentEnabled).
pub struct Context<'this> {
	sessionConfig: SessionConfig,

	pub(crate) session: Session<'this>,

	compatHash: u64,
	environment: Option<compile::Environment<EnvModule>>
}
impl Context<'_>
{
	/// Helper for obtaining a fresh *Slang* session.
	fn freshSession<'gs> (globalSession: &'gs NativeGlobalSessionContainer, sessionConfig: &SessionConfig)
		-> Result<Session<'gs>, ()>
	{
		let targetDesc = constructTargetDescs(&sessionConfig);
		Ok(Session::new(globalSession.lock().unwrap().create_session(&slang::SessionDesc::default()
			.targets(&targetDesc)
			.search_paths(sessionConfig.searchPathsAsPointers().as_slice())
			.options(&sessionConfig.compilerOptions)
		).ok_or(())?, globalSession))
	}

	///
	pub fn targetType (&self) -> Option<WgpuSourceType> {
		match self.sessionConfig.targets.first()? {
			compile::Target::SPIRV => Some(WgpuSourceType::SPIRV),
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
	pub fn compile (&self, sourcefile: impl AsRef<Path>) -> Result<slang::Module, compile::LoadModuleError>
	{
		// Let slang load and compile the module
		let module =  self.session.load_module(
			sourcefile.as_ref().to_string_lossy().as_ref()
		).or_else(|err|
			Err(compile::LoadModuleError::CompilationError(format!("File {} – {err}", sourcefile.as_ref().display())))
		)?;

		// Done!
		Ok(module)
	}
}
impl<'ctx> compile::Context for Context<'ctx>
{
	type ModuleType<'module> = Module where Self: 'module;
	type EntryPointType<'ep> = EntryPoint where Self: 'ep;
	type CompositeType<'cp> = Composite;
	type LinkedCompositeType<'lct> = LinkedComposite where Self: 'lct;
	type Builder = ContextBuilder<'ctx>;

	#[inline]
	fn compileFromSource (&self, sourceCode: &str) -> Result<Module, compile::LoadModuleError> {
		let targetPath = PathBuf::from(format!("_unnamed__{}.slang", util::unique::uint32()));
		self.compileFromNamedSource(&targetPath, sourceCode)
	}

	fn compileFromNamedSource (&self, virtualFilepath: impl AsRef<Path>, sourceCode: &str)
		-> Result<Module, compile::LoadModuleError>
	{
		// Make sure we get a valid target path
		let targetPath = validateModulePath(virtualFilepath.as_ref())?;

		// Let slang compile the module
		let module =  self.session.load_module_from_source_string(targetPath, targetPath, sourceCode)
			.or_else(|err| Err(compile::LoadModuleError::CompilationError(format!("{err}"))))?;

		// Done!
		Ok(Module(module))
	}

	fn createComposite<'this> (
		&'this self, _components: &[compile::ComponentRef<'this, Module, EntryPoint, Composite>]
	) -> Result<Composite, compile::CreateCompositeError> {
		todo!("implement via to-be-completed unified Slang interface")
	}

	fn linkComposite (&self, _composite: &Composite) -> Result<LinkedComposite, compile::LinkError> {
		todo!("implement via to-be-completed unified Slang interface")
	}
}
impl compile::EnvironmentEnabled for Context<'_>
{
	type ModuleType = EnvModule;
	type EnvStorageHint = EnvironmentStorage;

	fn loadModule (&mut self, filename: impl AsRef<Path>) -> Result<(), compile::LoadModuleError>
	{
		let module = EnvModule::fromSlangModule(self.compile(&filename)?).map_err(
			|err| compile::LoadModuleError::CompilationError(format!("{err}"))
		)?;
		storeInEnvironment(self.environment.as_mut(), filename, module).map_err(|err| match err {
			AddModuleError::DuplicateModulePaths(path) => compile::LoadModuleError::DuplicatePath(path)
		})
	}

	fn loadModuleFromSource (
		&mut self, envStorage: EnvironmentStorage, virtualFilepath: impl AsRef<Path>, sourceCode: &str
	) -> Result<(), compile::LoadModuleError>
	{
		// Compile the source code inside the Slang session
		use compile::Context;
		let slangModule = self.compileFromNamedSource(&virtualFilepath, sourceCode)?;
		let module = match envStorage {
			EnvironmentStorage::SourceCode => EnvModule::fromSlangSourceCode(sourceCode),
			EnvironmentStorage::IR => EnvModule::fromSlangModule(slangModule.0).map_err(
				|err| compile::LoadModuleError::CompilationError(format!("{err}"))
			)?
		};

		// Store the module in the environment
		storeInEnvironment(self.environment.as_mut(), virtualFilepath, module).map_err(|err| match err {
			AddModuleError::DuplicateModulePaths(path) => compile::LoadModuleError::DuplicatePath(path)
		})
	}

	fn loadModuleFromIR (&mut self, targetPath: impl AsRef<Path>, bytes: &[u8]) -> Result<(), compile::LoadModuleError>
	{
		// Make sure we get a valid target path
		let targetPath_str = validateModulePath(targetPath.as_ref())?;

		// Load the IR bytecode blob into the Slang session
		let irBlob = slang::ComPtr::new(slang::VecBlob::from_slice(bytes));
		self.session.load_module_from_ir_blob(targetPath_str, targetPath_str, &irBlob).or_else(
			|err| Err(compile::LoadModuleError::CompilationError(format!("{err}")))
		)?;

		// Store the IR module in the environment
		storeInEnvironment(self.environment.as_mut(), targetPath, EnvModule::IR(bytes.to_owned())).map_err(
			|err| match err {
				AddModuleError::DuplicateModulePaths(path) => compile::LoadModuleError::DuplicatePath(path)
			}
		)
	}

	fn replaceEnvironment (&mut self, environment: Option<compile::Environment<EnvModule>>)
		-> Result<Option<compile::Environment<EnvModule>>, compile::SetEnvironmentError>
	{
		// Check if the new environment is compatible (in case it's `Some`)
		if let Some(newEnv) = &environment && self.compatHash != newEnv.compatHash() {
			return Err(compile::SetEnvironmentError::IncompatibleEnvironment)
		}

		// Start from a fresh session
		let newSession = Self::freshSession(&obtainGlobalSession(), &self.sessionConfig).expect(
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
								compile::LoadModuleError::CompilationError(format!("{err}")).into()
							)
						))?,

					EnvModule::IR(bytes) => {
						let irBlob = slang::ComPtr::new(slang::VecBlob::from_slice(bytes));
						newSession.load_module_from_ir_blob(&path, "", &irBlob).or_else(|err|Err(
							SetEnvironmentError::ImplementationSpecific(
								compile::LoadModuleError::CompilationError(format!("{err}")).into()
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

///
#[inline(always)]
fn extractSlangObjectInstancePointer (slangObject: &impl slang::Interface) -> std::ptr::NonNull<std::ffi::c_void> {
	util::static_assertions::assert_eq_size!(slang::IUnknown, std::ptr::NonNull<std::ffi::c_void>);
	unsafe {
		// SAFETY:
		// A `slang::IUnknown` is a 1-element tuple struct wrapping a C pointer (asserted above), so we can safely
		// transmute-copy it to a new pointer variable (the Rust standard says that single element tuple-structs are
		// guaranteed to have the exact same memory layout as their sole element).
		std::mem::transmute_copy(slangObject.as_unknown())
	}
}

/// Obtain a reference to the singleton [`slang::GlobalSession`](slang::GlobalSession) from which actual, stateful
/// compiler sessions can be created.
#[inline(always)]
pub fn obtainGlobalSession () -> &'static NativeGlobalSessionContainer {
	&SLANG_GLOBAL_SESSION
}

///
fn constructTargetDescs<'td> (sessionConfig: &SessionConfig)
	-> Vec<slang::TargetDesc<'td>>
{
	sessionConfig.targets.iter().map(|target| {
		let targetDesc = slang::TargetDesc::default().profile(sessionConfig.profile);
		match target {
			compile::Target::SPIRV => targetDesc.format(slang::CompileTarget::Spirv),
			compile::Target::WGSL => targetDesc.format(slang::CompileTarget::Wgsl),
			compile::Target::DXIL => targetDesc.format(slang::CompileTarget::Dxil),
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
		compile::LoadModuleError::InvalidModulePath(targetPath.to_owned())
	).unwrap();
	targetPath.file_stem().ok_or(
		compile::LoadModuleError::InvalidModulePath(targetPath.to_owned())
	).unwrap();

	targetPath.as_os_str().to_string_lossy()
}
