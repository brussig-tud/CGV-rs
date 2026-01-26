
//////
//
// Module definitions
//

/// Submodule implementing the Slang runtime context.
mod context;
pub use context::{
	Context, ContextBuilder, Module, EntryPoint, Composite, LinkedComposite, ComponentRef, EnvModule,
	EnvironmentStorage, obtainGlobalSession
}; // re-export



//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Local imports
use cgv_util as util;
use crate::compile;



//////
//
// Structs
//

/// Used by the *CGV-rs* adapters to the *Slang* compiler for lightning-fast $O(1)$ checks if a compilation target is
/// active, and if yes, which [target index](slang_native::ComponentType::target_code) it corresponds to.
type GenericActiveTargetsMap<IndexType> = [Option<IndexType>; compile::Target::NUM_SLOTS as usize];



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
