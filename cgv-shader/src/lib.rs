
//////
//
// Language config
//

// Eff this convention.
#![allow(non_snake_case)]

// Enable the `intersperse` iterator feature
#![feature(iter_intersperse)]



//////
//
// Module definitions
//

/// Submodule implementing shader compilation infrastructure
#[cfg(feature="compilation")]
pub mod compile;

/// Submodule implementing the shader package facilities
mod pak;
pub use pak::{Program, Package}; // re-export

/// Submodule providing the abstractions for *Slang* [`Program`](slang::Program)s.
#[cfg(feature="slang_runtime")]
pub mod slang;



//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Bitcode library
use bitcode;

// Local imports
use cgv_util as util;
use util::meta;



//////
//
// Enums
//

/// Enum describing the type of a [program instance](Program) in accordance with *WGPU*
/// [shader module source](wgpu::ShaderSource) types.
#[derive(Debug,Ord,PartialOrd,Eq,PartialEq,Copy,Clone,bitcode::Encode,bitcode::Decode)]
pub enum WgpuSourceType {
	/// The source is a blob of *SPIR-V* bytecode, potentially including debug information.
	SPIRV,

	/// The source is a [`String`] of self-contained *WGSL* code.
	WGSL,

	/// The source is a [`String`] of self-contained *GLSL* code.
	GLSL
}
impl std::fmt::Display for WgpuSourceType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			WgpuSourceType::SPIRV => write!(f, "SPIR-V"),
			WgpuSourceType::WGSL => write!(f, "WGSL"),
			WgpuSourceType::GLSL => write!(f, "GLSL")
		}
	}
}



//////
//
// Functions
//

/// Determine the most suitable shader compilation target for the platform the module was built for.
#[inline(always)]
pub fn mostSuitableCompilationTarget () -> compile::Target
{
	// WebGPU/WASM
	#[cfg(target_arch="wasm32")] {
		compile::Target::WGSL
	}
	// All native backends (currently always considers SPIR-V preferable even on non-Vulkan backends)
	#[cfg(not(target_arch="wasm32"))] {
		#[cfg(debug_assertions)] {
			compile::Target::SPIRV(true)
		}
		#[cfg(not(debug_assertions))] {
			compile::Target::SPIRV(false)
		}
	}
}

/// Determine the most suitable shader compilation target for the given platform.
pub fn mostSuitableCompilationTargetForPlatform (platform: &meta::SupportedPlatform) -> compile::Target
{
	// WebGPU/WASM
	if platform.isWasm() {
		compile::Target::WGSL
	}
	// All native backends
	else {
		// Currently always considers SPIR-V preferable even on non-Vulkan backends
		compile::Target::SPIRV(platform.isDebug())
	}
}

/// Return a list of feasible shader compilation target for the platform the module was built for, from most to least
/// suitable.
#[inline(always)]
pub fn feasibleCompilationTargets () -> &'static [compile::Target]
{
	// WebGPU/WASM
	#[cfg(target_arch="wasm32")]
		const COMPILATION_TARGETS: [compile::Target; 2] = [compile::Target::WGSL, compile::Target::SPIRV(false)];

	// All native backends (currently always considers SPIR-V preferable even on non-Vulkan backends)
	#[cfg(all(not(target_arch="wasm32"),debug_assertions))]
		const COMPILATION_TARGETS: [compile::Target; 2] = [compile::Target::SPIRV(true), compile::Target::WGSL];
	#[cfg(all(not(target_arch="wasm32"),not(debug_assertions)))]
		const COMPILATION_TARGETS: [compile::Target; 2] = [compile::Target::SPIRV(false), compile::Target::WGSL];

	&COMPILATION_TARGETS
}

/// Return a list of feasible shader compilation target for the given platform, from most to least suitable.
pub fn feasibleCompilationTargetsForPlatform (platform: &meta::SupportedPlatform) -> &'static [compile::Target]
{
	// WebGPU/WASM
	if platform.isWasm() {
		const COMPILATION_TARGETS: [compile::Target; 2] = [compile::Target::WGSL, compile::Target::SPIRV(false)];
		&COMPILATION_TARGETS
	}
	// All native backends
	else {
		// Currently always considers SPIR-V preferable even on non-Vulkan backends
		if !platform.isDebug() {
			const COMPILATION_TARGETS: [compile::Target; 2] = [compile::Target::SPIRV(false), compile::Target::WGSL];
			&COMPILATION_TARGETS
		}
		else {
			const COMPILATION_TARGETS: [compile::Target; 2] = [compile::Target::SPIRV(true), compile::Target::WGSL];
			&COMPILATION_TARGETS
		}
	}
}
