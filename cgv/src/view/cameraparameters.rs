
//////
//
// Imports
//

// Standard library
/* Nothing here yet */

// Egui library
use egui;

// Local imports
use crate::*;
use crate::view::*;



//////
//
// Classes
//

#[derive(Clone, Copy)]
pub struct Intrinsics {
	pub fovY: FoV,
	pub aspect: f32,
	pub f: f32,
	pub zNear: f32,
	pub zFar: f32
}
impl Intrinsics {
	fn defaultWithAspect (aspect: f32) -> Self {
		Self {
			aspect, f: 1., zNear: 0.01, zFar: 100.,
			fovY: FoV::Perspective(math::deg2rad!(60.)),
		}
	}
}

#[derive(Clone, Copy)]
pub struct Extrinsics {
	pub eye: glm::Vec3,
	pub dir: glm::Vec3,
	pub up: glm::Vec3,
}
impl Default for Extrinsics {
	fn default () -> Self {
		Self {
			eye: glm::Vec3::zeros(),
			dir: glm::vec3(0., 0., -1.),
			up: glm::vec3(0., 1., 0.)
		}
	}
}

#[derive(Clone, Copy)]
pub struct CameraParameters {
	pub intrinsics: Intrinsics,
	pub extrinsics: Extrinsics,
}
impl CameraParameters
{
	pub fn defaultWithAspect (aspect: f32) -> Self {
		Self {
			intrinsics: Intrinsics::defaultWithAspect(aspect),
			extrinsics: Default::default()
		}
	}

	pub(crate) fn sidepanel (&mut self, ui: &egui::Ui) {

	}
}
