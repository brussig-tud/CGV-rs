
//////
//
// Module definitions
//

/// Submodule implementing the Slang runtime context.
mod context;
pub use context::{Context, Module, EnvironmentStorage}; // re-export

/// Submodule implementing the Slang shader program representation.
mod program;
pub use program::Program; // re-export



//////
//
// Imports
//

// Standard library
use std::path::Path;

// Wasm-bindgen library
#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

// Anyhow library
use anyhow;

// Slang library
use shader_slang as slang;

// Local imports
use cgv_util as util;
use crate::CompilationTarget;



//////
//
// Structs
//

/// 
pub struct EntryPoint {
	slang: slang::EntryPoint,
	progBytecode: slang::Blob,
}
impl EntryPoint
{
	#[inline]
	pub fn slangEntryPoint (&self) -> &slang::EntryPoint {
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

/// Turn a list of [compilation targets](CompilationTarget) into a list of [*Slang* contexts](Context) for compiling to
/// these targets.
pub fn createContextsForTargets<'a> (targets: &[CompilationTarget], shaderPath: &[impl AsRef<Path>])
-> anyhow::Result<cgv_util::BorrowVec<'a, Context>> {
	let mut contexts = Vec::<Context>::with_capacity(targets.len());
	for &target in targets {
		contexts.push(Context::forTarget(target, shaderPath)?);
	}
	Ok(contexts.into())
}

/// Report the most suitable storagy type for Slang-sourced compilation enviorment modules.
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

#[cfg(target_arch="wasm32")]
#[wasm_bindgen]
extern "C" {
	fn slangjs_interopTest(moduleSourceCode: &str) -> Vec<u8>;
}
#[cfg(target_arch="wasm32")]
pub fn testJsInterop(moduleSourceCode: &str) -> Vec<u8> {
	slangjs_interopTest(moduleSourceCode)
}
