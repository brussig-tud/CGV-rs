
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

/// Helper enum to keep track of transient state required for multi-frame camera movements
enum ContinousRedrawing {
	Idle,
	WASD(&'static Player),
	FocusChange{focusChange: FocusChange, player: &'static Player},
}
impl ContinousRedrawing
{
	#[inline(always)]
	fn idle (&self) -> bool {
		if let ContinousRedrawing::Idle = self {true} else {false}
	}

	#[inline(always)]
	fn notChangingFocus (&self) -> bool {
		if let ContinousRedrawing::FocusChange {..} = self {false} else {true}
	}

	#[inline(always)]
	fn notMovingWASD (&self) -> bool {
		if let ContinousRedrawing::WASD(_) = self {false} else {true}
	}
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
	slowFactor: f32,
	referenceUp: glm::Vec3,
	moving: [bool; 6],
	slow: bool,
	continousRedrawing: ContinousRedrawing
}

impl WASDInteractor
{
	const FORE: usize = 0;
	const LEFT: usize = 1;
	const BACK: usize = 2;
	const RIGHT: usize = 3;
	const DOWN: usize = 4;
	const UP: usize = 5;

	pub fn new () -> Self { Self {
		dragSensitivity: 1./3.,
		movementSpeedFactor: 1.,
		slowFactor: 0.25,
		referenceUp: glm::vec3(0., 1., 0.),
		moving: [false; 6],
		slow: false,
		continousRedrawing: ContinousRedrawing::Idle
	}}

	#[inline(always)]
	fn anyMovementKeyPressed (&self) -> bool {
		self.moving.iter().any(|state| *state)
	}

	#[inline(always)]
	fn noMovementKeyPressed (&self) -> bool {
		self.moving.iter().all(|state| !state)
	}
}

impl CameraInteractor for WASDInteractor
{
	fn title (&self) -> &str {
		"WASD"
	}

	fn update (&mut self, camera: &mut dyn Camera, player: &Player)
	{
		/// Local helper to calculate the actual movement speed
		#[inline(always)] fn moveFactor (this: &WASDInteractor) -> f32 {
			if this.slow { this.slowFactor * this.movementSpeedFactor } else { this.movementSpeedFactor }
		}

		// We only borrow the camera parameters inside a scope where we're sure we'll be changing
		// something, as the camera usually recalculates internal state after a mutable borrow
		if self.anyMovementKeyPressed() {
			let params = camera.parameters_mut();
			let moveDist = player.lastFrameTime()*moveFactor(self)*params.intrinsics.f;
			if self.moving[Self::FORE] {
				params.extrinsics.eye += params.extrinsics.dir * moveDist;
			}
			if self.moving[Self::LEFT] {
				let right = params.extrinsics.dir.cross(&params.extrinsics.up);
				params.extrinsics.eye -= right * moveDist;
			}
			if self.moving[Self::BACK] {
				params.extrinsics.eye -= params.extrinsics.dir * moveDist;
			}
			if self.moving[Self::RIGHT] {
				let right = params.extrinsics.dir.cross(&params.extrinsics.up);
				params.extrinsics.eye += right * moveDist;
			}
			if self.moving[Self::DOWN] {
				params.extrinsics.eye -= params.extrinsics.up * moveDist;
			}
			if self.moving[Self::UP] {
				params.extrinsics.eye += params.extrinsics.up * moveDist;
			}
		}
		else if let ContinousRedrawing::FocusChange{focusChange, player}
			= &mut self.continousRedrawing
		{
			if focusChange.update(player.lastFrameTime(), camera.parameters_mut()) {
				player.dropContinuousRedrawRequest();
				self.continousRedrawing = ContinousRedrawing::Idle;
			}
		}
	}

	fn input (&mut self, event: &InputEvent, camera: &mut dyn Camera, player: &'static Player) -> EventOutcome
	{
		// Helper function for setting the movement key flags
		fn updateKeyFlag (this: &mut WASDInteractor, directionId: usize, pressed: bool, player: &'static Player) -> bool
		{
			if pressed {
				this.moving[directionId] = true;
				if this.continousRedrawing.idle() {
					player.pushContinuousRedrawRequest();
					this.continousRedrawing = ContinousRedrawing::WASD(player);
				}
				false // we're in continous redraw anyways
			}
			else {
				this.moving[directionId] = false;
				if this.noMovementKeyPressed() {
					this.continousRedrawing = ContinousRedrawing::Idle;
					player.dropContinuousRedrawRequest();
				}
				false // we just stopped continuous redrawing, but we won't redraw one last time
			}
		}

		// Match on relevant events
		match event
		{
			InputEvent::Key(info)
			=> {
				// Handle slow modifier
				let noMoveOutcome = if self.slow != info.modifiers.shift {
					EventOutcome::HandledDontClose(false)
				} else {
					EventOutcome::NotHandled
				};
				self.slow = info.modifiers.shift;

				// React to movement keys
				if !info.repeat { match info.key
				{
					egui::Key::W
					=> EventOutcome::HandledExclusively(updateKeyFlag(self, Self::FORE, info.pressed, player)),

					egui::Key::A
					=> EventOutcome::HandledExclusively(updateKeyFlag(self, Self::LEFT, info.pressed, player)),

					egui::Key::S
					=> EventOutcome::HandledExclusively(updateKeyFlag(self, Self::BACK, info.pressed, player)),

					egui::Key::D
					=> EventOutcome::HandledExclusively(updateKeyFlag(self, Self::RIGHT, info.pressed, player)),

					egui::Key::Q
					=> EventOutcome::HandledExclusively(updateKeyFlag(self, Self::DOWN, info.pressed, player)),

					egui::Key::E
					=> EventOutcome::HandledExclusively(updateKeyFlag(self, Self::UP, info.pressed, player)),

					_ => noMoveOutcome
				}}
				else {
					noMoveOutcome
				}
			},

			InputEvent::Dragged(info)
			=> {
				// Adapt dragged delta
				let delta = glm::vec2(-info.direction.x, info.direction.y);

				// We only borrow the camera parameters inside a scope where we're sure we'll be changing something,
				// as the camera usually recalculates internal state after a mutable borrow
				if info.button(egui::PointerButton::Primary) && info.modifiers.shift
				{
					let p = camera.parameters_mut();
					self.referenceUp = glm::rotate_vec3(
						&p.extrinsics.up, math::deg2rad!(delta.y*-0.75*self.dragSensitivity),
						&p.extrinsics.dir
					);
					p.extrinsics.up = self.referenceUp;
					EventOutcome::HandledExclusively(/* redraw */true)
				}
				else if info.button(egui::PointerButton::Secondary)
				{
					let p = camera.parameters_mut();
					let mut newDir = glm::rotate_vec3(
						&p.extrinsics.dir, math::deg2rad!(delta.x*self.dragSensitivity), &self.referenceUp
					);
					let right = glm::normalize(&glm::cross(&newDir, &self.referenceUp));
					newDir = glm::rotate_vec3(
						&newDir, math::deg2rad!(delta.y*-self.dragSensitivity), &right
					);
					p.extrinsics.dir = newDir;
					p.extrinsics.up = glm::cross(&right, &p.extrinsics.dir);
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
				if self.continousRedrawing.notMovingWASD()
				{
					let this = util::mutify(self);
					let mut focusChange = FocusChange::new(camera.parameters(), 0.5);
					player.unprojectPointAtSurfacePixel_async(info.position, move |point| {
						if let Some(point) = point {
							tracing::debug!("Double-click to new focus: {:?}", point);
							if this.continousRedrawing.notChangingFocus() {
								// Re-use the ongoing continuous redraw request if a focus change was in progress before
								player.pushContinuousRedrawRequest();
							}
							focusChange.setNewFocus(point);
							this.continousRedrawing = ContinousRedrawing::FocusChange{focusChange, player};
						}
					});
					EventOutcome::HandledExclusively(/* redraw */true)
				}
				else {
					EventOutcome::NotHandled
				}
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
				/* -- Slow movement multiplier -------------------------------------- */
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.set_min_width(lhsminw);
					ui.label("slow-key mult.")
				});
				ui.add(egui::Slider::new(&mut self.slowFactor, 0.03125..=0.75)
					.clamping(egui::SliderClamping::Never)
				);
				self.slowFactor = self.slowFactor.max(0.03125); // sanitize
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
		match self.continousRedrawing {
			ContinousRedrawing::WASD(player) => player.dropContinuousRedrawRequest(),
			ContinousRedrawing::FocusChange{player, .. } => player.dropContinuousRedrawRequest(),
			ContinousRedrawing::Idle => {}
		}
	}
}
