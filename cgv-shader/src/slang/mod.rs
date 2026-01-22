
//////
//
// Module definitions
//

/// Submodule implementing the Slang runtime context.
mod context;
pub use context::{Context, ContextBuilder, EnvModule, EnvironmentStorage}; // re-export

/// Submodule implementing the Slang shader program representation.
#[cfg(not(target_arch="wasm32"))]
mod program;
#[cfg(not(target_arch="wasm32"))]
pub use program::Program; // re-export



//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Slang library
#[cfg(not(target_arch="wasm32"))]
use shader_slang as slang_native;

// Local imports
use cgv_util as util;



//////
//
// Structs
//

///
#[cfg(not(target_arch="wasm32"))]
pub struct EntryPoint {
	slang: slang_native::EntryPoint,
	progBytecode: slang_native::Blob,
}
#[cfg(not(target_arch="wasm32"))]
impl EntryPoint
{
	#[inline]
	pub fn slangEntryPoint (&self) -> &slang_native::EntryPoint {
		&self.slang
	}

	#[inline]
	pub fn programBytecode (&self) -> &[u8] {
		self.progBytecode.as_slice()
	}
}



//////
//
// Functions
//

/// Report the most suitable storage type for Slang-sourced compilation environment modules.
pub fn mostSuitableEnvironmentStorageForPlatform (platform: &util::meta::SupportedPlatform) -> EnvironmentStorage
{
	// WebGPU/WASM
	if platform.isWasm() {
		// Slang-WASM currently doesn't support loading IR modules
		EnvironmentStorage::SourceCode
	}
	// All native backends
	else {
		EnvironmentStorage::IR
	}
}
