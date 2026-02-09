
//////
//
// Imports
//

// Standard library
use std::{collections::BTreeMap, path::{PathBuf, Path}, sync::LazyLock, marker::PhantomData};

// Wasm-bindgen library
use wasm_bindgen::prelude::*;

// Anyhow library
use anyhow::anyhow;

// Local imports
use crate::{compile::{self, ComponentRef}, slang::{*, context::*}};


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

/// Our `ActiveTargetsMap` specialization using`u32` as that is what out JavaScript bridge uses.
type ActiveTargetsMap = GenericActiveTargetsMap<u32>;


/// A handle for a JavaScript-side `slang::GlobalSession` instance.
pub struct GlobalSession(u64);
impl GlobalSession
{
	pub fn new () -> Option<Self> {
		let handle = slangjs_createGlobalSession();
		if handle > 0 {
			Some(Self(handle as u64))
		}
		else { None }
	}

	fn createSession (&self, sessionConfig: &SessionConfig) -> Result<Session<'_>, CreateSessionError>
	{
		let handle = slangjs_GlobalSession_createSession(self.0);
		if handle > 0 {
			let mut activeTargetsMap = ActiveTargetsMap::default();
			sessionConfig.targets.iter().enumerate().for_each(
				|(idx, target)| activeTargetsMap[target.slot()] = Some(idx as u32)
			);
			Ok(Session { handle: handle as u64, activeTargetsMap, gsPhantom: Default::default() })
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

	pub fn addModule (&self, module: &Module<'_>) {
		slangjs_ComponentList_addModule(self.0, module.handle);
	}

	pub fn addEntryPoint (&self, entryPoint: &EntryPoint<'_>) {
		slangjs_ComponentList_addEntryPoint(self.0, entryPoint.handle);
	}

	pub fn addComposite (&self, composite: &Composite) {
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
	activeTargetsMap: ActiveTargetsMap,
	gsPhantom: PhantomData<&'this GlobalSession>
}
impl Session<'_>
{
	pub fn loadModuleFromSourceString (&self, virtualFilepath: impl AsRef<Path>, sourceCode: &str)
		-> Result<Module<'_>, compile::LoadModuleError>
	{
		// Make sure we get a valid target path
		let targetPath = validateModulePath(virtualFilepath.as_ref())?;

		// Compile via JavaScript bridge
		tracing::warn!("Session #{}: Compiling module `{targetPath}` via JavaScript bridge", self.handle);
		let moduleHandle = slangjs_Session_loadModuleFromSource(self.handle, targetPath, targetPath, sourceCode);
		if moduleHandle < 0 {
			return Err(compile::LoadModuleError::CompilationError(anyhow!("Failed to compile module `{targetPath}`")))
		}

		// Return resulting module
		Module::new(moduleHandle as u64, targetPath)
	}

	fn createComposite (&self, componentList: &ComponentList) -> Result<Composite<'_>, compile::CreateCompositeError>
	{
		// Composit via JavaScript bridge
		let compositeHandle = slangjs_Session_createComposite(self.handle, componentList.0);
		if compositeHandle < 0 {
			return Err(compile::CreateCompositeError::Backend(anyhow!("Slang error")))
		}

		// Return resulting module
		Ok(Composite::new(compositeHandle as u64))
	}
}
impl Drop for Session<'_> {
	fn drop (&mut self) {
		slangjs_Session_drop(self.handle);
	}
}

/// A handle for a JavaScript-side `slang::Module` instance.
pub struct Module<'this> {
	handle: u64,
	virtualFilepath: PathBuf,
	entryPoints: Vec<EntryPoint<'this>>,
	sessionPhantom: PhantomData<&'this Session<'this>>
}
impl Module<'_> {
	pub(crate) fn new (handle: u64, virtualFilepath: impl AsRef<Path>) -> Result<Self, compile::LoadModuleError>
	{
		let entryPoints = slangjs_Module_getEntryPoints(handle).into_iter().map(
			|epHandle| EntryPoint::new(epHandle)
		).collect();
		Ok(Self {
			handle, virtualFilepath: virtualFilepath.as_ref().to_owned(),
			entryPoints, sessionPhantom: PhantomData
		})
	}
}
impl Drop for Module<'_> {
	fn drop (&mut self) {
		slangjs_Module_drop(self.handle);
	}
}
impl<'this> compile::Module<EntryPoint<'this>> for Module<'this>
{
	fn virtualFilepath (&self) -> &Path {
		&self.virtualFilepath
	}

