
//////
//
// Imports
//

// Standard library
use std::{path::{PathBuf, Path}, collections::BTreeSet, sync::LazyLock, cell::RefCell};
use shader_slang::SourceLanguage::Slang;
// Wasm-bindgen library
use wasm_bindgen::prelude::*;

// Local imports
use crate::*;
use crate::{compile::{SetEnvironmentError, AddModuleError}, /*slang::Program, */slang::context::*};



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
type ComponentRef<'sess, 'gs> = compile::ComponentRef<
	'sess, JsSlangModule<'sess, 'gs>, SlangEntryPoint<'sess, 'gs>, SlangComposite<'sess, 'gs>
>;

/// The mutable state of a [`GlobalSession`] instance.
struct GlobalSessionState {
	pub(crate) handle: u64,
	pub(crate) sessions: BTreeSet<u64>
}
/// A handle for a JavaScript-side `slang::GlobalSession` instance.
pub struct GlobalSession{
	state: RefCell<GlobalSessionState>
}
impl GlobalSession {
	pub(crate) fn new () -> Option<Self> {
		let handle = slangjs_createGlobalSession();
		if handle > 0 {
			Some(Self {
				state: RefCell::new(GlobalSessionState{ handle: handle as u64, sessions: BTreeSet::new() })
			})
		}
		else          { None }
	}

	pub(crate) fn dropSession (&self, handle: u64) {
		let mut state = self.state.borrow_mut();
		slangjs_GlobalSession_dropSession(handle);
		state.sessions.remove(&handle);
	}

	pub fn createSession (&self) -> Result<Session<'_, '_>, CreateSessionError>
	{
		let mut state = self.state.borrow_mut();
		let handle = slangjs_GlobalSession_createSession(state.handle);
		if handle > 0 {
			state.sessions.insert(handle as u64);
			Ok(Session { handle: handle as u64, globalSession: self, sessionPhantom: std::marker::PhantomData })
		}
		else {
			Err(CreateSessionError::Generic)
		}
	}
}
impl Drop for GlobalSession {
	fn drop (&mut self)
	{
		tracing::warn!("Dropping global session #{}", self.state.borrow().handle);
		/* debug info */ {
			let state = self.state.borrow();
			if !state.sessions.is_empty() {
				tracing::warn!("Dropping global session #{} with active child sessions", state.handle);
			}
		}
		while let Some(handle) = self.state.borrow().sessions.iter().next().map(|&h| h) {
			self.dropSession(handle);
		}
		let state = self.state.borrow();
		if !state.sessions.is_empty() {
			let msg = format!(
				"INTERNAL LOGIC ERROR: dropped global session #{} still contains child sessions", state.handle
			);
			tracing::error!("{msg}");
			panic!("{msg}");
		}
		slangjs_dropGlobalSession(state.handle);
	}
}
unsafe // SAFETY: `GlobalSession` stores all its state in a RefCell, which prevents concurrent mutable access.
impl Sync for GlobalSession {}

/// A handle for a JavaScript-side `ComponentList` instance.
struct ComponentList(u64);
impl ComponentList
{
	pub fn new () -> Self { Self(
		slangjs_createComponentList()
	)}

	pub fn addModule (&self, module: &JsSlangModule<'_, '_>) {
		slangjs_ComponentList_addModule(self.0, module.handle);
	}

