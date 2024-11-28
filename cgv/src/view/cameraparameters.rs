
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
pub struct Intrinsics
{
	pub fovY: FoV,
	pub aspect: f32,
	pub f: f32,
	pub zNear: f32,
	pub zFar: f32
}
impl Intrinsics
{
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
		(0.5*diameter)/f32::tan(0.5*fov)
	}

	pub fn uiWithSizes (&mut self, lhsMinWidth: f32, rhsWidth: f32, ui: &mut egui::Ui)
	{
		// Create UI
		ui.vertical(|ui| {
			ui.spacing_mut().slider_width = rhsWidth - 56.;
			ui.label(egui::RichText::new("Intrinsics").underline());
			let mut ortho = self.fovY.isOrthographic();
			egui::Grid::new("CGV__cam_intr").num_columns(2).striped(true).show(ui, |ui| {
				/* -- perspective/orthographic -------------------------------------- */
				// - define UI
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsMinWidth);
					ui.label("projection");
				});
				ui.add(egui::Checkbox::new(&mut ortho, "orthographic"));
				ui.end_row();
				// - sanitize
				if ortho != self.fovY.isOrthographic() {
					self.fovY = match self.fovY {
						FoV::Perspective(fovY)
						=> FoV::Orthographic(Intrinsics::frustumDiameterAtFocus(fovY, self.f)),

						FoV::Orthographic(height) => FoV::Perspective(
							Intrinsics::angleForFrustumDiameterAndFocus(height, self.f)
						)
					};
				}
				/* -- FoV ----------------------------------------------------------- */
				// - define UI
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsMinWidth);
					ui.label("FoV (Y)")
				});
				match self.fovY
				{
					FoV::Perspective(old) => {
						let old = math::rad2deg!(old);
						let mut new = old;
						ui.add(egui::Slider::new(&mut new, 1f32..=179.)
							.drag_value_speed(0.03125*old as f64)
							.clamping(egui::SliderClamping::Never)
						);
						if new != old {
							self.fovY = FoV::Perspective(math::deg2rad!(new));
						}
					},
					FoV::Orthographic(old) => {
						let mut new = old;
						ui.add(egui::Slider::new(&mut new, 0.1..=100.)
							.logarithmic(true)
							.drag_value_speed(0.03125*old as f64)
							.clamping(egui::SliderClamping::Never)
						);
						if new != old {
							self.fovY = FoV::Orthographic(new);
						}
					}
				}
				ui.end_row();
				// - sanitize
				self.f = f32::max(self.f, 0.);
				/* -- f ------------------------------------------------------------- */
				// - define UI
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsMinWidth);
					ui.label("focus distance")
				});
				let tmp = self.f;
				ui.add(egui::Slider::new(&mut self.f, self.zNear..=self.zFar)
					.drag_value_speed(0.03125*tmp as f64)
					.clamping(egui::SliderClamping::Never)
				);
				ui.end_row();
				// - sanitize
				self.f = f32::max(self.f, 0.);
				/* -- zNear --------------------------------------------------------- */
				// - define UI
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsMinWidth);
					ui.label("zNear")
				});
				let tmp = self.zNear;
				ui.add(egui::Slider::new(&mut self.zNear, 0.0001..=f32::min(10., self.zFar))
					.logarithmic(true)
					.drag_value_speed(0.03125*tmp as f64)
					.clamping(egui::SliderClamping::Never)
				);
				ui.end_row();
				// - sanitize
				self.zNear = f32::max(self.zNear, 0.);
				/* -- zFar --------------------------------------------------------- */
				// - define UI
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsMinWidth);
					ui.label("zFar")
				});
				let tmp = self.zFar;
				ui.add(egui::Slider::new(&mut self.zFar, 0.001..=f32::max(self.zNear, 1024.))
					.logarithmic(true)
					.drag_value_speed(0.03125*tmp as f64)
					.clamping(egui::SliderClamping::Never)
				);
				ui.end_row();
				// - sanitize
				self.zFar = f32::max(self.zFar, 0.);
			});
		});
	}

	pub fn ui (&mut self, ui: &mut egui::Ui)
	{
		// Layouting calculations
		let awidth = ui.available_width();
		let rhswidth = f32::max(192f32, awidth*1./2.);
		let lhsminw = f32::max(awidth - rhswidth - ui.spacing().item_spacing.x, 0.);

		// Create UI
		self.uiWithSizes(lhsminw, rhswidth, ui);
	}
}
impl PartialEq for Intrinsics
{
	fn eq (&self, other: &Self) -> bool {
		   self.fovY == other.fovY && self.aspect == other.aspect && self.f == other.f && self.zNear == other.zNear
		&& self.zFar == other.zFar
	}

