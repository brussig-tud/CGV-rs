
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Local imports
use crate::*;



//////
//
// Structs
//

/// Non-indexed, interleaved test [render data](renderer::data::Data) with positions and normals.
struct NonIndexedInterleavedPosNormal {
	data: Vec<(/* positions: */glm::Vec4, /* normals: */glm::Vec4)>
}
impl renderer::Data for NonIndexedInterleavedPosNormal {
	fn num (&self) -> u32 {
		self.data.len() as u32
	}
}
impl renderer::data::Interleaved for NonIndexedInterleavedPosNormal {}
impl renderer::data::HasPositions for NonIndexedInterleavedPosNormal {}
impl renderer::data::HasNormals for NonIndexedInterleavedPosNormal {}