	fn entryPoint (&self, name: &str) -> Option<&EntryPoint<'this>> {
		self.entryPoints.iter().find(|ep| ep.name == name)
	}

	fn entryPoints (&self) -> &[EntryPoint<'this>] {
		&self.entryPoints
	}
}
impl compile::Component for Module<'_> {
	type Id = u64;

	#[inline(always)]
	fn id (&self) -> Self::Id {
		self.handle
	}
}

/// A handle for a JavaScript-side `slang::EntryPoint` instance.
pub struct EntryPoint<'this> {
	handle: u64,
	name: String,
	modulePhantom: PhantomData<&'this Module<'this>>
}
impl EntryPoint<'_> {
	pub(crate) fn new (handle: u64) -> Self { Self {
		handle, name: slangjs_EntryPoint_name(handle), modulePhantom: PhantomData::default()
	}}
}
impl compile::EntryPoint for EntryPoint<'_> {
	fn name (&self) -> &str {
		&self.name
	}
}
impl compile::Component for EntryPoint<'_> {
	type Id = u64;

	#[inline(always)]
	fn id (&self) -> Self::Id {
		self.handle
	}
}

/// A handle for a JavaScript-side *Slang* *composite component* instance.
pub struct Composite<'this> {
	handle: u64,
	sessionPhantom: PhantomData<&'this Session<'this>>
}
impl Composite<'_> {
	pub(crate) fn new (handle: u64) -> Self { Self {
		handle, sessionPhantom: PhantomData
	}}
}
impl Drop for Composite<'_> {
	fn drop (&mut self) {
		tracing::warn!("Dropping composite #{}",self.handle);
		slangjs_Session_dropComposite(self.handle);
	}
}
impl compile::Component for Composite<'_> {
	type Id = u64;

	#[inline(always)]
	fn id (&self) -> Self::Id {
		self.handle
	}
}
impl compile::Composite for Composite<'_> {}

/// A handle for a **linked** JavaScript-side *Slang* *composite component* instance.
pub struct LinkedComposite<'this> {
	handle: u64,
	entryPointsMap: BTreeMap<String, u32>,
	entryPointNames: Vec<&'this str>,
	activeTargetsMap: &'this ActiveTargetsMap,
	sessionPhantom: PhantomData<&'this Session<'this>>
}
impl LinkedComposite<'_> {
	#[inline]
	fn checkTranslationResult (code: &[u8], target: &compile::Target) -> Result<(), compile::TranslateError>
	{
		if code.len()>1 || (code.len()==1 && code[0]!=0xff) {
			Ok(())
		}
		else {
			Err(compile::TranslateError::Backend(anyhow!("translation to {target} failed")))
		}
	}
}
impl Drop for LinkedComposite<'_> {
	fn drop (&mut self) {
		tracing::warn!("Dropping linked composite #{}",self.handle);
		slangjs_Session_dropComposite(self.handle);
	}
}
impl compile::LinkedComposite for LinkedComposite<'_>
{
	#[inline(always)]
	fn numEntryPoints (&self) -> usize {
		self.entryPointsMap.len()
	}

	#[inline]
	fn entryPointIdx (&self, name: &str) -> usize {
		*self.entryPointsMap.get(name).expect(&format!("entry point '{name}' not found")) as usize
	}

	fn entryPointName (&self, entryPointIdx: usize) -> &str {
		self.entryPointNames[entryPointIdx]
	}

	fn allEntryPointsCode (&self, target: compile::Target) -> Result<compile::ProgramCode, compile::TranslateError>
	{
		if let Some(targetIdx) = self.activeTargetsMap[target.slot()]
		{
			// Translate via JavaScript bridge
			let code = slangjs_Composite_targetCode(self.handle, targetIdx);
			Self::checkTranslationResult(&code, &target)?;

			// Done!
			Ok(if target.isText() {
				compile::ProgramCode::Text(
					String::from_utf8(code).expect(&format!("{target} targets should be UTF-8-encoded"))
				)
			} else {
				compile::ProgramCode::Binary(code)
			})
		}
		else {
			Err(compile::TranslateError::InvalidTarget(target))
		}
	}

