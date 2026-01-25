
//////
//
// Imports
//

// Standard library
use std::{collections::BTreeMap, path::{PathBuf, Path}, borrow::Cow, ops::Deref, sync::{Mutex, LazyLock}};

// Anyhow library
use anyhow::anyhow;

// Slang library
use shader_slang as slang;

// Local imports
use crate::slang::*;
use crate::{compile, slang::context::*};



//////
//
// Globals
//

/// The singleton global session instance.
static GLOBAL_SESSION: LazyLock<GlobalSession> = LazyLock::new(
	|| GlobalSession(Mutex::new(slang::GlobalSession::new().unwrap()))
);

/// The common error message for all instances where entry point names are queried from *Slang*.
static MISSING_ENTRY_POINT_NAME_MSG: &str = "entry points should always have a name";



//////
//
// Structs
//

/// Convenience alias for our `compile::ComponentRef`.
pub type ComponentRef<'this> = compile::ComponentRef<'this, Module<'this>, EntryPoint<'this>, Composite<'this>>;

///
pub struct GlobalSession(Mutex<slang::GlobalSession>);
impl Deref for GlobalSession {
	type Target =  Mutex<slang::GlobalSession>;
	fn deref (&self) -> &Self::Target {
		&self.0
	}
}
unsafe impl Send for GlobalSession {
	// SAFETY:
	// According to the [*Slang* docs](https://shader-slang.org/slang/user-guide/compiling.html#multithreading), global
	// session methods may be called from any thread, as long as they don't do it concurrently.
}
unsafe impl Sync for GlobalSession {
	// SAFETY:
	// According to the [*Slang* docs](https://shader-slang.org/slang/user-guide/compiling.html#multithreading), global
	// session methods are **NOT** re-entrant. However, we guarantee that the only way to access the wrapped *Slang*
	// global session object is through our mutex.
}


///
pub(crate) struct Session<'this> {
	slangSession: slang::Session,
	activeTargetsMap: ActiveTargetsMap,
	gsPhantom: std::marker::PhantomData<&'this GlobalSession>
}
impl<'this> Session<'this> {
	fn new (slangSession: slang::Session, activeTargetsMap: ActiveTargetsMap) -> Self { Self {
		slangSession, activeTargetsMap, gsPhantom: Default::default()
	}}
}
impl Deref for Session<'_> {
	type Target = slang::Session;
	fn deref (&self) -> &Self::Target {
		&self.slangSession
	}
}


///
pub struct EntryPoint<'this> {
	component: slang::EntryPoint,
	modulePhantom: std::marker::PhantomData<&'this Module<'this>>
}
impl compile::Component for EntryPoint<'_> {
	type Id = std::ptr::NonNull<std::ffi::c_void>;
	fn id (&self) -> Self::Id {
		extractSlangObjectInstancePointer(&self.component)
	}
}
impl compile::EntryPoint for EntryPoint<'_> {
	fn name (&self) -> &str {
		self.component.function_reflection().name().expect(MISSING_ENTRY_POINT_NAME_MSG)
	}
}

///
pub struct Module<'this> {
	pub(crate) component: slang::Module,
	virtualPath: PathBuf,
	entryPoints: Vec<EntryPoint<'this>>
}
impl compile::Component for Module<'_> {
	type Id = std::ptr::NonNull<std::ffi::c_void>;
	fn id (&self) -> Self::Id {
		extractSlangObjectInstancePointer(&self.component)
	}
}
impl<'this> compile::Module<EntryPoint<'this>> for Module<'this>
{
	fn virtualFilepath (&self) -> &Path {
		&self.virtualPath
	}

	fn entryPoint (&self, name: &str) -> Option<&EntryPoint<'this>> {
		use compile::EntryPoint;
		self.entryPoints.iter().find(|ep| ep.name() == name)
	}

	fn entryPoints (&self) -> &[EntryPoint<'this>] {
		&self.entryPoints
	}
}

