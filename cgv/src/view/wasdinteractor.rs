
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
use view::*;
use util::math;



//////
//
// Structs
//

struct FocusChange {
	pub old: glm::Vec3,
	pub new: glm::Vec3,
	pub prev: glm::Vec3,
	pub t: f32,
	pub player: &'static Player
}



//////
//
// Classes
//

////
// OrbitCamera

/// A camera interactor for orbital movement around a focal point.
pub struct WASDInteractor {
	dragSensitivity: f32,
	movementSpeedFactor: f32,
	referenceUp: glm::Vec3,
	focusChange: Option<FocusChange>,
	pressedW: bool,
	pressedA: bool,
	pressedS: bool,
	pressedD: bool
}

impl WASDInteractor {
	pub fn new () -> Self { Self {
		dragSensitivity: 1./3.,
		movementSpeedFactor: 1.,
		referenceUp: glm::vec3(0., 1., 0.),
		focusChange: None,
		pressedW: false,
		pressedA: false,
		pressedS: false,
		pressedD: false,
	}}
}

impl CameraInteractor for WASDInteractor
{
	fn title (&self) -> &str {
		"WASD"
	}

	fn update (&mut self, camera: &mut dyn Camera, player: &Player)
	{
		if self.pressedW {
			// We only borrow the camera parameters inside a scope where we're sure we'll be changing
			// something, as the camera usually recalculates internal state after a mutable borrow
			let params = camera.parameters_mut();
			params.extrinsics.eye +=
				params.extrinsics.dir * self.movementSpeedFactor*params.intrinsics.f*player.lastFrameTime();
		}
		if self.pressedA {
			// We only borrow the camera parameters inside a scope where we're sure we'll be changing
			// something, as the camera usually recalculates internal state after a mutable borrow
			let params = camera.parameters_mut();
			let right = params.extrinsics.dir.cross(&params.extrinsics.up).normalize();
			params.extrinsics.eye -=
				right * self.movementSpeedFactor*params.intrinsics.f*player.lastFrameTime();
		}
		if self.pressedS {
			// We only borrow the camera parameters inside a scope where we're sure we'll be changing
			// something, as the camera usually recalculates internal state after a mutable borrow
			let params = camera.parameters_mut();
			params.extrinsics.eye -=
				params.extrinsics.dir * self.movementSpeedFactor*params.intrinsics.f*player.lastFrameTime();
		}
		if self.pressedD {
			// We only borrow the camera parameters inside a scope where we're sure we'll be changing
			// something, as the camera usually recalculates internal state after a mutable borrow
			let params = camera.parameters_mut();
			let right = params.extrinsics.dir.cross(&params.extrinsics.up).normalize();
			params.extrinsics.eye +=
				right * self.movementSpeedFactor*params.intrinsics.f*player.lastFrameTime();
		}
		else if let Some(focusChange) = &mut self.focusChange
		{
			let extr = &mut camera.parameters_mut().extrinsics;
			focusChange.t = f32::min(focusChange.t + player.lastFrameTime()*2f32, 1f32);
			let focusCur = math::smoothLerp3(&focusChange.old, &focusChange.new, focusChange.t);
			let offset = focusCur - focusChange.prev;
			focusChange.prev = focusCur;
			extr.eye += offset;
			if focusCur == focusChange.new {
				self.focusChange = None;
				player.dropContinuousRedrawRequest();
			}
		}
	}