	fn entryPointCode (&self, target: compile::Target, entryPointIdx: usize)
		-> Option<Result<compile::ProgramCode, compile::TranslateError>>
	{
		// Check for entry points index out-of-bounds
		if entryPointIdx >= self.entryPointsMap.len() {
			return None;
		}

		// Only proceed if target is active
		if let Some(targetIdx) = self.activeTargetsMap[target.slot()]
		{
			// Translate to target
			let code = slangjs_Composite_entryPointCode(
				self.handle, entryPointIdx as u32, targetIdx
			);

			// Make sense of the translation result
			let translationResult = Self::checkTranslationResult(&code, &target);
			if translationResult.is_err() {
				return Some(Err(translationResult.err().unwrap()));
			};

			// Done!
			if target.isText() {
				let textCode = unsafe {
					// SAFETY: We strongly believe in Slang's commitment to always returning valid UTF-8
					String::from_utf8_unchecked(code)
				};
				Some(Ok(compile::ProgramCode::Text(textCode)))
			} else {
				Some(Ok(compile::ProgramCode::Binary(code)))
			}
		}
		else {
			Some(Err(compile::TranslateError::InvalidTarget(target)))
		}
	}
}


/// Helper struct storing session configuration info to facilitate both [quick rebuilds](Context::freshSession) as well
/// as [`compile::Environment`] compatibility checking.
#[derive(Clone)]
struct SessionConfig {
	targets: util::ds::BTreeUniqueVec<compile::Target>
}


///
pub struct ContextBuilder<'ctx> {
	targets: util::ds::BTreeUniqueVec<compile::Target>,
	lifetimePhantom: PhantomData<&'ctx ()>
}
impl ContextBuilder<'_>
{
	///
	#[inline(always)]
	pub fn buildWithGlobalSession (self, globalSession: &GlobalSession)
		-> Result<Context<'_>, compile::CreateContextError>
	{
		// Until we can do further testing, we will only support the WGSL target
		for target in self.targets.iter() {
			if !target.isWGSL() {
				return Err(compile::CreateContextError::UnsupportedTarget(*target));
			}
		}

		// Populate reusable session config
		let sessionConfig = SessionConfig { targets: self.targets };

		// Create the stateful Slang compiler session
		let session = Context::freshSession(&globalSession, &sessionConfig).map_err(
			|err| compile::CreateContextError::Backend(err.into())
		)?;
		Ok(Context { session, sessionConfig, compatHash: 0, environment: None })
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
	fn withPlatformDefaults (platform: &util::meta::SupportedPlatform) -> Self { Self {
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
		globalSession.createSession(sessionConfig)
	}
}
impl<'this> compile::Context for Context<'this>
{
	type ModuleType<'module> = Module<'module> where Self: 'module;
	type EntryPointType<'ep> = EntryPoint<'ep> where Self: 'ep;
	type CompositeType<'ct> = Composite<'ct> where Self: 'ct;
	type LinkedCompositeType<'lct> = LinkedComposite<'lct> where Self: 'lct;
	type Builder = ContextBuilder<'this>;

	fn supportsTarget (&self, target: compile::Target) -> bool {
		self.session.activeTargetsMap[target.slot()].is_some()
	}

	#[inline]
	fn compileFromSource (&self, sourceCode: &str) -> Result<Module<'_>, compile::LoadModuleError> {
		let targetPath = uniqueAnonymousSlangFilename();
		self.compileFromNamedSource(&targetPath, sourceCode)
	}

	fn compileFromNamedSource (&self, targetPath: impl AsRef<Path>, sourceCode: &str)
		-> Result<Module<'_>, compile::LoadModuleError>
	{
		// Make sure we get a valid target path
		let targetPath = validateModulePath(targetPath.as_ref())?;

		// Let slang compile the module
		let module =  self.session.loadModuleFromSourceString(targetPath, sourceCode)?;

		// Done!
		Ok(module)
	}

	fn createComposite<'outer, 'ctx> (&'ctx self, components: &'outer [ComponentRef<'outer, 'ctx, Self>])
		-> Result<Self::CompositeType<'ctx>, compile::CreateCompositeError>
	{
		// Build JavaScript-side component list
		let componentList = ComponentList::new();
		for component in components {
			match component {
				ComponentRef::Module(module) => componentList.addModule(module),
				ComponentRef::EntryPoint(entryPoint) => componentList.addEntryPoint(entryPoint),
				ComponentRef::Composite(composite) => componentList.addComposite(composite)
			}
		}

		// Create the composite
		self.session.createComposite(&componentList)
	}

	fn linkComposite (&self, composite: &Composite) -> Result<LinkedComposite<'_>, compile::LinkError>
	{
		// Link
		let handle = slangjs_Composite_link(composite.handle);
		if handle < 0 {
			return Err(compile::LinkError::Backend(anyhow!("Slang link error")));
		}

		// Build entry point name->index map
		let epNames_owned = slangjs_Composite_orderedEntryPointNames(handle as u64);
		let mut entryPointsMap = BTreeMap::default();
		let mut entryPointNames = Vec::with_capacity(epNames_owned.len());
		for (idx, epName) in epNames_owned.into_iter().enumerate() {
			entryPointNames.push(unsafe {
				// SAFETY:
				// Strings always live on the heap in Rust, so the address of their contents will be stable even when
				// moved. The lifetime of the reference we create to the string slice is the same as `name` as per our
				// struct definition, so we will not cause dangling references. Finally, `LinkedComposite`s are
				// immutable once created, so no aliasing rule will ever be violated by disconnecting the reference from
				// the owning `name` string object.
				&*(epName.as_str() as *const str)
			});
			entryPointsMap.insert(epName, idx as u32);
		}

		Ok(LinkedComposite {
			handle: handle as u64,
			entryPointsMap, entryPointNames, activeTargetsMap: &self.session.activeTargetsMap,
			sessionPhantom: Default::default()
		})
	}
}
impl compile::EnvironmentEnabled for Context<'_>
{
	type ModuleType = EnvModule;
	type EnvStorageHint = EnvironmentStorage;