///
pub struct Composite<'this> {
	pub(crate) component: slang::ComponentType,
	sessionPhantom: std::marker::PhantomData<&'this Session<'this>>
}
impl compile::Component for Composite<'_> {
	type Id = std::ptr::NonNull<std::ffi::c_void>;
	fn id (&self) -> Self::Id {
		extractSlangObjectInstancePointer(&self.component)
	}
}
impl compile::Composite for Composite<'_> {}

///
pub struct LinkedComposite<'this> {
	pub(crate) component: slang::ComponentType,
	entryPointMap: BTreeMap<String, i64>,
	activeTargetsMap: &'this ActiveTargetsMap,
	sessionPhantom: std::marker::PhantomData<&'this Session<'this>>
}
impl LinkedComposite<'_> {
	pub fn entryPointMap (&self) -> &BTreeMap<String, i64> {
		&self.entryPointMap
	}
}
impl compile::LinkedComposite for LinkedComposite<'_>
{
	fn allEntryPointsCode (&self, target: compile::Target) -> Result<compile::ProgramCode, compile::TranslateError>
	{
		if let Some(targetIdx) = self.activeTargetsMap[target.slot()]
		{
			// Translate to target
			let code = self.component.target_code(targetIdx).or_else(|e|
				Err(compile::TranslateError::Backend(anyhow!("translation to {target} failed: {e}")))
			)?;

			// Done!
			Ok(if target.isBinary() {
				compile::ProgramCode::Binary(code.as_slice().to_owned())
			} else {
				compile::ProgramCode::Text(code.as_str().expect("{target} targets should be UTF-8-encoded").to_owned())
			})
		}
		else {
			Err(compile::TranslateError::InvalidTarget(target))
		}
	}

	fn entryPointCode (&self, target: compile::Target, entryPointIdx: usize)
		-> Option<Result<compile::ProgramCode, compile::TranslateError>>
	{
		if entryPointIdx as usize>= self.entryPointMap.len() {
			return None;
		}
		if let Some(targetIdx) = self.activeTargetsMap[target.slot()]
		{
			// Translate to target
			let translateResult = self.component.entry_point_code(
				entryPointIdx as i64, targetIdx
			).map_err(|e| Err(compile::TranslateError::Backend(anyhow!(
				"translating entry point {entryPointIdx}[''] to {target} failed: {e}"
			))));

			// Make sense of the translation result
			let code = if translateResult.is_ok() {
				translateResult.ok().unwrap()
			}
			else {
				return Some(translateResult.err().unwrap());
			};

			// Done!
			Some(Ok(if target.isBinary() {
				compile::ProgramCode::Binary(code.as_slice().to_owned())
			} else {
				compile::ProgramCode::Text(code.as_str().expect("{target} targets should be UTF-8-encoded").to_owned())
			}))
		}
		else {
			Some(Err(compile::TranslateError::InvalidTarget(target)))
		}
	}
}

impl From<&slang::Module> for EnvModule {
	fn from (value: &shader_slang::Module) -> Self {
		Self::fromSlangIRBytes(
			value.serialize().expect("Slang modules should always successfully serialize to IR bytes").as_slice()
		)
	}
}