	fn ne (&self, other: &Self) -> bool {
		   self.fovY != other.fovY || self.aspect != other.aspect || self.f != other.f || self.zNear != other.zNear
		|| self.zFar != other.zFar
	}
}

#[derive(Clone, Copy)]
pub struct Extrinsics {
	pub eye: glm::Vec3,
	pub dir: glm::Vec3,
	pub up: glm::Vec3,
}
impl Extrinsics
{
	pub fn uiWithSizes (&mut self, lhsMinWidth: f32, #[allow(unused_variables)]rhsWidth: f32, ui: &mut egui::Ui)
	{
		// Create UI
		ui.vertical(|ui| {
			ui.label(egui::RichText::new("Extrinsics").underline());
			egui::Grid::new("CGV__cam_extr").num_columns(4).striped(true).show(ui, |ui| {
				/* -- eye ----------------------------------------------------------- */
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsMinWidth);
					ui.label("eye point")
				});
				util::widgets::vec3_sized(ui, &mut self.eye, rhsWidth);
				ui.end_row();
				/* -- dir ----------------------------------------------------------- */
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsMinWidth);
					ui.label("direction")
				});
				if util::widgets::vec3_sized(ui, &mut self.dir, rhsWidth) {
					if self.dir.norm_squared() == 0. {
						self.dir.z = -1.;
					}
					else {
						self.dir.normalize_mut();
					}
					self.up = glm::cross(&self.dir, &self.up).cross(&self.dir).normalize();
				};
				ui.end_row();
				/* -- up ------------------------------------------------------------ */
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsMinWidth);
					ui.label("up direction")
				});
				if util::widgets::vec3_sized(ui, &mut self.up, rhsWidth) {
					if self.up.norm_squared() == 0. {
						self.up.y = 1.;
					}
					else {
						self.up.normalize_mut();
					}
					self.dir = glm::cross(&self.up, &self.dir).cross(&self.up).normalize();
				}
				ui.end_row();
			});
		});
	}

	pub fn ui (&mut self, ui: &mut egui::Ui)
	{
		// Layouting calculations
		let awidth = ui.available_width();
		let rhswidth = f32::max(192f32, awidth*1./2.);
		let lhsminw = f32::max(awidth - rhswidth - ui.spacing().item_spacing.x, 0.);

		// Create UI
		self.uiWithSizes(lhsminw, rhswidth, ui);
	}
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
impl PartialEq for Extrinsics
{
	fn eq (&self, other: &Self) -> bool {
		self.eye == other.eye && self.dir == other.dir && self.up == other.up
	}

	fn ne (&self, other: &Self) -> bool {
		self.eye != other.eye || self.dir != other.dir || self.up != other.up
	}
}

#[derive(Clone, Copy)]
pub struct CameraParameters {
	pub intrinsics: Intrinsics,
	pub extrinsics: Extrinsics,
}
impl CameraParameters
{
	pub fn defaultWithAspect (aspect: f32) -> Self { Self {
		intrinsics: Intrinsics::defaultWithAspect(aspect),
		extrinsics: Default::default()
	}}

	pub fn adjustZoom (&mut self, amount: f32) {
		let focus = self.extrinsics.eye + self.extrinsics.dir*self.intrinsics.f;
		self.intrinsics.f = self.intrinsics.f * (1. + amount*-1./256.);
		self.extrinsics.eye = focus - self.extrinsics.dir*self.intrinsics.f;
	}

