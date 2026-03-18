
//////
//
// Module definitions
//

// Private submodule implementing the HasWGSLEquivalent trait
mod wgslequiv;
pub use wgslequiv::*; // re-export everything

// Submodule implementing the HasWGSLEquivalent trait
mod transferfunc;
pub use transferfunc::{GPUTransferFunction, builtin}; // re-export