/// Helper struct storing session configuration info to facilitate both [quick rebuilds](Context::freshSession) as well
/// as [`compile::Environment`] compatibility checking.
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
	pub fn buildWithGlobalSession (self, globalSession: &GlobalSession)
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

		// Store in reusable session config
		let sessionConfig = SessionConfig {
			targets: self.targets, compilerOptions,
			searchPaths: self.searchPath.iter().map(|p| unsafe {
				std::ffi::CString::from_vec_unchecked(p.to_string_lossy().as_bytes().to_vec())
			}).collect::<Vec<std::ffi::CString>>(),
			profile: globalSession.lock().unwrap().find_profile("glsl_460")
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
	fn withPlatformDefaults (platform: &util::meta::SupportedPlatform) -> Self { Self {
		targets: vec![compile::mostSuitableTargetForPlatform(platform)].into(),
		debug: !platform.isWasm() && platform.isDebug(),
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
impl compile::BuildsContextWithFilesystemAccess for ContextBuilder<'_> {
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
	session: Session<'this>,
	compatHash: u64,
	environment: Option<compile::Environment<EnvModule>>
}
impl Context<'_>
{
	/// Helper for obtaining a fresh *Slang* session.
	fn freshSession<'gs> (globalSession: &'gs GlobalSession, sessionConfig: &SessionConfig)
		-> Result<Session<'gs>, CreateSessionError>
	{
		let mut activeTargetsMap = ActiveTargetsMap::default();
		let targetDesc = constructTargetDescs(&sessionConfig, &mut activeTargetsMap);
		Ok(Session::new(globalSession.lock().unwrap().create_session(&slang::SessionDesc::default()
			.targets(&targetDesc)
			.search_paths(sessionConfig.searchPathsAsPointers().as_slice())
			.options(&sessionConfig.compilerOptions)
		).ok_or(CreateSessionError::Generic)?, activeTargetsMap))
	}
}
impl<'ctx> compile::Context for Context<'ctx>
{
	type ModuleType<'module> = Module<'module> where Self: 'module;
	type EntryPointType<'ep> = EntryPoint<'ep> where Self: 'ep;
	type CompositeType<'cp> = Composite<'cp>;
	type LinkedCompositeType<'lct> = LinkedComposite<'lct> where Self: 'lct;
	type Builder = ContextBuilder<'ctx>;

	fn supportsTarget (&self, target: compile::Target) -> bool {
		self.session.activeTargetsMap[target.slot()].is_some()
	}

	#[inline]
	fn compileFromSource (&self, sourceCode: &str) -> Result<Module<'_>, compile::LoadModuleError> {
		let targetPath = PathBuf::from(format!("_unnamed__{}.slang", util::unique::uint32()));
		self.compileFromNamedSource(&targetPath, sourceCode)
	}

	fn compileFromNamedSource (&self, virtualFilepath: impl AsRef<Path>, sourceCode: &str)
		-> Result<Module<'_>, compile::LoadModuleError>
	{
		// Make sure we get a valid target path
		let targetPath = validateModulePath(virtualFilepath.as_ref())?;

		// Let slang compile the module
		let module =  self.session.load_module_from_source_string(
			targetPath, targetPath, sourceCode
		).or_else(
			|err| Err(compile::LoadModuleError::CompilationError(format!("{err}")))
		)?;

		// Enumerate and save entry points
		let entryPoints = module.entry_points().map(
			|ep| EntryPoint { component: ep, modulePhantom: Default::default() }
		).collect();

		// Done!
		Ok(Module { component: module, virtualPath: virtualFilepath.as_ref().to_owned(), entryPoints })
	}

	fn createComposite<'this, 'inner> (
		&'this self, components: &'inner [ComponentRef<'this>]
	) -> Result<Composite<'this>, compile::CreateCompositeError>
	{
		// Gather component list
		let components: Vec<_> = components.iter().map(|component| match component {
			ComponentRef::Module(module) => module.component.clone().into(),
			ComponentRef::EntryPoint(entryPoint) => entryPoint.component.clone().into(),
			ComponentRef::Composite(composite) => composite.component.clone().into()
		}).collect();

		// Composit
		let composite = self.session.create_composite_component_type(
			components.as_slice()
		).or_else(|err| Err(
			compile::CreateCompositeError::ImplementationSpecific(anyhow!("layout error: {err}"))
		))?;

		// Done!
		Ok(Composite { component: composite, sessionPhantom: Default::default() })
	}

	fn linkComposite (&self, composite: &Composite) -> Result<LinkedComposite<'_>, compile::LinkError>
	{
		// Link
		let componentType = composite.component.link().or_else(|err| Err(
			compile::LinkError::ImplementationSpecific(anyhow!("link failure: {err}"))
		))?;

		// Enumerate all entry points. We blanket-use the very first target, as names and ordering of entry points
		// should be completely target-independent. We can infer this logical guarantee from the fact that according to
		// several official Slang examples, you can – and in fact are typically expected to – use the entry point
		// information obtained prior to linking from untranslated *Slang* modules.
		let layout = componentType.layout(0).or_else(|err| Err(
			compile::LinkError::ImplementationSpecific(anyhow!("layout error: {err}"))
		))?;
		let mut entryPointMap = BTreeMap::default();
		for (idx, ep) in layout.entry_points().enumerate() {
			entryPointMap.insert(ep.name().expect(MISSING_ENTRY_POINT_NAME_MSG).to_owned(), idx as i64);
		}

		// Done!
		Ok(LinkedComposite {
			component: componentType, entryPointMap, activeTargetsMap: &self.session.activeTargetsMap,
			sessionPhantom: Default::default()
		})
	}
}
impl compile::HasFileSystemAccess for Context<'_>
{
	fn compile (&self, sourceFile: impl AsRef<Path>) -> Result<Module<'_>, compile::LoadModuleError>
	{
		// Let slang load and compile the module
		let module =  self.session.load_module(
			sourceFile.as_ref().to_string_lossy().as_ref()
		).or_else(
			|err| Err(compile::LoadModuleError::CompilationError(format!("{err}")))
		)?;

		// Enumerate and save entry points
		let entryPoints = module.entry_points().map(
			|ep| EntryPoint { component: ep, modulePhantom: Default::default() }
		).collect();

		// Done!
		Ok(Module { component: module, virtualPath: sourceFile.as_ref().to_owned(), entryPoints })
	}
}
impl compile::EnvironmentEnabled for Context<'_>
{
	type ModuleType = EnvModule;
	type EnvStorageHint = EnvironmentStorage;

	fn loadModule (&mut self, filename: impl AsRef<Path>) -> Result<(), compile::LoadModuleError>
	{
		use compile::HasFileSystemAccess;
		let module = EnvModule::fromSlangModule(self.compile(&filename)?.component).map_err(
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
			EnvironmentStorage::IR => EnvModule::fromSlangModule(slangModule.component).map_err(
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
							compile::SetEnvironmentError::ImplementationSpecific(
								compile::LoadModuleError::CompilationError(format!("{err}")).into()
							)
						))?,

					EnvModule::IR(bytes) => {
						let irBlob = slang::ComPtr::new(slang::VecBlob::from_slice(bytes));
						newSession.load_module_from_ir_blob(&path, "", &irBlob).or_else(|err|Err(
							compile::SetEnvironmentError::ImplementationSpecific(
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

/// Obtain a reference to a `'static` [`slang::GlobalSession`](GlobalSession) from which actual, stateful compiler
/// sessions can be created. *CGV-rs* uses this global session for all its internal shader compilation tasks.
#[inline(always)]
pub fn obtainGlobalSession () -> &'static GlobalSession {
	&GLOBAL_SESSION
}

///
fn constructTargetDescs<'td> (sessionConfig: &SessionConfig, activeTargetsMap: &mut ActiveTargetsMap)
	-> Vec<slang::TargetDesc<'td>>
{
	sessionConfig.targets.iter().enumerate().map(|(idx, target)|
	{
		let targetDesc = slang::TargetDesc::default().profile(sessionConfig.profile);
		activeTargetsMap[target.slot()] = Some(idx as i64);
		match target {
			compile::Target::SPIRV => targetDesc.format(slang::CompileTarget::Spirv),
			compile::Target::WGSL => targetDesc.format(slang::CompileTarget::Wgsl),
			compile::Target::DXIL => targetDesc.format(slang::CompileTarget::Dxil),
			compile::Target::GLSL => targetDesc.format(slang::CompileTarget::Glsl),
			compile::Target::HLSL => targetDesc.format(slang::CompileTarget::Hlsl),
			compile::Target::CudaCpp => targetDesc.format(slang::CompileTarget::CudaSource),
			compile::Target::Metal => targetDesc.format(slang::CompileTarget::Metal),
			_ => unreachable!("unsupported target types should have been rejected by earlier logic")
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
