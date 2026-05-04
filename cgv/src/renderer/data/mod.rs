
//////
//
// Module definitions
//

/// Module defining the host-side data model.
pub mod host;

/// Module defining the GPU-side data model.
pub mod gpu;



//////
//
// Imports
//

// Local imports
pub use cgv_derive::{
	// Re-export the relevant procedural derive macros from cgv-derive
	InterleavedElem, ElemWithNormal, ElemWithTangent, ElemWithRadius, ElemWithRadiusDeriv, ElemWithOrientation,
	ElemWithScaling, ElemWithColor
};
