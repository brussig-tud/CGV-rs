
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
use cgv_util::bitcode;



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

/// Determine the most suitable shader compilation target for the current target platform.
#[inline]
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

/// Return a list of feasible shader compilation target for the current target platform, from most to least suitable.
#[inline]
pub fn feasibleCompilationTargets () -> &'static [CompilationTarget]
{
	static COMPILATION_TARGETS: &[CompilationTarget] = {
		// WebGPU/WASM
		#[cfg(target_arch="wasm32")] {
			&[CompilationTarget::WGSL, CompilationTarget::SPIRV(false)]
		}
		// All native backends (currently always considers SPIR-V preferable even on non-Vulkan backends)
		#[cfg(not(target_arch="wasm32"))] {
			#[cfg(debug_assertions)] {
				&[CompilationTarget::SPIRV(true), CompilationTarget::WGSL]
			}
			#[cfg(not(debug_assertions))] {
				&[CompilationTarget::SPIRV(false), CompilationTarget::WGSL]
			}
		}
	};
	COMPILATION_TARGETS
}
