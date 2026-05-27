
//////
//
// Module definitions
//

/// Tests for the `renderer` module.
mod renderer;



//////
//
// Tests for functionality in the root module
//

////
// Imports

// Local imports
use crate::*;


////
// Tests

#[test]
fn test_RGBAisPremultiplied () {
	let rgba = cgv::RGBA::from_rgba_unmultiplied(0.5, 0.25, 1., 0.5);
	assert!(rgba.r() == 0.25 && rgba.g() == 0.125 && rgba.b() == 0.5 && rgba.a() == 0.5);
}
