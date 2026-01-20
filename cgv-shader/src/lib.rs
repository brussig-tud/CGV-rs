
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