	#[inline(always)]
	fn loadModule (&mut self, _: impl AsRef<Path>) -> Result<(), compile::LoadModuleError>
	where Self: compile::HasFileSystemAccess
	{
		// File loading is not currently supported by WASM Slang
		unsafe {
			// SAFETY: We don't implement `compile::HasFileSystemAccess`, so it is impossible to call this method.
			std::hint::unreachable_unchecked();
		}
	}

	fn loadModuleFromSource (&mut self, envStorage: EnvironmentStorage, targetPath: impl AsRef<Path>, sourceCode: &str)
		-> Result<(), compile::LoadModuleError>
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
			AddModuleError::DuplicateModulePaths(path) => compile::LoadModuleError::DuplicatePath(path)
		})
	}

	fn loadModuleFromIR (&mut self, targetPath: impl AsRef<Path>, _bytes: &[u8]) -> Result<(), compile::LoadModuleError>
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
		let newSession = Self::freshSession(&GLOBAL_SESSION, &self.sessionConfig).expect(
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
							compile::SetEnvironmentError::Backend(err.into())
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

	fn slangjs_createComponentList () -> u64;
	fn slangjs_dropComponentList (handle: u64);

	fn slangjs_ComponentList_addModule (componentListHandle: u64, handle: u64);
	fn slangjs_ComponentList_addEntryPoint (componentListHandle: u64, handle: u64);
	fn slangjs_ComponentList_addComposite (componentListHandle: u64, handle: u64);

	fn slangjs_Session_drop (handle: u64);
	fn slangjs_Session_loadModuleFromSource (
		sessionHandle: u64, moduleName: &str, modulePath: &str, moduleSourceCode: &str
	) -> i64;
	fn slangjs_Session_createComposite (sessionHandle: u64, componentListHandle: u64) -> i64;
	fn slangjs_Session_dropComposite (handle: u64);

	fn slangjs_Module_drop (handle: u64);

	fn slangjs_Module_getEntryPoints (moduleHandle: u64) -> Vec<u64>;

	fn slangjs_EntryPoint_name (entryPointHandle: u64) -> String;

	fn slangjs_Composite_orderedEntryPointNames (handle: u64) -> Vec<String>;
	fn slangjs_Composite_link (handle: u64) -> i64;
	fn slangjs_Composite_targetCode (handle: u64, targetIdx: u32) -> Vec<u8>;
	fn slangjs_Composite_entryPointCode (handle: u64, targetIdx: u32, entryPointIdx: u32) -> Vec<u8>;
}
