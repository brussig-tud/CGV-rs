
//////
//
// Imports
//

// Local imports
use crate::{self as cgv, renderer::spheres::*};



//////
//
// Structs
//

/// Stores the default attributes that the [`Spheres`](renderer::Spheres) will use when rendering spheres when the
/// corresponding attributes are not sourced from user data.
#[derive(Default)]
pub struct ConstantAttributes {
	///
	pub radius: f32,

	///
	pub color: Rgba,
}
pub type ConstantAttribsUniformGroup = hal::UniformGroup<ConstantAttributes>;

/// A [`renderer::Data`]-compliant GPU-side storage of varying sphere attributes.
pub struct Data {}
impl Data {
	pub fn new<D: renderer::Data> (data: D) -> Self {
		todo!()
	}

	pub fn empty () -> Self {
		Self {}
	}
}
impl renderer::Data for Data
{
	type PosIterator<'data> = util::notsafe::StridedRefIter<'data, glm::Vec3>;

	fn num (&self) -> u32 {
		todo!()
	}

	fn positions (&self) -> Self::PosIterator<'_> {
		todo!()
	}

	fn pos (&self, _index: u32) -> &glm::Vec3 {
		todo!()
	}
}
impl renderer::data::Interleaved for Data {}
impl renderer::data::CanHaveRadii for Data
{
	type RadiusIterator<'data> = util::notsafe::StridedRefIter<'data, f32>;

	fn hasRadii (&self) -> bool {
		todo!()
	}

	fn radii (&self) -> Self::RadiusIterator<'_> {
		todo!()
	}

	fn radius (&self, index: u32) -> f32 {
		todo!()
	}
}
impl renderer::data::CanHaveColors for Data
{
	type ColorIterator<'data> = util::notsafe::StridedRefIter<'data, cgv::RGBA>;

	fn hasColors (&self) -> bool {
		todo!()
	}

	fn colors (&self) -> Self::ColorIterator<'_> {
		todo!()
	}

	fn color (&self, index: u32) -> &RGBA {
		todo!()
	}
}
impl<D: renderer::Data> From<&D> for Data {
	fn from (other: &D) -> Self {
		todo!()
	}
}