	fn input (&mut self, event: &InputEvent, camera: &mut dyn Camera, player: &'static Player) -> EventOutcome
	{
		// Match on relevant events
		match event
		{
			InputEvent::Key(info)
			=> {
				if !info.repeat { match info.key
				{
					egui::Key::W => {
						if info.pressed {
							self.pressedW = true;
							player.pushContinuousRedrawRequest();
						}
						else {
							self.pressedW = false;
							player.dropContinuousRedrawRequest();
						}
						EventOutcome::HandledExclusively(/* redraw */true)
					},
					egui::Key::A => {
						if info.pressed {
							self.pressedA = true;
							player.pushContinuousRedrawRequest();
						}
						else {
							self.pressedA = false;
							player.dropContinuousRedrawRequest();
						}
						EventOutcome::HandledExclusively(/* redraw */true)
					},
					egui::Key::S => {
						if info.pressed {
							self.pressedS = true;
							player.pushContinuousRedrawRequest();
						}
						else {
							self.pressedS = false;
							player.dropContinuousRedrawRequest();
						}
						EventOutcome::HandledExclusively(/* redraw */true)
					},
					egui::Key::D => {
						if info.pressed {
							self.pressedD = true;
							player.pushContinuousRedrawRequest();
						}
						else {
							self.pressedD = false;
							player.dropContinuousRedrawRequest();
						}
						EventOutcome::HandledExclusively(/* redraw */true)
					},

					_ => EventOutcome::NotHandled
				}}
				else {
					EventOutcome::NotHandled
				}
			},

			InputEvent::Dragged(info)
			=> {
				if info.button(egui::PointerButton::Secondary)
				{
					let delta = glm::vec2(-info.direction.x, info.direction.y);
					// We only borrow the camera parameters inside a scope where we're sure we'll be changing something,
					// as the camera usually recalculates internal state after a mutable borrow
					let p = camera.parameters_mut();
					if info.modifiers.shift {
						self.referenceUp = glm::rotate_vec3(
							&p.extrinsics.up, math::deg2rad!(delta.y*-0.75*self.dragSensitivity),
							&p.extrinsics.dir
						);
						p.extrinsics.up = self.referenceUp;
					}
					else
					{
						let mut newDir = glm::rotate_vec3(
							&p.extrinsics.dir, math::deg2rad!(delta.x*self.dragSensitivity), &self.referenceUp
						);
						let right = glm::normalize(&glm::cross(&newDir, &self.referenceUp));
						newDir = glm::rotate_vec3(
							&newDir, math::deg2rad!(delta.y*-self.dragSensitivity), &right
						);
						p.extrinsics.dir = newDir;
						p.extrinsics.up = glm::cross(&right, &p.extrinsics.dir);
					}
					EventOutcome::HandledExclusively(/* redraw */true)
				}
				else {
					EventOutcome::NotHandled
				}
			},

			InputEvent::MouseWheel(info)
			=> {
				if info.amount.y != 0. {
					if info.modifiers.alt {
						// We only borrow the camera parameters inside a scope where we're sure we'll be changing
						// something, as the camera usually recalculates internal state after a mutable borrow
						let params = camera.parameters_mut();
						params.adjustFovBy(info.amount.y, math::deg2rad!(5.));
					} else {
						// We only borrow the camera parameters inside a scope where we're sure we'll be changing
						// something, as the camera usually recalculates internal state after a mutable borrow
						let params = camera.parameters_mut();
						params.adjustZoom(info.amount.y);
					}
					EventOutcome::HandledExclusively(/* redraw */true)
				}
				else {
					EventOutcome::NotHandled
				}
			},

			InputEvent::DoubleClick(info) => {
				let this = util::mutify(self);
				let (extr, f) = {
					let params = camera.parameters();
					(params.extrinsics.clone(), params.intrinsics.f)
				};
				player.unprojectPointAtSurfacePixel_async(info.position, move |point| {
					if let Some(point) = point {
						tracing::debug!("Double-click to new focus: {:?}", point);
						let old = extr.eye + f*extr.dir;
						this.focusChange = Some(FocusChange {old, new: *point, prev: old, t: 0., player});
						player.pushContinuousRedrawRequest();
					}
				});
				EventOutcome::HandledExclusively(/* redraw */true)
			},

			_ => EventOutcome::NotHandled
		}
	}

	fn ui (&mut self, assignedCamera: &mut dyn Camera, ui: &mut egui::Ui)
	{
		// Layouting calculations
		let awidth = ui.available_width();
		let rhswidth = f32::max(192f32, awidth*1./2.);
		let lhsminw = f32::max(awidth-rhswidth - ui.spacing().item_spacing.x, 0.);

		// UI for compound settings
		ui.vertical(|ui| {
			ui.spacing_mut().slider_width = rhswidth-56.;
			//ui.label(egui::RichText::new("Compounds").underline());
			egui::Grid::new("CGV__orbint").num_columns(2).striped(true).show(ui, |ui| {
				/* -- Fix up direction ---------------------------------------------- */
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsminw);
					ui.label("orientation")
				});
				if ui.add(egui::Button::new("reset up direction")).clicked() {
					self.referenceUp = glm::vec3(0., 1., 0.);
					let params = assignedCamera.parameters_mut();
					let right = params.extrinsics.dir.cross(&self.referenceUp);
					params.extrinsics.up = right.cross(&params.extrinsics.dir).normalize();
				};
				ui.end_row();
				/* -- Movement speed factor ----------------------------------------- */
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsminw);
					ui.label("speed mult.")
				});
				ui.add(egui::Slider::new(&mut self.movementSpeedFactor, 0.03125..=4.)
					.clamping(egui::SliderClamping::Never)
				);
				self.movementSpeedFactor = self.movementSpeedFactor.max(0.03125); // sanitize
				ui.end_row();
				/* -- Drag sensitivity ---------------------------------------------- */
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsminw);
					ui.label("drag sensitivity")
				});
				ui.add(egui::Slider::new(&mut self.dragSensitivity, 0.03125..=2.)
					.clamping(egui::SliderClamping::Always)
				);
				ui.end_row();
			})
		});
	}
}

impl Drop for WASDInteractor {
	fn drop (&mut self) {
		// Make sure we let go of our continuous redraw request
		if let Some(focusChange) = &self.focusChange {
			focusChange.player.dropContinuousRedrawRequest();
		}
	}
}