	pub fn addEntryPoint (&self, entryPoint: &SlangEntryPoint<'_, '_>) {
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
pub struct Session<'sess, 'gs> {
	pub(crate) globalSession: &'gs GlobalSession,
	handle: u64,
	sessionPhantom: std::marker::PhantomData<&'sess Session<'sess, 'gs>>
}
impl<'sess, 'gs> Session<'sess, 'gs> {
	pub fn loadModuleFromSourceString (&self, virtualFilepath: &str, sourceCode: &str)
		-> Result<JsSlangModule<'_, '_>, LoadModuleError>
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
		JsSlangModule::new(moduleHandle as u64)
	}

	pub fn createComposite (&self, components: &[ComponentRef<'sess, 'gs>])
		-> Result<SlangComposite<'_, '_>, compile::CreateCompositeError>
	{
		/// Build JavaScript-side component list
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
impl Drop for Session<'_, '_> {
	fn drop (&mut self) {
		self.globalSession.dropSession(self.handle);
	}
}

/// A handle for a JavaScript-side `slang::Module` instance.
pub struct JsSlangModule<'sess, 'gs> {
	handle: u64,
	entryPoints: Vec<SlangEntryPoint<'sess, 'gs>>,
	sessionPhantom: std::marker::PhantomData<&'sess Session<'sess, 'gs>>
}
impl JsSlangModule<'_, '_> {
	pub(crate) fn new (handle: u64) -> Result<Self, LoadModuleError>
	{
		let entryPoints = slangjs_Module_getEntryPoints(handle).into_iter().map(
			|epHandle| SlangEntryPoint::new(epHandle)
		).collect();
		Ok(Self { handle, entryPoints, sessionPhantom: std::marker::PhantomData })
	}

	#[inline(always)]
	pub fn enter(&self) -> ComponentRef<'_, '_> {
		ComponentRef::Module(self)
	}
}
impl<'sess, 'gs> compile::Module<SlangEntryPoint<'sess, 'gs>> for JsSlangModule<'sess, 'gs> {
	fn entryPoints (&self) -> &[SlangEntryPoint<'sess, 'gs>] {
		&self.entryPoints
	}
}
impl compile::Component for JsSlangModule<'_, '_> {
	fn handle (&self) -> util::Handle {
		self.handle.into()
	}
}

/// A handle for a JavaScript-side `slang::EntryPoint` instance.
pub struct SlangEntryPoint<'sess, 'gs> {
	handle: u64,
	name: String,
	modulePhantom: std::marker::PhantomData<&'sess JsSlangModule<'sess, 'gs>>
}
impl SlangEntryPoint<'_, '_> {
	pub(crate) fn new (handle: u64) -> Self { Self {
		handle, name: slangjs_EntryPoint_name(handle), modulePhantom: std::marker::PhantomData
	}}

	#[inline(always)]
	pub fn enter(&self) -> ComponentRef<'_, '_> {
		ComponentRef::EntryPoint(self)
	}
}
impl compile::EntryPoint for SlangEntryPoint<'_, '_> {
	fn name (&self) -> &str {
		&self.name
	}
}
impl compile::Component for SlangEntryPoint<'_, '_> {
	fn handle (&self) -> util::Handle {
		self.handle.into()
	}
}

/// A handle for a JavaScript-side *Slang* *composite component* instance.
pub struct SlangComposite<'sess, 'gs> {
	handle: u64,
	sessionPhantom: std::marker::PhantomData<&'sess Session<'sess, 'gs>>
}
impl SlangComposite<'_, '_> {
	pub(crate) fn new (handle: u64) -> Self { Self {
		handle, sessionPhantom: std::marker::PhantomData
	}}

	#[inline(always)]
	pub	fn enter(&self) -> ComponentRef<'_, '_> {
		ComponentRef::Composite(self)
	}
}
impl Drop for SlangComposite<'_, '_> {
	fn drop (&mut self) {
		tracing::warn!("Dropping composite #{}", self.handle);
		slangjs_Session_dropComposite(self.handle);
	}
}
impl compile::Component for SlangComposite<'_, '_> {
	fn handle (&self) -> util::Handle {
		self.handle.into()
	}
}
impl compile::Composite for SlangComposite<'_, '_> {}


/// A *Slang* [compilation context](compile::Context) for `wasm32-unknown-unknown` targets that makes use of a *light*
/// JavaScript bridge. It is considered "light" because it only forwards the small number of high-level APIs that the
/// `Context` implements, rather than translating the full JavaScript *Slang* API. This reduces function call overhead
/// significantly, but also limits clients to the small and abstracted subset of functionality exposed by the `Context`.
pub struct Context<'this> {
	session: Session<'this, 'this>,
	compatHash: u64,
	environment: Option<compile::Environment<Module>>
}
impl<'this> Context<'this>
{
	/// Helper for obtaining a fresh *Slang* session.
	fn freshSession (globalSession: &GlobalSession) -> Result<Session<'_, '_>, CreateSessionError> {
		globalSession.createSession()
	}

	/// Create a new *Slang* context for the given compilation target using the given module search path.
	///
	/// # Arguments
	///
	/// * `target` – The target representation this `Context` will compile/transpile to.
	/// * `searchPath` – The module search path for the *Slang* compiler.
	pub fn forTarget (target: CompilationTarget, _searchPath: &[impl AsRef<Path>]) -> anyhow::Result<Self>
	{
		if target.isWGSL()
		{
			let session = Self::freshSession(&GLOBAL_SESSION).map_err(
				|_| anyhow::anyhow!("Failed to create Slang session")
			)?;

			Ok(Self { session, compatHash: 123, environment: None })
		}
		else {
			Err(anyhow::anyhow!("Unsupported compilation target: {target}"))
		}
	}

	/// Create a new *Slang* context for the *WGSL* target with the given module search path.
	///
	/// # Arguments
	///
	/// * `searchPath` – The module search path for the *Slang* compiler.
	pub fn new (_searchPath: &[impl AsRef<Path>]) -> anyhow::Result<Self> {
		Self::forTarget(CompilationTarget::WGSL, _searchPath)
	}

	/*/// Build a shader program from the given *Slang* source file.
	///
	/// # Arguments
	///
	/// * `sourceFile` – The `.slang` file containing the shader source code.
	pub fn buildProgram (&self, sourceFile: impl AsRef<Path>) -> anyhow::Result<Program> {
		Program::fromSource(self, sourceFile)
	}*/

	///
	#[inline]
	pub fn compileFromSource (&self, sourceCode: &str) -> Result<JsSlangModule<'_, '_>, LoadModuleError> {
		let targetPath = PathBuf::from(format!("_unnamed__{}.slang", util::unique::uint32()));
		self.compileFromNamedSource(&targetPath, sourceCode)
	}

	///
	pub fn compileFromNamedSource (&self, targetPath: impl AsRef<Path>, sourceCode: &str)
		-> Result<JsSlangModule<'_, '_>, LoadModuleError>
	{
		// Make sure we get a valid target path
		let targetPath = validateModulePath(targetPath.as_ref())?;

		// Let slang compile the module
		let module =  self.session.loadModuleFromSourceString(targetPath, sourceCode)?;

		// Done!
		Ok(module)
	}

	///
	pub fn loadModule (&mut self, _filename: impl AsRef<Path>) -> Result<(), LoadModuleError> {
		// File loading is not currently supported by WASM Slang
		unimplemented!("Compiling from source files is not currently supported by WASM Slang");
	}

	///
	pub fn loadModuleFromSource (
		&mut self, envStorage: EnvironmentStorage, targetPath: impl AsRef<Path>, sourceCode: &str
	) -> Result<(), LoadModuleError>
	{
		// Compile the source code inside the Slang session
		self.compileFromNamedSource(&targetPath, sourceCode)?;
		let module = match envStorage {
			EnvironmentStorage::SourceCode => Module::fromSlangSourceCode(sourceCode),
			EnvironmentStorage::IR => unimplemented!(
				"IR bytecode modules are not currently supported by WASM Slang"
			)
		};

		// Store the module in the environment
		storeInEnvironment(self.environment.as_mut(), targetPath, module).map_err(|err| match err {
			AddModuleError::DuplicateModulePaths(path) => LoadModuleError::DuplicatePath(path)
		})
	}

	///
	pub fn loadModuleFromIR (&mut self, targetPath: impl AsRef<Path>, _bytes: &[u8]) -> Result<(), LoadModuleError>
	{
		// Make sure we get a valid target path
		let _targetPath_str = validateModulePath(targetPath.as_ref())?;

		// IR bytecode loading is not currently supported by WASM Slang
		unimplemented!("IR bytecode loading is not currently supported by WASM Slang");
	}
}
impl<'this> compile::Context<
	'this, JsSlangModule<'this, 'this>, SlangEntryPoint<'this, 'this>, SlangComposite<'this, 'this>
> for Context<'this>
{
	fn createComposite (&'this self, components: &[ComponentRef<'this, 'this>])
	-> Result<SlangComposite<'this, 'this>, compile::CreateCompositeError> {
		self.session.createComposite(components)
	}
}
impl compile::EnvironmentEnabled<Module> for Context<'_>
{
	fn replaceEnvironment (&mut self, environment: Option<compile::Environment<Module>>)
		-> Result<Option<compile::Environment<Module>>, compile::SetEnvironmentError>
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
				let path = encodeValidModulePath(&module.path);
				match &module.module
				{
					Module::SourceCode(sourceCode) =>
						newSession.loadModuleFromSourceString(&path, sourceCode).or_else(|err| Err(
							SetEnvironmentError::ImplementationSpecific(err.into())
						))?,

					Module::IR(_) =>
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
}
