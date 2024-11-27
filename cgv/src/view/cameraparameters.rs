
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
use crate::util::math;
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
	fn defaultWithAspect (aspect: f32) -> Self { Self {
		aspect, f: 1., zNear: 0.01, zFar: 100.,
		fovY: FoV::Perspective(math::deg2rad!(60.)),
	}}

	pub fn frustumDiameterAtFocus (fov: f32, f: f32) -> f32 {
		let h05 = f * f32::tan(0.5*fov);
		h05+h05
	}

	pub fn angleForFrustumDiameterAndFocus (diameter: f32, f: f32) -> f32 {
		let theta = f32::atan(0.5*diameter / f);
		theta+theta
	}

	pub fn focusDistForFrustumDiameterAndFov (diameter: f32, fov: f32) -> f32 {
		let f = f32::tan(0.5*fov)/(0.5*diameter);
		f
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

	pub(crate) fn ui (camera: &mut dyn Camera, ui: &mut egui::Ui)
	{
		// Track camera parameters wrt. to current values
		let params_orig = camera.parameters();
		let mut params = params_orig.clone();
		let mut changed = false;

		// Create UI
		ui.vertical(|ui| {
			// --- Prelude: layouting calculations --------------------------------------------------------------------
			let awidth = ui.available_width();
			let rhswidth = f32::max(136f32, awidth*1./2.);
			let lhsminw = f32::max(awidth-rhswidth - ui.spacing().item_spacing.x, 0.);
			ui.spacing_mut().slider_width = rhswidth-56.;
			// --- Compounds ------------------------------------------------------------------------------------------
			ui.label(egui::RichText::new("Compounds").underline());
			egui::Grid::new("CGV__cam_cmpd").num_columns(2).striped(true).show(ui, |ui| {
				/* -- Zoom (affects f, eye) ----------------------------------------- */
				// - define UI
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsminw);
					ui.label("zoom")
				});
				ui.add(
					egui::Slider::new(&mut params.intrinsics.f, params.intrinsics.zNear..=params.intrinsics.zFar)
						.drag_value_speed(0.03125*params_orig.intrinsics.f as f64)
						.clamping(egui::SliderClamping::Never)
				);
				ui.end_row();
				// - handle changes
				params.intrinsics.f = f32::max(params.intrinsics.f, 0.);
				if params.intrinsics.f != params_orig.intrinsics.f {
					let focus =
						params.extrinsics.eye + params.extrinsics.dir*params_orig.intrinsics.f;
					params.extrinsics.eye = focus - params.extrinsics.dir*params.intrinsics.f;
					changed = true;
				}
			});
			// --- Intrinsics -----------------------------------------------------------------------------------------
			ui.label(egui::RichText::new("Intrinsics").underline());
			let mut ortho = params.intrinsics.fovY.isOrthographic();
			egui::Grid::new("CGV__cam_intr").num_columns(2).striped(true).show(ui, |ui| {
				/* -- perspective/orthographic -------------------------------------- */
				// - define UI
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsminw);
					ui.label("projection");
				});
				ui.add(egui::Checkbox::new(&mut ortho, "orthographic"));
				ui.end_row();
				// - handle changes
				if ortho != params_orig.intrinsics.fovY.isOrthographic() {
					params.intrinsics.fovY = match params_orig.intrinsics.fovY {
						FoV::Perspective(fovY)
						=> FoV::Orthographic(Intrinsics::frustumDiameterAtFocus(fovY, params_orig.intrinsics.f)),

						FoV::Orthographic(height) => FoV::Perspective(
							Intrinsics::angleForFrustumDiameterAndFocus(height, params_orig.intrinsics.f)
						)
					};
					changed = true;
				}
				/* -- FoV ----------------------------------------------------------- */
				// - define UI
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsminw);
					ui.label("FoV (Y)")
				});
				match params.intrinsics.fovY
				{
					FoV::Perspective(old) => {
						let old = math::rad2deg!(old);
						let mut new = old;
						ui.add(
							egui::Slider::new(&mut new, 1f32..=179.)
								.drag_value_speed(0.03125*params_orig.intrinsics.f as f64)
								.clamping(egui::SliderClamping::Never)
						);
						if new != old {
							params.intrinsics.fovY = FoV::Perspective(math::deg2rad!(new));
							changed = true;
						}
					},
					FoV::Orthographic(old) => {
						let mut new = old;
						ui.add(
							egui::Slider::new(&mut new, 0.1..=100.)
								.logarithmic(true)
								.drag_value_speed(0.03125*params_orig.intrinsics.f as f64)
								.clamping(egui::SliderClamping::Never)
						);
						if new != old {
							params.intrinsics.fovY = FoV::Orthographic(new);
							changed = true;
						}
					}
				}
				ui.end_row();
				// - handle changes
				params.intrinsics.f = f32::max(params.intrinsics.f, 0.);
				changed |= params.intrinsics.f != params_orig.intrinsics.f;
				/* -- f ------------------------------------------------------------- */
				// - define UI
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsminw);
					ui.label("focal distance")
				});
				ui.add(
					egui::Slider::new(&mut params.intrinsics.f, params.intrinsics.zNear..=params.intrinsics.zFar)
						.drag_value_speed(0.03125*params_orig.intrinsics.f as f64)
						.clamping(egui::SliderClamping::Never)
				);
				ui.end_row();
				// - handle changes
				params.intrinsics.f = f32::max(params.intrinsics.f, 0.);
				changed |= params.intrinsics.f != params_orig.intrinsics.f;
				/* -- zNear --------------------------------------------------------- */
				// - define UI
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsminw);
					ui.label("zNear")
				});
				ui.add(
					egui::Slider::new(
						&mut params.intrinsics.zNear, 0.0001..=f32::min(10., params_orig.intrinsics.zFar)
					)
						.logarithmic(true)
						.drag_value_speed(0.03125*params_orig.intrinsics.zNear as f64)
						.clamping(egui::SliderClamping::Never)
				);
				ui.end_row();
				// - handle changes
				params.intrinsics.zNear = f32::max(params.intrinsics.zNear, 0.);
				changed |= params.intrinsics.zNear != params_orig.intrinsics.zNear;
				/* -- zFar --------------------------------------------------------- */
				// - define UI
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsminw);
					ui.label("zFar")
				});
				ui.add(
					egui::Slider::new(
						&mut params.intrinsics.zFar, 0.001..=f32::max(params_orig.intrinsics.zNear, 1024.)
					)
						.logarithmic(true)
						.drag_value_speed(0.03125*params_orig.intrinsics.zFar as f64)
						.clamping(egui::SliderClamping::Never)
				);
				ui.end_row();
				// - handle changes
				params.intrinsics.zFar = f32::max(params.intrinsics.zFar, 0.);
				changed |= params.intrinsics.zFar != params_orig.intrinsics.zFar;
			});
		});

		// Apply changes
		if changed {
			*camera.parameters_mut() = params;
		}
	}
}
