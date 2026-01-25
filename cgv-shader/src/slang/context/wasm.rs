
//////
//
// Imports
//

// Standard library
use std::{path::{PathBuf, Path}, sync::LazyLock};

// Wasm-bindgen library
use wasm_bindgen::prelude::*;

// Local imports
use crate::*;
use crate::{compile, slang::{Program, context::*}};



//////
//
// Globals
//

/// A realm of unique unsigned 32-bit integers.
pub static GLOBAL_SESSION: LazyLock<GlobalSession> = LazyLock::new(|| GlobalSession::new().unwrap());



//////
//
// Structs
//

/// Alias for our `compile::ComponentRef`.
type ComponentRef<'sess> = compile::ComponentRef<
	'sess, JsSlangModule<'sess>, SlangEntryPoint<'sess>, SlangComposite<'sess>
>;

/// A handle for a JavaScript-side `slang::GlobalSession` instance.
pub struct GlobalSession(u64);
impl GlobalSession {
	pub(crate) fn new () -> Option<Self> {
		let handle = slangjs_createGlobalSession();
		if handle > 0 {
			Some(Self(handle as u64))
		}
		else { None }
	}

	pub fn createSession (&self) -> Result<Session<'_>, CreateSessionError>
	{
		let handle = slangjs_GlobalSession_createSession(self.0);
		if handle > 0 {
			Ok(Session { handle: handle as u64, globalSession: self })
		}
		else {
			Err(CreateSessionError::Generic)
		}
	}
}
impl Drop for GlobalSession {
	fn drop (&mut self) {
		slangjs_dropGlobalSession(self.0);
	}
}

/// A handle for a JavaScript-side `ComponentList` instance.
struct ComponentList(u64);
impl ComponentList
{
	pub fn new () -> Self { Self(
		slangjs_createComponentList()
	)}

	pub fn addModule (&self, module: &JsSlangModule<'_>) {
		slangjs_ComponentList_addModule(self.0, module.handle);
	}

	pub fn addEntryPoint (&self, entryPoint: &SlangEntryPoint<'_>) {
		slangjs_ComponentList_addEntryPoint(self.0, entryPoint.handle);
	}

	pub fn addComposite (&self, composite: &SlangComposite) {
		slangjs_ComponentList_addComposite(self.0, composite.handle);
	}
}
impl Drop for ComponentList {
	fn drop (&mut self) {
		slangjs_dropComponentList(self.0);
	}
}

/// A handle for a JavaScript-side `slang::Session` instance.
pub struct Session<'this> {
	handle: u64,

	#[expect(dead_code)]
	globalSession: &'this GlobalSession
}
impl Session<'_> {
	pub fn loadModuleFromSourceString (&self, virtualFilepath: impl AsRef<Path>, sourceCode: &str)
		-> Result<JsSlangModule<'_>, LoadModuleError>
	{
		// Make sure we get a valid target path
		let targetPath = validateModulePath(virtualFilepath.as_ref())?;

		// Compile via JavaScript bridge
		tracing::warn!("Session #{}: Compiling module `{targetPath}` via JavaScript bridge", self.handle);
		let moduleHandle = slangjs_Session_loadModuleFromSource(self.handle, targetPath, targetPath, sourceCode);
		if moduleHandle < 0 {
			return Err(LoadModuleError::CompilationError("Failed to compile module `{targetPath}`".into()))
		}

		// Return resulting module
		JsSlangModule::new(moduleHandle as u64, targetPath)
	}

	pub fn createComposite<'this> (&'this self, components: &[ComponentRef<'this>])
		-> Result<SlangComposite<'this>, compile::CreateCompositeError>
	{
		// Build JavaScript-side component list
		let componentList = ComponentList::new();
		for component in components {
			match component {
				compile::ComponentRef::Module(module) => componentList.addModule(module),
				compile::ComponentRef::EntryPoint(entryPoint) => componentList.addEntryPoint(entryPoint),
				compile::ComponentRef::Composite(composite) => componentList.addComposite(composite)
			}
		}

		// Composit via JavaScript bridge
		let compositeHandle = slangjs_Session_createComposite(self.handle, componentList.0);
		if compositeHandle < 0 {
			return Err(compile::CreateCompositeError::ImplementationSpecific(anyhow::anyhow!("Slang error")))
		}

		// Return resulting module
		Ok(SlangComposite::new(compositeHandle as u64))
	}
}
impl Drop for Session<'_> {
	fn drop (&mut self) {
		slangjs_GlobalSession_dropSession(self.handle);
	}
}