	pub fn adjustFovTo (&mut self, newFov: f32, orthoThreshold: f32)
	{
		if let FoV::Perspective(fov) = self.intrinsics.fovY
		{
			let dia = Intrinsics::frustumDiameterAtFocus(fov, self.intrinsics.f);
			if newFov <= orthoThreshold {
				self.intrinsics.fovY = FoV::Orthographic(dia)
			}
			else {
				let focusOld = self.extrinsics.eye + self.extrinsics.dir*self.intrinsics.f;
				self.intrinsics.fovY = FoV::Perspective(newFov);
				self.intrinsics.f = Intrinsics::focusDistForFrustumDiameterAndFov(dia, newFov);
				self.extrinsics.eye = focusOld - self.extrinsics.dir*self.intrinsics.f;
			}
		}
		else if let FoV::Orthographic(dia) = self.intrinsics.fovY && newFov > orthoThreshold {
			let focusOld = self.extrinsics.eye + self.extrinsics.dir*self.intrinsics.f;
			self.intrinsics.fovY = FoV::Perspective(orthoThreshold);
			self.intrinsics.f = Intrinsics::focusDistForFrustumDiameterAndFov(dia, orthoThreshold);
			self.extrinsics.eye = focusOld - self.extrinsics.dir*self.intrinsics.f;
		}
	}

	pub fn adjustFovBy (&mut self, amount: f32, orthoThreshold: f32)
	{
		if let FoV::Perspective(fov) = self.intrinsics.fovY
		{
			let dia = Intrinsics::frustumDiameterAtFocus(fov, self.intrinsics.f);
			let newFov = f32::min(fov + math::deg2rad!(amount*0.125), 179.);
			if newFov <= orthoThreshold {
				self.intrinsics.fovY = FoV::Orthographic(dia)
			}
			else {
				let focusOld = self.extrinsics.eye + self.extrinsics.dir*self.intrinsics.f;
				self.intrinsics.fovY = FoV::Perspective(newFov);
				self.intrinsics.f = Intrinsics::focusDistForFrustumDiameterAndFov(dia, newFov);
				self.extrinsics.eye = focusOld - self.extrinsics.dir*self.intrinsics.f;
			}
		}
		else if let FoV::Orthographic(dia) = self.intrinsics.fovY && amount > 0. {
			let focusOld = self.extrinsics.eye + self.extrinsics.dir*self.intrinsics.f;
			self.intrinsics.fovY = FoV::Perspective(orthoThreshold);
			self.intrinsics.f = Intrinsics::focusDistForFrustumDiameterAndFov(dia, orthoThreshold);
			self.extrinsics.eye = focusOld - self.extrinsics.dir*self.intrinsics.f;
		}
	}

	pub fn ui (camera: &mut dyn Camera, ui: &mut egui::Ui)
	{
		// Track camera parameters wrt. to current values
		let params_orig = camera.parameters();
		let mut params = params_orig.clone();
		let mut changed = false;

		// Layouting calculations
		let awidth = ui.available_width();
		let rhswidth = f32::max(192f32, awidth*1./2.);
		let lhsminw = f32::max(awidth-rhswidth - ui.spacing().item_spacing.x, 0.);

		// UI for compound settings
		ui.vertical(|ui| {
			ui.spacing_mut().slider_width = rhswidth-56.;
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
				/* -- Vertigo (affects f, fov, eye) --------------------------------- */
				// - define UI
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsminw);
					ui.label("vertigo")
				});
				let mut fov = if let FoV::Perspective(fov) = params.intrinsics.fovY {
					math::rad2deg!(fov)
				} else { 5. };
				let tmp = fov;
				if ui.add(egui::Slider::new(&mut fov, 5f32..=179.)
					.drag_value_speed(0.03125*tmp as f64)
					.clamping(egui::SliderClamping::Always)
				).changed() {
					params.adjustFovTo(math::deg2rad!(fov), math::deg2rad!(5.));
					changed = true;
				};
				ui.end_row();
			});
		});

		// UI for intrinsics
		params.intrinsics.uiWithSizes(lhsminw, rhswidth, ui);
		changed |= params.intrinsics != params_orig.intrinsics;

		// UI for extrinsics
		params.extrinsics.uiWithSizes(lhsminw, rhswidth, ui);
		changed |= params.extrinsics != params_orig.extrinsics;

		// Apply changes
		if changed {
			*camera.parameters_mut() = params;
		}
	}
}
