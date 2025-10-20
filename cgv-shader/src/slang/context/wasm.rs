
//////
//
// Imports
//

// Wasm-bindgen library
use wasm_bindgen::prelude::*;

// Local imports
use crate::*;
use crate::{compile::{SetEnvironmentError, AddModuleError}, slang::Program, slang::context::*};



//////
//
// Structs
//

/// A handle for the JavaScript-side `slang::Session` instance.
struct SlangSessionHandle(u32);
impl SlangSessionHandle
{
	pub fn new () -> Result<Self, ()> {
		todo!()
	}
}
impl Drop for SlangSessionHandle
{
	fn drop (&mut self) {
		todo!()
	}
}

/// An thin layer around the JavaScript-side `slang::Session` instance.
struct SlangSession {}

/// A *Slang* [compilation context](compile::Context) for `wasm32-unknown-unknown` targets that makes use of a *light*
/// JavaScript bridge. It is considered "light" because it only forwards the small number of high-level APIs that the
/// `Context` implements, rather than translating the full JavaScript *Slang* API. This reduces function call overhead
/// significantly but also limits clients to the small and abstracted subset of functionality exposed by the `Context`.
pub struct Context {
	sessionHandle: SlangSession,
	searchPath: Vec<String>,
	compatHash: u64,
	environment: Option<compile::Environment<Module>>
}
impl Context
{
	/// Helper for obtaining a fresh *Slang* session.
	fn freshSession (searchPath: &[impl AsRef<Path>]) -> Result<SlangSession, ()> {
		todo!()
	}

	/// Create a new *Slang* context for the given compilation target using the given module search path.
	///
	/// # Arguments
	///
	/// * `target` – The target representation this `Context` will compile/transpile to.
	/// * `searchPath` – The module search path for the *Slang* compiler.
	pub fn forTarget (target: CompilationTarget, searchPath: &[impl AsRef<Path>]) -> anyhow::Result<Self> {
		todo!()
	}

	/// Create a new *Slang* context for the *WGSL* target with the given module search path.
	///
	/// # Arguments
	///
	/// * `searchPath` – The module search path for the *Slang* compiler.
	pub fn new (searchPath: &[impl AsRef<Path>]) -> anyhow::Result<Self> {
		Self::forTarget(CompilationTarget::WGSL, searchPath)
	}

	///
	pub fn targetType (&self) -> WgpuSourceType {
		todo!()/*match self.sessionConfig.target {
			CompilationTarget::SPIRV(_) => WgpuSourceType::SPIRV,
			CompilationTarget::WGSL => WgpuSourceType::WGSL
		}*/
	}

	/// Build a shader program from the given *Slang* source file.
	///
	/// # Arguments
	///
	/// * `sourceFile` – The `.slang` file containing the shader source code.
	pub fn buildProgram (&self, sourceFile: impl AsRef<Path>) -> anyhow::Result<Program> {
		Program::fromSource(self, sourceFile)
	}

	///
	pub fn compile (&self, sourcefile: impl AsRef<Path>) -> Result<crate::slang::Module, LoadModuleError>
	{
		todo!()
		/*// Let slang load and compile the module
		let module =  self.session.load_module(
			sourcefile.as_ref().to_string_lossy().as_ref()
		).or_else(|err|
			Err(LoadModuleError::CompilationError(format!("File {} – {err}", sourcefile.as_ref().display())))
		)?;

		// Done!
		Ok(module)*/
	}

	///
	#[inline]
	pub fn compileFromSource (&self, sourceCode: &str) -> Result<crate::slang::Module, LoadModuleError> {
		let targetPath = PathBuf::from(format!("_unnamed__{}.slang", util::unique::uint32()));
		self.compileFromNamedSource(&targetPath, sourceCode)
	}

	///
	pub fn compileFromNamedSource (&self, targetPath: impl AsRef<Path>, sourceCode: &str)
		-> Result<crate::slang::Module, LoadModuleError>
	{
		// Make sure we get a valid target path
		let targetPath = validateModulePath(targetPath.as_ref())?;

		// Let slang compile the module
		todo!()
		/*let module =  self.session.load_module_from_source_string(targetPath, targetPath, sourceCode)
			.or_else(|err| Err(LoadModuleError::CompilationError(format!("{err}"))))?;

		// Done!
		Ok(module)*/
	}

	///
	pub fn loadModule (&mut self, filename: impl AsRef<Path>) -> Result<(), LoadModuleError>
	{
		todo!()
		/*let module = Module::fromSlangModule(self.compile(&filename)?).map_err(
			|err| LoadModuleError::CompilationError(format!("{err}"))
		)?;
		storeInEnvironment(self.environment.as_mut(), filename, module).map_err(|err| match err {
			AddModuleError::DuplicateModulePaths(path) => LoadModuleError::DuplicatePath(path)
		})*/
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
			EnvironmentStorage::IR => todo!()/*Module::fromSlangIRBytes(slangModule.serialize()?).map_err(
				|err| LoadModuleError::CompilationError(format!("{err}"))
			)?*/
		};

		// Store the module in the environment
		storeInEnvironment(self.environment.as_mut(), targetPath, module).map_err(|err| match err {
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
		todo!();
		/*let irBlob = slang::ComPtr::new(slang::VecBlob::from_slice(bytes));
		self.session.load_module_from_ir_blob(targetPath_str, targetPath_str, &*irBlob).or_else(
			|err| Err(LoadModuleError::CompilationError(format!("{err}")))
		)?;*/

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
		let newSession = Self::freshSession(&self.searchPath).expect(
			"Creating a Slang session identical to an existing one should never fail unless there are \
			unrecoverable external circumstances (out-of-memory etc.)"
		);

		// Apply the new environment to the new session
		if let Some(newEnv) = &environment
		{
			for module in newEnv.modules()
			{
				let path = encodeValidModulePath(&module.path);
				todo!();
				/*match &module.module
				{
					Module::SourceCode(sourceCode) =>
						newSession.load_module_from_source_string(&path, "", sourceCode).or_else(|err|Err(
							SetEnvironmentError::ImplementationSpecific(
								LoadModuleError::CompilationError(format!("{err}")).into()
							)
						))?,

					Module::IR(bytes) => {
						let irBlob = slang::ComPtr::new(slang::VecBlob::from_slice(bytes));
						newSession.load_module_from_ir_blob(&path, "", &*irBlob).or_else(|err|Err(
							SetEnvironmentError::ImplementationSpecific(
								LoadModuleError::CompilationError(format!("{err}")).into()
							)
						))?
					}
				};*/
			}
		}

		// Commit both
		let oldEnv = self.environment.take();
		self.sessionHandle = newSession;
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