/// A handle for a JavaScript-side `slang::Module` instance.
pub struct JsSlangModule<'this> {
	handle: u64,
	virtualFilepath: PathBuf,
	entryPoints: Vec<SlangEntryPoint<'this>>,
	sessionPhantom: std::marker::PhantomData<&'this Session<'this>>
}
impl JsSlangModule<'_> {
	pub(crate) fn new (handle: u64, virtualFilepath: impl AsRef<Path>) -> Result<Self, LoadModuleError>
	{
		let entryPoints = slangjs_Module_getEntryPoints(handle).into_iter().map(
			|epHandle| SlangEntryPoint::new(epHandle)
		).collect();
		Ok(Self {
			handle, virtualFilepath: virtualFilepath.as_ref().to_owned(),
			entryPoints, sessionPhantom: std::marker::PhantomData
		})
	}

	#[inline(always)]
	pub fn enter (&self) -> ComponentRef<'_> {
		ComponentRef::Module(self)
	}
}
impl<'sess> compile::Module<SlangEntryPoint<'sess>> for JsSlangModule<'sess>
{
	fn virtualFilepath (&self) -> &Path {
		&self.virtualFilepath
	}

	fn entryPoint (&self, name: &str) -> Option<&SlangEntryPoint<'sess>> {
		self.entryPoints.iter().find(|ep| ep.name == name)
	}

	fn entryPoints (&self) -> &[SlangEntryPoint<'sess>] {
		&self.entryPoints
	}
}
impl compile::Component for JsSlangModule<'_> {
	type Id = u64;

	#[inline(always)]
	fn id (&self) -> Self::Id {
		self.handle
	}
}

/// A handle for a JavaScript-side `slang::EntryPoint` instance.
pub struct SlangEntryPoint<'m> {
	handle: u64,
	name: String,
	modulePhantom: std::marker::PhantomData<&'m JsSlangModule<'m>>
}
impl SlangEntryPoint<'_> {
	pub(crate) fn new (handle: u64) -> Self { Self {
		handle, name: slangjs_EntryPoint_name(handle), modulePhantom: std::marker::PhantomData
	}}

	#[inline(always)]
	pub fn enter (&self) -> ComponentRef<'_> {
		ComponentRef::EntryPoint(self)
	}
}
impl compile::EntryPoint for SlangEntryPoint<'_> {
	fn name (&self) -> &str {
		&self.name
	}
}
impl compile::Component for SlangEntryPoint<'_> {
	type Id = u64;

	#[inline(always)]
	fn id (&self) -> Self::Id {
		self.handle
	}
}

/// A handle for a JavaScript-side *Slang* *composite component* instance.
pub struct SlangComposite<'this> {
	handle: u64,
	sessionPhantom: std::marker::PhantomData<&'this Session<'this>>
}
impl SlangComposite<'_> {
	pub(crate) fn new (handle: u64) -> Self { Self {
		handle, sessionPhantom: std::marker::PhantomData
	}}

	#[inline(always)]
	pub	fn enter (&self) -> ComponentRef<'_> {
		ComponentRef::Composite(self)
	}
}
impl Drop for SlangComposite<'_> {
	fn drop (&mut self) {
		tracing::warn!("Dropping composite #{}",self.handle);
		slangjs_Session_dropComposite(self.handle);
	}
}
impl compile::Component for SlangComposite<'_> {
	type Id = u64;

	#[inline(always)]
	fn id (&self) -> Self::Id {
		self.handle
	}
}
impl compile::Composite for SlangComposite<'_> {}

/// A handle for a **linked** JavaScript-side *Slang* *composite component* instance.
pub struct SlangLinkedComposite<'this> {
	handle: u64,

	#[expect(dead_code)]
	session: &'this Session<'this>,
}
impl<'sess> SlangLinkedComposite<'sess> {
	pub(crate) fn new (handle: u64, session: &'sess Session) -> Self { Self {
		handle, session
	}}
}
impl Drop for SlangLinkedComposite<'_> {
	fn drop (&mut self) {
		tracing::warn!("Dropping linked composite #{}",self.handle);
		slangjs_Session_dropComposite(self.handle);
	}
}
impl compile::LinkedComposite for SlangLinkedComposite<'_> {
	fn allEntryPointsCode (&self, _target: compile::Target) -> Result<compile::ProgramCode, compile::TranslateError> {
		Err(compile::TranslateError::Backend(anyhow::anyhow!("Not yet implemented")))
	}

	fn entryPointCode (&self, _target: compile::Target, _entryPointIdx: usize)
		-> Option<Result<compile::ProgramCode, compile::TranslateError>>
	{
		None
	}
}


