
//////
//
// Module definitions
//

/// Submodule implementing the Slang runtime interface for native builds
#[cfg(not(target_arch="wasm32"))]
mod native;
#[cfg(not(target_arch="wasm32"))]
pub use native::*; // re-export

/// Submodule implementing the Slang runtime interface for WASM builds
#[cfg(target_arch="wasm32")]
mod wasm;
#[cfg(target_arch="wasm32")]
pub use wasm::*; // re-export




//////
//
// Imports
//

// Standard library
use std::path::Path;

// Anyhow library
use anyhow::Result;

// Local imports
use crate::CompilationTarget;



//////
//
// Functions
//

/// Turn a list of [compilation targets](CompilationTarget) into a list of [*Slang* contexts](Context) for compiling to
/// these targets.
pub fn createContextsForTargets<'a> (targets: &[CompilationTarget], shaderPath: &[impl AsRef<Path>])
-> Result<cgv_util::BorrowVec<'a, Context>> {
	let mut contexts = Vec::<Context>::with_capacity(targets.len());
	for &target in targets {
		contexts.push(Context::forTarget(target, shaderPath)?);
	}
	Ok(contexts.into())
}
