
//////
//
// Imports
//

// Standard library
/* nothing here yet */

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

/// Small helper struct for storing stuff we need when animation focus changes
struct FocusChangeContext<'player> {
	pub fc: view::FocusChange,
	pub player: &'player Player
}



//////
//
// Classes
//

////
// OrbitCamera

/// A camera interactor for orbital movement around a focal point.
pub struct OrbitInteractor {
	dragSensitivity: f32,
	fixUp: Option<glm::Vec3>,
	focusChange: Option<FocusChangeContext<'static>>
}

impl OrbitInteractor {
	pub fn new () -> Self { Self {
		dragSensitivity: 1./3.,
		fixUp: None,
		focusChange: None
	}}
}

impl CameraInteractor for OrbitInteractor
{
	fn title (&self) -> &str {
		"Orbit"
	}

	fn update (&mut self, camera: &mut dyn Camera, player: &Player)
	{
		if let Some(focusChange) = &mut self.focusChange {
			if focusChange.fc.update(player.lastFrameTime(), camera.parameters_mut()) {
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
			InputEvent::Dragged(info)
			=> {
				// Adapt dragged delta
				let delta = glm::vec2(-info.direction.x, info.direction.y);
				let mut handled= false;

				// We only borrow the camera parameters inside a scope where we're sure we'll be changing something,
				// as the camera usually recalculates internal state after a mutable borrow
				if info.button(egui::PointerButton::Primary)
				{
					let p = camera.parameters_mut();
					if info.modifiers.shift && self.fixUp.is_none() {
						p.extrinsics.up = glm::rotate_vec3(
							&p.extrinsics.up, math::deg2rad!(delta.y*-0.75*self.dragSensitivity),
							&p.extrinsics.dir
						);
					}
					else if let Some(upAxis) = &self.fixUp
					{
						let target = p.extrinsics.eye + p.intrinsics.f*p.extrinsics.dir;
						let mut newDir = glm::rotate_vec3(
							&p.extrinsics.dir, math::deg2rad!(delta.x*self.dragSensitivity), &upAxis
						);
						let right = glm::normalize(&glm::cross(&newDir, &upAxis));
						newDir = glm::rotate_vec3(
							&newDir, math::deg2rad!(delta.y*-self.dragSensitivity), &right
						);
						p.extrinsics.dir = newDir;
						p.extrinsics.up = glm::cross(&right, &p.extrinsics.dir);
						p.extrinsics.eye = target - p.intrinsics.f*p.extrinsics.dir;
					}
					else {
						let target = p.extrinsics.eye + p.intrinsics.f*p.extrinsics.dir;
						let mut right = glm::normalize(&glm::cross(&p.extrinsics.dir, &p.extrinsics.up));
						right = glm::rotate_vec3(
							&right, math::deg2rad!(delta.x*self.dragSensitivity), &p.extrinsics.up
						);
						p.extrinsics.up = glm::rotate_vec3(
							&p.extrinsics.up, math::deg2rad!(delta.y*-self.dragSensitivity), &right
						);
						p.extrinsics.dir = glm::cross(&p.extrinsics.up, &right);
						p.extrinsics.eye = target - p.intrinsics.f*p.extrinsics.dir;
					}
					handled = true;
				}
				if info.button(egui::PointerButton::Secondary)
				{
					let p = camera.parameters_mut();
					let speed = p.intrinsics.f * delta*1./192.*self.dragSensitivity;
					let right = &glm::cross(&p.extrinsics.dir, &p.extrinsics.up);
					let diff = speed.x*right + speed.y*p.extrinsics.up;
					p.extrinsics.eye += diff;
					if self.focusChange.is_some() {
						self.focusChange = None;
						player.dropContinuousRedrawRequest();
					}
					handled = true;
				}
				if info.button(egui::PointerButton::Middle)
				{
					let p = camera.parameters_mut();
					let movement = p.intrinsics.f*delta.y*1./96.*self.dragSensitivity * p.extrinsics.dir;
					p.extrinsics.eye += movement;
					if self.focusChange.is_some() {
						self.focusChange = None;
						player.dropContinuousRedrawRequest();
					}
					handled = true;
				}
				if handled {
					EventOutcome::HandledExclusively(/* redraw */true)
				} else {
					EventOutcome::NotHandled
				}
			},

			InputEvent::MouseWheel(info)
			=> {
				if info.amount.y != 0. {
					if info.modifiers.alt {
						let params = camera.parameters_mut();
						params.adjustFovBy(info.amount.y, math::deg2rad!(5.));
					} else {
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
				let mut focusChange = FocusChange::new(camera.parameters(), 0.5);
				player.unprojectPointAtSurfacePixel_async(info.position, move |point| {
					if let Some(point) = point {
						tracing::debug!("Double-click to new focus: {:?}", point);
						player.pushContinuousRedrawRequest();
						focusChange.setNewFocus(point);
						this.focusChange = Some(FocusChangeContext{fc: focusChange, player});
					}
				});
				EventOutcome::HandledExclusively(/* redraw */true)
			},

			_ => EventOutcome::NotHandled
		}
	}

	fn ui (&mut self, assignedCamera: &mut dyn Camera, ui: &mut egui::Ui)
	{
		// Put the UI inside a standard ControlTable
		gui::layout::ControlTableLayouter::new(ui).layout(ui, "CGV__orbint", |orbitUi|
		{
			// fixUp
			let mut fix = self.fixUp.is_some();
			if orbitUi.add("orbit", |ui, _|
				ui.add(egui::Checkbox::new(&mut fix, "fix up direction"))
			).changed() {
				self.fixUp = fix.then_some(assignedCamera.parameters().extrinsics.up);
			}

			// Action: reset up direction
			if orbitUi.add("", |ui, _| ui.add(
				egui::Button::new("reset up direction")
			)).clicked() {
				let fixedUp = glm::vec3(0., 1., 0.);
				let params = assignedCamera.parameters_mut();
				let right = params.extrinsics.dir.cross(&fixedUp);
				params.extrinsics.up = right.cross(&params.extrinsics.dir).normalize();
				if let Some(upAxis) = &mut self.fixUp {
					*upAxis = fixedUp;
				};
			}

			// dragSensitivity
			orbitUi.add("drag sensitivity", |ui, _| ui.add(
				egui::Slider::new(&mut self.dragSensitivity, 0.03125..=2.)
					.clamping(egui::SliderClamping::Always)
			));
		});
	}
}

impl Drop for OrbitInteractor {
	fn drop (&mut self) {
		// Make sure we let go of our continuous redraw request
		if let Some(focusChange) = &self.focusChange {
			focusChange.player.dropContinuousRedrawRequest();
		}
	}
}
