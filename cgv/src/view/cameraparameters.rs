
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Egui library
use egui;

// CGV-rs components
use cgv_util::math;

// Local imports
use crate::{*, view::*};



//////
//
// Module-wide constants
//

/// The threshold angle, in degrees, at which the auto-generated camera parameter GUIs will switch between orthogonal
/// and perspective projections.
const FOV_ORTHO_THRESHOLD: f32 = 5.;



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

	pub fn ui (&mut self, ui: &mut egui::Ui) {
		// Create ControlTable layouter and delegate the actual work
		self.uiWithLayouter(ui, &gui::layout::ControlTableLayouter::new(ui));
	}

	pub fn uiWithLayouter (&mut self, ui: &mut egui::Ui, layouter: &gui::layout::ControlTableLayouter)
	{
		// Header
		ui.label(egui::RichText::new("Intrinsics").underline());

		// Define the GUI elements
		layouter.layout(ui, "CGV__cam_intr", |intrinsicsUi|
		{
			// Perspective/orthographic
			let mut ortho = self.fovY.isOrthographic();
			if intrinsicsUi.add("projection", |ui, _| ui.add(
				egui::Checkbox::new(&mut ortho, "orthographic")
			)).changed() {
				self.fovY = match self.fovY
				{
					FoV::Perspective(fovY)
					=> FoV::Orthographic(Intrinsics::frustumDiameterAtFocus(fovY, self.f)),

					FoV::Orthographic(height) => FoV::Perspective(
						Intrinsics::angleForFrustumDiameterAndFocus(height, self.f)
					)
				};
			}

			// fovY
			intrinsicsUi.add("FoV (Y)", |ui, _| match self.fovY {
				FoV::Perspective(old) => {
					let mut angle = math::rad2deg!(old);
					let response = ui.add(egui::Slider::new(&mut angle, 1f32..=179.)
						.drag_value_speed(0.03125*old as f64)
						.clamping(egui::SliderClamping::Never)
					);
					if response.changed() {
						self.fovY = FoV::Perspective(math::deg2rad!(angle));
					};
					response
				},
				FoV::Orthographic(old) => {
					let mut height = old;
					let response = ui.add(egui::Slider::new(&mut height, 0.1..=100.)
						.logarithmic(true)
						.drag_value_speed(0.03125*old as f64)
						.clamping(egui::SliderClamping::Never)
					);
					if response.changed() {
						self.fovY = FoV::Orthographic(height);
					};
					response
				}
			});

			// f
			let tmp = self.f;
			if intrinsicsUi.add("focus distance", |ui, _| ui.add(
				egui::Slider::new(&mut self.f, self.zNear..=self.zFar)
					.drag_value_speed(0.03125*tmp as f64)
					.clamping(egui::SliderClamping::Never)
			)).changed() {
				self.f = f32::max(self.f, 0.);
			}

			// zNear
			let tmp = self.zNear;
			intrinsicsUi.add("zNear", |ui, _| ui.add(
				egui::Slider::new(&mut self.zNear, 0.0001..=self.zFar-0.0001)
					.logarithmic(true)
					.drag_value_speed(0.03125*tmp as f64)
					.clamping(egui::SliderClamping::Always)
			));

			// zFar
			let tmp = self.zFar;
			intrinsicsUi.add("zFar", |ui, _| ui.add(
				egui::Slider::new(&mut self.zFar, self.zNear+0.0001..=1024.)
					.logarithmic(true)
					.drag_value_speed(0.03125*tmp as f64)
					.clamping(egui::SliderClamping::Always)
			));
		});
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

	pub fn ui (&mut self, ui: &mut egui::Ui) {
		// Create ControlTable layouter and delegate the actual work
		self.uiWithLayouter(ui, &gui::layout::ControlTableLayouter::new(ui));
	}

	pub fn uiWithLayouter (&mut self, ui: &mut egui::Ui, layouter: &gui::layout::ControlTableLayouter)
	{
		// Header
		ui.label(egui::RichText::new("Extrinsics").underline());

		// Define the GUI elements
		layouter.layout(ui, "CGV__cam_extr", |extrinsicsUi|
		{
			// eye
			extrinsicsUi.add("eye point", |ui, idealSize|
				gui::complex::vec3_sized(ui, &mut self.eye, idealSize)
			);

			// dir
			if extrinsicsUi.add("direction", |ui, idealSize|
				gui::complex::vec3_sized(ui, &mut self.dir, idealSize)
			){
				if self.dir.norm_squared() < 0.00001 {
					self.dir.z = -1.;
				}
				self.dir.normalize_mut();
				self.up = glm::cross(&self.dir, &self.up).cross(&self.dir).normalize();
			}

			// up
			if extrinsicsUi.add("up direction", |ui, idealSize|
				gui::complex::vec3_sized(ui, &mut self.up, idealSize)
			){
				if self.up.norm_squared() < 0.00001 {
					self.up.y = 1.;
				}
				self.up.normalize_mut();
				self.dir = glm::cross(&self.up, &self.dir).cross(&self.up).normalize();
			}
		});
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

	pub fn adjustForTargetFov (&mut self, newFov: f32, orthoThreshold: f32)
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
			let newFov = f32::min(fov + math::deg2rad!(amount*0.125), math::deg2rad!(179.));
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
		// Track changes to camera parameters
		let params_orig = camera.parameters();
		let mut params = params_orig.clone();
		let mut changed = false;

		// UI for compound settings
		let controlTable = ui.vertical(|ui|
		{
			// Header
			ui.label(egui::RichText::new("Compounds").underline());

			// UI section contents
			let controlTable = gui::layout::ControlTableLayouter::new(ui);
			controlTable.layout(ui, "CGV__cam_cmpd", |compoundsUi| {
				/* -- zoom (affects f, eye) ----------------------------------------- */
				compoundsUi.add("zoom", |ui, _| {
					if ui.add(
						egui::Slider::new(
							&mut params.intrinsics.f, params_orig.intrinsics.zNear..=params_orig.intrinsics.zFar
						)
						.drag_value_speed(0.03125 * params_orig.intrinsics.f as f64)
						.clamping(egui::SliderClamping::Never)
					).changed() {
						let focus = params.extrinsics.eye + params.extrinsics.dir*params_orig.intrinsics.f;
						params.extrinsics.eye = focus - params.extrinsics.dir*params.intrinsics.f;
						changed = true;
					}
				});
				/* -- vertigo (affects f, fov, eye) --------------------------------- */
				let mut fov = if let FoV::Perspective(fov) = params.intrinsics.fovY { math::rad2deg!(fov) }
				                     else                                                      { FOV_ORTHO_THRESHOLD };
				let fov_old = fov;
				compoundsUi.add("vertigo", |ui, _| {
					if ui.add(egui::Slider::new(&mut fov, 5f32..=179.)
						.drag_value_speed(0.03125*fov_old as f64)
						.clamping(egui::SliderClamping::Always)
					).changed() {
						params.adjustForTargetFov(
							math::deg2rad!(fov), math::deg2rad!(FOV_ORTHO_THRESHOLD)
						);
						changed = true;
					}
				});
			});
			controlTable
		}).inner;

		// UI for intrinsics
		params.intrinsics.uiWithLayouter(ui, &controlTable);
		changed |= params.intrinsics != params_orig.intrinsics;

		// UI for extrinsics
		params.extrinsics.uiWithLayouter(ui, &controlTable);
		changed |= params.extrinsics != params_orig.extrinsics;

		// Apply changes
		if changed {
			*camera.parameters_mut() = params;
		}
	}
}
