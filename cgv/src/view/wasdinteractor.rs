
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

/// Helper enum to keep track of transient state required for multi-frame camera movements
enum ContinuousRedrawing {
	Idle,
	WASD,
	FocusChange(FocusChange),
}
impl ContinuousRedrawing
{
	#[inline(always)]
	fn idle (&self) -> bool {
		if let Self::Idle = self {true} else {false}
	}

	#[inline(always)]
	fn notChangingFocus (&self) -> bool {
		if let Self::FocusChange {..} = self {false} else {true}
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
	continuousRedrawing: ContinuousRedrawing,
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
		continuousRedrawing: ContinuousRedrawing::Idle,
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

impl CameraInteractor<WASDInteractor> for CamIntObject<WASDInteractor>
{
	fn title (&self) -> &str {
		"WASD"
	}

	fn update (&mut self, player: &mut Player, _: player::CamIntHandle)
	{
		type This = WASDInteractor;

		/// Local helper to calculate the actual movement speed
		#[inline(always)] fn moveFactor (this: &WASDInteractor) -> f32 {
			if this.slow { this.slowFactor * this.movementSpeedFactor } else { this.movementSpeedFactor }
		}

		// We only borrow the camera parameters inside a scope where we're sure we'll be changing
		// something, as the camera usually recalculates internal state after a mutable borrow
		if self.anyMovementKeyPressed() {
			let params = player.camera.parameters_mut();
			let moveDist = player.state.lastFrameTime()*moveFactor(self)*params.intrinsics.f;
			if self.moving[This::FORE] {
				params.extrinsics.eye += params.extrinsics.dir * moveDist;
			}
			if self.moving[This::LEFT] {
				let right = params.extrinsics.dir.cross(&params.extrinsics.up);
				params.extrinsics.eye -= right * moveDist;
			}
			if self.moving[This::BACK] {
				params.extrinsics.eye -= params.extrinsics.dir * moveDist;
			}
			if self.moving[This::RIGHT] {
				let right = params.extrinsics.dir.cross(&params.extrinsics.up);
				params.extrinsics.eye += right * moveDist;
			}
			if self.moving[This::DOWN] {
				params.extrinsics.eye -= params.extrinsics.up * moveDist;
			}
			if self.moving[This::UP] {
				params.extrinsics.eye += params.extrinsics.up * moveDist;
			}
		}
		else if let ContinuousRedrawing::FocusChange(focusChange) = &mut self.continuousRedrawing {
			if focusChange.update(player.state.lastFrameTime(), player.camera.parameters_mut()) {
				player.dropContinuousRedrawRequest();
				self.continuousRedrawing = ContinuousRedrawing::Idle;
			}
		}
	}

	fn input (&mut self, event: &InputEvent, player: &mut Player, handle: player::CamIntHandle) -> EventOutcome
	{
		type This = WASDInteractor;

		// Helper function for setting the movement key flags
		fn updateKeyFlag (this: &mut WASDInteractor, directionId: usize, pressed: bool, player: &mut Player) -> bool
		{
			if pressed {
				this.moving[directionId] = true;
				if this.continuousRedrawing.idle() {
					player.pushContinuousRedrawRequest();
					this.continuousRedrawing = ContinuousRedrawing::WASD;
				}
				false // we're in continuous redraw anyways
			}
			else {
				this.moving[directionId] = false;
				if this.noMovementKeyPressed() {
					this.continuousRedrawing = ContinuousRedrawing::Idle;
					player.dropContinuousRedrawRequest();
				}
				false // we just stopped continuous redrawing, but we won't redraw one last time
			}
		}

		// Match on relevant events
		'event:{ match event
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
					=> EventOutcome::HandledExclusively(updateKeyFlag(self, This::FORE, info.pressed, player)),

					egui::Key::A
					=> EventOutcome::HandledExclusively(updateKeyFlag(self, This::LEFT, info.pressed, player)),

					egui::Key::S
					=> EventOutcome::HandledExclusively(updateKeyFlag(self, This::BACK, info.pressed, player)),

					egui::Key::D
					=> EventOutcome::HandledExclusively(updateKeyFlag(self, This::RIGHT, info.pressed, player)),

					egui::Key::Q
					=> EventOutcome::HandledExclusively(updateKeyFlag(self, This::DOWN, info.pressed, player)),

					egui::Key::E
					=> EventOutcome::HandledExclusively(updateKeyFlag(self, This::UP, info.pressed, player)),

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
					let p = player.camera.parameters_mut();
					self.referenceUp = glm::rotate_vec3(
						&p.extrinsics.up, math::deg2rad!(delta.y*-0.75*self.dragSensitivity),
						&p.extrinsics.dir
					);
					p.extrinsics.up = self.referenceUp;
					EventOutcome::HandledExclusively(/* redraw */true)
				}
				else if info.button(egui::PointerButton::Secondary)
				{
					let p = player.camera.parameters_mut();
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
						let params = player.camera.parameters_mut();
						params.adjustFovBy(info.amount.y, math::deg2rad!(5.));
					} else {
						// We only borrow the camera parameters inside a scope where we're sure we'll be changing
						// something, as the camera usually recalculates internal state after a mutable borrow
						let params = player.camera.parameters_mut();
						params.adjustZoom(info.amount.y);
					}
					EventOutcome::HandledExclusively(/* redraw */true)
				}
				else {
					EventOutcome::NotHandled
				}
			},

			InputEvent::DoubleClick(info)
			=> {
				if let ContinuousRedrawing::WASD = self.continuousRedrawing {break 'event EventOutcome::NotHandled}
				let mut focusChange = FocusChange::new(player.camera.parameters(), 0.5);
				player.unprojectPointAtSurfacePixel_async(info.position, move |point| {
					let Some(point) = point else {return};

					tracing::debug!("Double-click to new focus: {:?}", point);
					let mut lock = player::lock();
					let player = &mut*lock;
					let this = player.cameraInteractors.get_mut::<Self>(handle);

					if this.continuousRedrawing.notChangingFocus() {
						// Re-use the ongoing continuous redraw request if a focus change was in progress before
						player.state.pushContinuousRedrawRequest();
					}
					focusChange.setNewFocus(point);
					this.continuousRedrawing = ContinuousRedrawing::FocusChange(focusChange);
				});
				EventOutcome::HandledExclusively(/* redraw */true)
			},

			_ => EventOutcome::NotHandled
		}}
	}

	fn ui (&mut self, assignedCamera: &mut dyn Camera, ui: &mut egui::Ui)
	{
		// Put the UI inside a standard ControlTable
		gui::layout::ControlTableLayouter::new(ui).layout(ui, "CGV__wasdint", |wasdUi|
		{
			// Action: reset up direction
			if wasdUi.add("orientation", |ui, _| ui.add(
				egui::Button::new("reset up direction")
			)).clicked() {
				self.referenceUp = glm::vec3(0., 1., 0.);
				let params = assignedCamera.parameters_mut();
				let right = params.extrinsics.dir.cross(&self.referenceUp);
				params.extrinsics.up = right.cross(&params.extrinsics.dir).normalize();
			}

			// movementSpeedFactor
			wasdUi.add("speed mult.", |ui, _| ui.add(
				egui::Slider::new(&mut self.movementSpeedFactor, 0.03125..=4.)
					.clamping(egui::SliderClamping::Never)
			));
			self.movementSpeedFactor = self.movementSpeedFactor.max(0.03125); // sanitize

			// slowFactor
			wasdUi.add("slow-key mult.", |ui, _| ui.add(
				egui::Slider::new(&mut self.slowFactor, 0.03125..=0.75)
					.clamping(egui::SliderClamping::Never)
			));
			self.slowFactor = self.slowFactor.max(0.03125); // sanitize

			// dragSensitivity
			wasdUi.add("drag sensitivity", |ui, _| ui.add(
				egui::Slider::new(&mut self.dragSensitivity, 0.03125..=2.)
					.clamping(egui::SliderClamping::Always)
			));
		});
	}
}

impl Drop for WASDInteractor {
	fn drop (&mut self) {
		// Make sure we let go of our continuous redraw request
		match self.continuousRedrawing {
			ContinuousRedrawing::WASD | ContinuousRedrawing::FocusChange{ .. } =>
				player::lock().dropContinuousRedrawRequest(),
			ContinuousRedrawing::Idle => {}
		}
	}
}
