//////
//
// Imports
//

// Standard library
use std::marker::PhantomData;

// Local imports
use crate::{self as cgv, *};



//////
//
// Structs
//

/// Non-indexed, interleaved test [render data](renderer::data::Data) with positions and normals.
struct NonIndexedInterleavedPosNormalRadiusColor {
	data: Vec<(/* positions: */glm::Vec4, /* normals: */glm::Vec4, /* radius */f32, /* color */cgv::RGBA)>
}

impl renderer::Data for NonIndexedInterleavedPosNormalRadiusColor {
	fn num (&self) -> u32 {
		self.data.len() as u32
	}
}
impl renderer::data::Interleaved for NonIndexedInterleavedPosNormalRadiusColor {}

impl renderer::data::HasPositions for NonIndexedInterleavedPosNormalRadiusColor {
	type PosIterator = util::notsafe::StridedIter<glm::Vec4>;

	fn positions (&self) -> Self::PosIterator {
		util::notsafe::strided_iter!(self.data, 0, glm::Vec4)
	}

	fn pos (&self, index: u32) -> &glm::Vec4 {
		&self.data[index as usize].0
	}
}

impl renderer::data::HasNormals for NonIndexedInterleavedPosNormalRadiusColor {
	type NormalIterator = util::notsafe::StridedIter<glm::Vec4>;

	fn normals (&self) -> Self::NormalIterator {
		util::notsafe::strided_iter!(self.data, 1, glm::Vec4)
	}

	fn normal (&self, index: u32) -> &glm::Vec4 {
		&self.data[index as usize].1
	}
}

impl renderer::data::HasRadii for NonIndexedInterleavedPosNormalRadiusColor {
	type RadiusIterator = util::notsafe::StridedIter<f32>;

	fn radii (&self) -> Self::RadiusIterator {
		util::notsafe::strided_iter!(self.data, 2, f32)
	}

	fn radius (&self, index: u32) -> f32 {
		self.data[index as usize].2
	}
}

impl renderer::data::HasColors for NonIndexedInterleavedPosNormalRadiusColor {
	type ColorIterator = util::notsafe::StridedIter<cgv::RGBA>;

	fn colors (&self) -> Self::ColorIterator {
		util::notsafe::strided_iter!(self.data, 3, cgv::RGBA)
	}

	fn color (&self, index: u32) -> &cgv::RGBA {
		&self.data[index as usize].3
	}
}
