
//////
//
// Module definitions
//

/// Module defining the host-side data model.
pub mod host;

/// Module defining the GPU-side data model.
pub mod gpu;

/// Module implementing runtime-wrappers for compile-time guarantees about presence of data attributes.
mod guarantees;
pub use guarantees::*; // re-export all public facilities (mainly the guarantee wrapper and combination aliases).
