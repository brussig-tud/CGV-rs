
//////
//
// Imports
//

// Local imports
use crate::{*, renderer::spheres::*};



//////
//
// Structs
//

/// Stores the default attributes that the [`Spheres`](renderer::Spheres) will use when rendering spheres when the corresponding
/// attributes are not sourced from user data.
#[derive(Default)]
pub struct ConstantAttributes {
	///
	pub radius: f32,

	///
	pub color: Rgba,
}
pub type ConstantAttribsUniformGroup = hal::UniformGroup<ConstantAttributes>;