///
pub struct ContextBuilder<'ctx> {
	targets: util::ds::BTreeUniqueVec<compile::Target>,
	lifetimePhantom: std::marker::PhantomData<&'ctx ()>
}
impl ContextBuilder<'_>
{
	#[inline(always)]
	pub fn buildWithGlobalSession (self, globalSession: &GlobalSession)
		-> Result<Context<'_>, compile::CreateContextError>
	{
		let session = globalSession.createSession().map_err(
			|err| compile::CreateContextError::ImplementationDefined(err.into())
		)?;
		Ok(Context { session, compatHash: 123, environment: None })
	}
}
impl Default for ContextBuilder<'_> {
	fn default () -> Self { Self {
		targets: vec![compile::mostSuitableTarget()].into(), lifetimePhantom: Default::default()
	}}
}
impl<'ctx> compile::ContextBuilder for ContextBuilder<'ctx> {
	type Context = Context<'ctx>;

	#[inline(always)]
	fn defaultForPlatform (platform: &util::meta::SupportedPlatform) -> Self { Self {
		targets: vec![compile::mostSuitableTargetForPlatform(platform)].into(),
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
	fn build (self) -> Result<Context<'ctx>, compile::CreateContextError> {
		self.buildWithGlobalSession(&GLOBAL_SESSION)
	}
}


/// A *Slang* [compilation context](compile::Context) for `wasm32-unknown-unknown` targets that makes use of a *light*
/// JavaScript bridge. It is considered "light" because it only forwards the small number of high-level APIs defined by
/// the [*CGV-rs* shader compilation model](crate::compile), rather than translating the full *Slang* API. This reduces
/// function call overhead significantly, but also limits clients to the small and abstracted subset of functionality
/// exposed by the `Context`.
pub struct Context<'this> {
	session: Session<'this>,
	compatHash: u64,
	environment: Option<compile::Environment<EnvModule>>
}
impl Context<'_>
{
	/// Helper for obtaining a fresh *Slang* session.
	fn freshSession (globalSession: &GlobalSession) -> Result<Session<'_>, CreateSessionError> {
		globalSession.createSession()
	}
}
impl<'ctx> compile::Context for Context<'ctx>
{
	type ModuleType<'module> = JsSlangModule<'module> where Self: 'module;
	type EntryPointType<'ep> = SlangEntryPoint<'ep> where Self: 'ep;
	type CompositeType<'ct> = SlangComposite<'ct> ;
	type LinkedCompositeType<'lct> = SlangLinkedComposite<'lct> where Self: 'lct;
	type Builder = ContextBuilder<'ctx>;

	#[inline]
	fn compileFromSource (&self, sourceCode: &str) -> Result<JsSlangModule<'_>, LoadModuleError> {
		let targetPath = PathBuf::from(format!("_unnamed__{}.slang", util::unique::uint32()));
		self.compileFromNamedSource(&targetPath, sourceCode)
	}

	fn compileFromNamedSource (&self, targetPath: impl AsRef<Path>, sourceCode: &str)
		-> Result<JsSlangModule<'_>, LoadModuleError>
	{
		// Make sure we get a valid target path
		let targetPath = validateModulePath(targetPath.as_ref())?;

		// Let slang compile the module
		let module =  self.session.loadModuleFromSourceString(targetPath, sourceCode)?;

		// Done!
		Ok(module)
	}

	fn createComposite<'this> (&'this self, components: &[ComponentRef<'this>])
		-> Result<SlangComposite<'this>, compile::CreateCompositeError>
	{
		self.session.createComposite(components)
	}

	fn linkComposite<'this> (&'this self, composite: &SlangComposite) -> Result<SlangLinkedComposite<'this>, LinkError>
	{
		let handle = slangjs_Composite_link(composite.handle);
		if handle > 0 {
			Ok(SlangLinkedComposite::new(handle as u64, &self.session))
		}
		else {
			Err(LinkError::ImplementationSpecific(anyhow::anyhow!("Slang link error")))
		}
	}
}
impl compile::EnvironmentEnabled for Context<'_>
{
	type ModuleType = EnvModule;
	type EnvStorageHint = EnvironmentStorage;

