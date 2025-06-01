
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

/// Submodule providing the abstractions for *Slang* [`Program`](slang::Program)s.
#[cfg(feature="slang_runtime")]
pub mod slang;



//////
//
// Imports
//

// Standard library
/* nothing here yet */



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
