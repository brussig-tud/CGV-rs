
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Local imports
use crate::{self as cgv, *, renderer::data::*};



//////
//
// Structs
//

// A position-only `InterleavedElem`.
#[derive(Clone,InterleavedElem,NoNormal,NoTangent,NoRadius,NoRadiusDeriv,NoOrientation,NoScaling,NoColor)]
pub struct Position {
	#[cgv_renderAttr(pos)] pub pos: glm::Vec3
}

// An `InterleavedElem` with tangents and colors.
#[derive(Clone,InterleavedElem,ElemWithTangent,ElemWithColor,NoNormal,NoRadius,NoRadiusDeriv,NoOrientation,NoScaling)]
pub struct PosTanColor {
	#[cgv_renderAttr(pos)]     pub pos: glm::Vec3,
	#[cgv_renderAttr(tangent)] pub tan: glm::Vec3,
	#[cgv_renderAttr(color)]   pub col: cgv::RGBA,
}



//////
//
// Tests
//

#[test]
fn test_derive_interleavedElem ()
{
	// Check generated `InterleavedElem` impl
	let posOnly = Position { pos: glm::vec3(0., 1., 0.2) };
	assert_eq!(posOnly.pos(), &glm::vec3(0., 1., 0.2));

	// Check select other `ElemWith...` impls
	let posTanColor = PosTanColor {
		pos: glm::vec3(0.3, 2., 0.1), tan: glm::vec3(2., 0.1, 0.),
		col: cgv::RGBA::from_rgba_premultiplied(0.1, 0.2, 0.3, 0.5)
	};
	assert_eq!(posTanColor.tangent(), &glm::vec3(2., 0.1, 0.));
	assert_eq!(posTanColor.color(), &cgv::RGBA::from_rgba_premultiplied(0.1, 0.2, 0.3, 0.5));
}
