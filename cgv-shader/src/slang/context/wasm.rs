
//////
//
// Imports
//

// Wasm-bindgen library
use wasm_bindgen::prelude::*;

// Local imports
use crate::*;
use crate::{compile::{SetEnvironmentError, AddModuleError}, /*slang::Program, */slang::context::*};



//////
//
// Structs
//

/// A handle for a JavaScript-side `slang::Session` instance.
struct Session(i64);
impl Session {
	pub(crate) fn new () -> Option<Self> {
		let handle = slangjs_createSession();
		if handle > 0 { Some(Self(handle)) }
		else          { None }
	}

	pub fn loadModuleFromSourceString (&self, virtualFilepath: &str, sourceCode: &str)
		-> Result<JsSlangModule, LoadModuleError>
	{
		// Make sure we get a valid target path
		let targetPath = validateModulePath(virtualFilepath.as_ref())?;

		// Compile via JavaScript bridge
		tracing::warn!("Session #{}: Compiling module `{targetPath}` via JavaScript bridge", self.0);
		let moduleHandle = slangjs_loadModuleFromSource(self.0 as u64, targetPath, targetPath, sourceCode);
		if moduleHandle < 0 {
			return Err(LoadModuleError::CompilationError("Failed to compile module `{targetPath}`".into()))
		}

		// Return resulting module
		Ok(JsSlangModule(moduleHandle))
	}
}
impl Drop for Session {
	fn drop (&mut self) {
		slangjs_dropSession(self.0 as u64);
		self.0 = -1;
	}
}

/// A handle for a JavaScript-side `slang::Module` instance.
struct JsSlangModule(i64);

/// A *Slang* [compilation context](compile::Context) for `wasm32-unknown-unknown` targets that makes use of a *light*
/// JavaScript bridge. It is considered "light" because it only forwards the small number of high-level APIs that the
/// `Context` implements, rather than translating the full JavaScript *Slang* API. This reduces function call overhead
/// significantly, but also limits clients to the small and abstracted subset of functionality exposed by the `Context`.
pub struct Context {
	session: Session,
	compatHash: u64,
	environment: Option<compile::Environment<Module>>
}
impl Context
{
	/// Helper for obtaining a fresh *Slang* session.
	fn freshSession () -> Result<Session, ()> {
		if let Some(session) = Session::new() { Ok(session) }
		else                                  { Err(()) }
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
			let session = Self::freshSession().map_err(
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
	pub fn compileFromSource (&self, sourceCode: &str) -> Result<JsSlangModule, LoadModuleError> {
		let targetPath = PathBuf::from(format!("_unnamed__{}.slang", util::unique::uint32()));
		self.compileFromNamedSource(&targetPath, sourceCode)
	}

	///
	pub fn compileFromNamedSource (&self, targetPath: impl AsRef<Path>, sourceCode: &str)
		-> Result<JsSlangModule, LoadModuleError>
	{
		// Make sure we get a valid target path
		let targetPath = validateModulePath(targetPath.as_ref())?;

		// Let slang compile the module
		let module =  self.session.loadModuleFromSourceString(targetPath, sourceCode)?;

		// Done!
		Ok(module)
	}

	///
	pub fn loadModule (&mut self, filename: impl AsRef<Path>) -> Result<(), LoadModuleError> {
		// File loading is not currently supported by WASM Slang
		unimplemented!("Compiling from source files is not currently supported by WASM Slang");
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
	pub fn loadModuleFromIR (&mut self, targetPath: impl AsRef<Path>, bytes: &[u8]) -> Result<(), LoadModuleError>
	{
		// Make sure we get a valid target path
		let targetPath_str = validateModulePath(targetPath.as_ref())?;

		// IR bytecode loading is not currently supported by WASM Slang
		unimplemented!("IR bytecode loading is not currently supported by WASM Slang");
	}
}
impl compile::Context<Module> for Context
{
	fn replaceEnvironment (&mut self, environment: Option<compile::Environment<Module>>)
		-> Result<Option<compile::Environment<Module>>, compile::SetEnvironmentError>
	{
		// Check if the new environment is compatible (in case it's `Some`)
		/*if let Some(newEnv) = &environment && self.compatHash != newEnv.compatHash() {
			return Err(compile::SetEnvironmentError::IncompatibleEnvironment)
		}*/

		// Start from a fresh session
		let newSession = Self::freshSession().expect(
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

					Module::IR(bytes) =>
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
	fn slangjs_interopTest(moduleSourceCode: &str) -> Vec<u8>;
	fn slangjs_createSession() -> i64;
	fn slangjs_dropSession(handle: u64);
	fn slangjs_loadModuleFromSource(
		sessionHandle: u64, moduleName: &str, modulePath: &str, moduleSourceCode: &str
	) -> i64;
}
