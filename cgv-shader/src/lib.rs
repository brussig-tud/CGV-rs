
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

// Bitcode library
use cgv_util::bitcode;

// Local imports
use cgv_util as util;
use util::meta;



//////
//
// Enums
//

/// Enum describing a shader compilation target (mostly mirrors [`SourceType`] plus extra information).
#[derive(Debug,Clone,Copy)]
pub enum CompilationTarget {
	/// Compile shaders to *SPIR-V*, specifying whether they should be debuggable or not.
	SPIRV(/* debug: */bool),

	/// Transpile shaders to *WGSL*.
	WGSL
}

/// Enum describing the type of a [program instance](Program) in accordance with *WGPU*
/// [shader module source](wgpu::ShaderSource) types.
#[derive(Debug,Ord,PartialOrd,Eq,PartialEq,Copy,Clone,bitcode::Encode,bitcode::Decode)]
pub enum SourceType {
	/// The source is a blob of *SPIR-V* bytecode, potentially including debug information.
	SPIRV,

	/// The source is a [`String`] of self-contained *WGSL* code.
	WGSL
}
impl std::fmt::Display for SourceType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			SourceType::SPIRV => write!(f, "SPIR-V"),
			SourceType::WGSL => write!(f, "WGSL")
		}
	}
}



//////
//
// Functions
//

/// Determine the most suitable shader compilation target for the platform the module was built for.
#[inline(always)]
pub fn mostSuitableCompilationTarget () -> CompilationTarget
{
	// WebGPU/WASM
	#[cfg(target_arch="wasm32")] {
		CompilationTarget::WGSL
	}
	// All native backends (currently always considers SPIR-V preferable even on non-Vulkan backends)
	#[cfg(not(target_arch="wasm32"))] {
		#[cfg(debug_assertions)] {
			CompilationTarget::SPIRV(true)
		}
		#[cfg(not(debug_assertions))] {
			CompilationTarget::SPIRV(false)
		}
	}
}

/// Determine the most suitable shader compilation target for the given platform.
pub fn mostSuitableCompilationTargetForPlatform (platform: &meta::SupportedPlatform) -> CompilationTarget
{
	// WebGPU/WASM
	if platform.isWasm() {
		CompilationTarget::WGSL
	}
	// All native backends
	else {
		// Currently always considers SPIR-V preferable even on non-Vulkan backends
		CompilationTarget::SPIRV(platform.isDebug())
	}
}

/// Return a list of feasible shader compilation target for the platform the module was built for, from most to least
/// suitable.
#[inline(always)]
pub fn feasibleCompilationTargets () -> &'static [CompilationTarget]
{
	// WebGPU/WASM
	#[cfg(target_arch="wasm32")]
		const COMPILATION_TARGETS: [CompilationTarget; 2] = [CompilationTarget::WGSL, CompilationTarget::SPIRV(false)];

	// All native backends (currently always considers SPIR-V preferable even on non-Vulkan backends)
	#[cfg(all(not(target_arch="wasm32"),debug_assertions))]
		const COMPILATION_TARGETS: [CompilationTarget; 2] = [CompilationTarget::SPIRV(true), CompilationTarget::WGSL];
	#[cfg(all(not(target_arch="wasm32"),not(debug_assertions)))]
		const COMPILATION_TARGETS: [CompilationTarget; 2] = [CompilationTarget::SPIRV(false), CompilationTarget::WGSL];

	&COMPILATION_TARGETS
}

/// Return a list of feasible shader compilation target for the given platform, from most to least suitable.
pub fn feasibleCompilationTargetsForPlatform (platform: &meta::SupportedPlatform) -> &'static [CompilationTarget]
{
	// WebGPU/WASM
	if platform.isWasm() {
		const COMPILATION_TARGETS: [CompilationTarget; 2] = [CompilationTarget::WGSL, CompilationTarget::SPIRV(false)];
		&COMPILATION_TARGETS
	}
	// All native backends
	else {
		// Currently always considers SPIR-V preferable even on non-Vulkan backends
		if !platform.isDebug() {
			const COMPILATION_TARGETS: [CompilationTarget; 2] = [CompilationTarget::SPIRV(false), CompilationTarget::WGSL];
			&COMPILATION_TARGETS
		}
		else {
			const COMPILATION_TARGETS: [CompilationTarget; 2] = [CompilationTarget::SPIRV(true), CompilationTarget::WGSL];
			&COMPILATION_TARGETS
		}
	}
}
