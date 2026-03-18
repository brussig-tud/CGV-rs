
//////
//
// Module definitions
//

// Submodule exposing GPU implementation providers for the built-in transfer functions in `cgv::data`.
pub mod builtin;



//////
//
// Imports
//

// Standard library
/* nothing here yet */



//////
//
// Structs and enums
//

/* nothing here yet */



//////
//
// Traits
//

/// The trait of being a transfer function usable in a `TransferFunctionRunner`.
pub trait GPUTransferFunction {
	fn wgslFnName (&self) -> String;
	fn wgslFnDef (&self) -> String;
}