	fn loadModule (&mut self, _filename: impl AsRef<Path>) -> Result<(), LoadModuleError> {
		// File loading is not currently supported by WASM Slang
		unimplemented!("Compiling from source files is not currently supported by WASM Slang");
	}

	fn loadModuleFromSource (
		&mut self, envStorage: EnvironmentStorage, targetPath: impl AsRef<Path>, sourceCode: &str
	) -> Result<(), LoadModuleError>
	{
		// Compile the source code inside the Slang session
		use compile::Context;
		self.compileFromNamedSource(&targetPath, sourceCode)?;
		let module = match envStorage {
			EnvironmentStorage::SourceCode => EnvModule::fromSlangSourceCode(sourceCode),
			EnvironmentStorage::IR => unimplemented!(
				"IR bytecode modules are not currently supported by WASM Slang"
			)
		};

		// Store the module in the environment
		storeInEnvironment(self.environment.as_mut(), targetPath, module).map_err(|err| match err {
			AddModuleError::DuplicateModulePaths(path) => LoadModuleError::DuplicatePath(path)
		})
	}

	fn loadModuleFromIR (&mut self, targetPath: impl AsRef<Path>, _bytes: &[u8]) -> Result<(), LoadModuleError>
	{
		// Make sure we get a valid target path
		let _targetPath_str = validateModulePath(targetPath.as_ref())?;

		// IR bytecode loading is not currently supported by WASM Slang
		unimplemented!("IR bytecode loading is not currently supported by WASM Slang");
	}

	fn replaceEnvironment (&mut self, environment: Option<compile::Environment<EnvModule>>)
		-> Result<Option<compile::Environment<EnvModule>>, compile::SetEnvironmentError>
	{
		// Check if the new environment is compatible (in case it's `Some`)
		/*if let Some(newEnv) = &environment && self.compatHash != newEnv.compatHash() {
			return Err(compile::SetEnvironmentError::IncompatibleEnvironment)
		}*/

		// Start from a fresh session
		let newSession = Self::freshSession(&GLOBAL_SESSION).expect(
			"Creating a Slang session identical to an existing one should never fail unless there are \
			unrecoverable external circumstances (out-of-memory etc.)"
		);

		// Apply the new environment to the new session
		if let Some(newEnv) = &environment
		{
			for module in newEnv.modules()
			{
				match &module.module
				{
					EnvModule::SourceCode(sourceCode) =>
						newSession.loadModuleFromSourceString(&module.path, sourceCode).or_else(|err| Err(
							SetEnvironmentError::ImplementationSpecific(err.into())
						))?,

					EnvModule::IR(_) =>
						unimplemented!("IR bytecode loading is not currently supported by WASM Slang")
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

/// Obtain a reference to a `'static` [`slang::GlobalSession`](GlobalSession) from which actual, stateful compiler
/// sessions can be created. *CGV-rs* uses this global session for all its internal shader compilation tasks.
#[inline(always)]
pub fn obtainGlobalSession () -> &'static GlobalSession {
	&GLOBAL_SESSION
}

/// API prototypes of the JavaScript bridge.
#[wasm_bindgen]
extern "C" {
	fn slangjs_createGlobalSession () -> i64;
	fn slangjs_dropGlobalSession (handle: u64);

	fn slangjs_GlobalSession_createSession (globalSessionHandle: u64) -> i64;
	fn slangjs_GlobalSession_dropSession (handle: u64);

	fn slangjs_createComponentList () -> u64;
	fn slangjs_dropComponentList (handle: u64);

	fn slangjs_ComponentList_addModule (componentListHandle: u64, handle: u64);
	fn slangjs_ComponentList_addEntryPoint (componentListHandle: u64, handle: u64);
	fn slangjs_ComponentList_addComposite (componentListHandle: u64, handle: u64);

	fn slangjs_Session_loadModuleFromSource (
		sessionHandle: u64, moduleName: &str, modulePath: &str, moduleSourceCode: &str
	) -> i64;
	fn slangjs_Session_createComposite (sessionHandle: u64, componentListHandle: u64) -> i64;
	fn slangjs_Session_dropComposite (handle: u64);

	fn slangjs_Module_getEntryPoints (moduleHandle: u64) -> Vec<u64>;

	fn slangjs_EntryPoint_name (entryPointHandle: u64) -> String;

	fn slangjs_Composite_link (handle: u64) -> i64;
}
