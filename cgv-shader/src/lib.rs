
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

/// Submodule implementing shader compilation infrastructure
#[cfg(feature="compilation")]
pub mod compile;

/// Submodule implementing the shader program abstraction
mod program;
pub use program::Program; // re-export

/// Submodule implementing the shader package facilities
mod pak;
pub use pak::Package; // re-export

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
impl WgpuSourceType
{
	/// Instantiate with the most suitable *WGPU* source type for the platform the caller is running on.
	#[inline(always)]
	pub const fn mostSuitable() -> WgpuSourceType
	{
		// WebGPU/WASM
		#[cfg(target_arch="wasm32")] {
			WgpuSourceType::SPIRV
		}
		// All native backends (currently always considers SPIR-V preferable even on non-Vulkan backends)
		#[cfg(not(target_arch="wasm32"))] {
			WgpuSourceType::SPIRV
		}
	}

	/// Instantiate with the most suitable *WGPU* source type for the given platform.
	pub const fn mostSuitableForPlatform (platform: &util::meta::SupportedPlatform) ->WgpuSourceType
	{
		// WebGPU/WASM
		if platform.isWasm() {
			WgpuSourceType::WGSL
		}
		// All native backends
		else {
			// Currently always considers SPIR-V preferable even on non-Vulkan backends
			// TODO: somehow incorporate notion of WGPU backend into this decision
			WgpuSourceType::SPIRV
		}
	}
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

/// Return a list of feasible *WGPU* source types for the platform the caller is running on, from most to least
/// suitable.
#[inline(always)]
pub const fn feasibleSourceTypes() -> &'static [WgpuSourceType]
{
	// WebGPU/WASM
	#[cfg(target_arch="wasm32")]
	const SOURCE_TYPES: [WgpuSourceType; 2] = [WgpuSourceType::WGSL, WgpuSourceType::SPIRV];

	// All native backends (currently always considers SPIR-V preferable even on non-Vulkan backends)
	#[cfg(not(target_arch="wasm32"))]
	const SOURCE_TYPES: [WgpuSourceType; 2] = [WgpuSourceType::SPIRV, WgpuSourceType::WGSL];

	&SOURCE_TYPES
}

/// Return a list of feasible *WGPU* source types for the platform the caller is running on, from most to least
/// suitable.
#[inline(always)]
pub const fn feasibleSourceTypesForPlatform(platform: &util::meta::SupportedPlatform) -> &'static [WgpuSourceType]
{
	// WebGPU/WASM
	if platform.isWasm() {
		const SOURCE_TYPES: [WgpuSourceType; 2] = [WgpuSourceType::WGSL, WgpuSourceType::SPIRV];
		&SOURCE_TYPES
	}
	// All native backends
	else {
		// Currently always considers SPIR-V preferable even on non-Vulkan backends
		if !platform.isDebug() {
			const SOURCE_TYPES: [WgpuSourceType; 2] = [WgpuSourceType::SPIRV, WgpuSourceType::WGSL];
			&SOURCE_TYPES
		}
		else {
			const SOURCE_TYPES: [WgpuSourceType; 2] = [WgpuSourceType::SPIRV, WgpuSourceType::WGSL];
			&SOURCE_TYPES
		}
	}
}
