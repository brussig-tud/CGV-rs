
//////
//
// Language config
//

// Eff this convention.
#![allow(non_snake_case)]



//////
//
// Module definitions
//

/// Submodule providing the shader [`Package`].
mod package;
pub use package::Package; // - re-export

/// Submodule providing the shader [`Program`].
mod program;
pub use program::Program; // - re-export



//////
//
// Imports
//

// Standard library
use std::path::Path;

// Anyhow library
use anyhow::*;

// Slang library
use slang;
use slang::Downcast;



//////
//
// Enums
//

/// Enum describing the platform shaders are being built for.
pub enum TargetPlatform {
	/// Build shaders for native applications, specifying whether they should be debuggable or not.
	Native(bool),

	/// Build shaders for the WASM platform.
	Wasm
}


//////
//
// Classes
//

///
pub struct SlangContext {
	#[allow(dead_code)] // we need to keep this around as it dictates the lifetime of `session`
	globalSession: slang::GlobalSession,

	pub(crate) session: slang::Session,
}
impl SlangContext
{
	/// Create a new Slang context for the given target platform using the given module search path.
	///
	/// # Arguments
	///
	/// * `targetPlatform` – The platform compiled shaders will run on.
	/// * `searchPath` – The module search path for the *Slang* compiler.
	pub fn forPlatform (targetPlatform: TargetPlatform, searchPath: &[impl AsRef<Path>]) -> anyhow::Result<Self>
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
		// - compile flags
		let sessionOptions = slang::CompilerOptions::default().matrix_layout_row(true);
		let sessionOptions = match targetPlatform
		{
			TargetPlatform::Native(debug) => sessionOptions
				.emit_spirv_directly(true)
				.optimization(
					if debug { slang::OptimizationLevel::None } else { slang::OptimizationLevel::Maximal }
				)
				.debug_information(
					if debug { slang::DebugInfoLevel::Maximal } else { slang::DebugInfoLevel::None }
				),

			TargetPlatform::Wasm => sessionOptions
				.optimization(slang::OptimizationLevel::Maximal)
				.debug_information(slang::DebugInfoLevel::None)
		};
		// - output profile
		let targetDesc = slang::TargetDesc::default()
			.profile(globalSession.find_profile("glsl_460"));
		let targetDesc = match targetPlatform {
			TargetPlatform::Native(_) => targetDesc.format(slang::CompileTarget::Spirv),
			TargetPlatform::Wasm => targetDesc.format(slang::CompileTarget::Wgsl)
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

		// Done!
		Ok(Self {	globalSession, session })
	}

	/// Create a new Slang context with the given module search path. The target platform is automatically detected
	/// before delegating to [`Self::forPlatform`].
	///
	/// # Arguments
	///
	/// * `searchPath` – The module search path for the *Slang* compiler.
	pub fn new (searchPath: &[impl AsRef<Path>]) -> anyhow::Result<Self>
	{
		#[cfg(not(target_arch="wasm32"))] {
			Self::forPlatform(TargetPlatform::Native(cfg!(debug_assertions)), searchPath)
		}
		#[cfg(target_arch="wasm32")] {
			Self::forPlatform(TargetPlatform::Wasm, searchPath)
		}
	}

	/// Build a shader program from the given *Slang* source file.
	///
	/// # Arguments
	///
	/// * `sourceFile` – The `.slang` file containing the shader source code.
	pub fn buildProgram (&self, sourceFile: impl AsRef<Path>) -> Result<Program> {
		Program::new(self, sourceFile)
	}
}
