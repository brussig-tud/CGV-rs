
//////
//
// Imports
//

// Standard library
/* Nothing here yet */

// Local imports
use crate::*;
use {view::*, util::math};



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
pub struct OrbitInteractor
{
	/*eye: glm::Vec3,
	target: glm::Vec3,
	up: glm::Vec3,
	fov: FoV,
	zNear: f32,
	zFar: f32,
	view: glm::Mat4,*/

	focusChange: Option<FocusChange>,
	//dirty: bool,
}

impl OrbitInteractor
{
	pub fn new () -> Self
	{
		OrbitInteractor {
			/*eye: glm::Vec3::new(0., 0., 2.),
			target: glm::Vec3::zeros(),
			up: glm::Vec3::new(0., 1., 0.),
			fov: FoV::Perspective(math::deg2rad!(60.)),
			zNear: 0.01,
			zFar: 100.,
			view: glm::Mat4::identity(),*/
			focusChange: None,
			//dirty: true,
		}
	}
}

impl CameraInteractor for OrbitInteractor
{
	fn title (&self) -> &str {
		"Orbit"
	}

	fn update (&mut self, camera: &mut dyn Camera, player: &Player)
	{
		if let Some(focusChange) = &mut self.focusChange
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
		// Local helper to share zoom adjustment code across match arms
		fn adjustZoom (extr: &mut Extrinsics, intr: &mut Intrinsics, amount: f32) {
			let focus = extr.eye + extr.dir*intr.f;
			intr.f = intr.f * (1. + amount*-1./256.);
			extr.eye = focus - extr.dir*intr.f;
		}
		// Local helper to share FoV adjustment code across match arms
		fn adjustFov (extr: &mut Extrinsics, intr: &mut Intrinsics, amount: f32)
		{
			const ORTHO_THRESHOLD: f32 = 15.;
			if let FoV::Perspective(fov) = intr.fovY {
				let dia = Intrinsics::frustumDiameterAtFocus(fov, intr.f);
				let newFov = f32::min(fov + math::deg2rad!(amount*0.125), 179.);
				if newFov < math::deg2rad!(ORTHO_THRESHOLD) {
					intr.fovY = FoV::Orthographic(dia)
				}
				else {
					let focusOld = extr.eye + extr.dir*intr.f;
					intr.fovY = FoV::Perspective(newFov);
					intr.f = Intrinsics::focusDistForFrustumDiameterAndFov(dia, newFov);
					extr.eye = focusOld - extr.dir*intr.f;
				}
			}
			else {
				if amount > 0. {
					intr.fovY = FoV::Perspective(math::deg2rad!(ORTHO_THRESHOLD + amount*0.125));
				}
			}
		}

		// Match on relevant events
		match event
		{
			InputEvent::Dragged(info)
			=> {
				let delta = glm::vec2(-info.direction.x, info.direction.y);
				let mut handled= false;
				if info.button(egui::PointerButton::Primary)
				{
					// We only borrow the camera parameters inside a scope where we're sure we'll be changing something,
					// as the camera WILL recalculate internal state after a mutable borrow
					let p = camera.parameters_mut();
					if info.modifiers.shift {
						p.extrinsics.up = glm::rotate_vec3(
							&p.extrinsics.up, math::deg2rad!(delta.y*-1./4.), &p.extrinsics.dir
						);
					}
					else {
						let target = p.extrinsics.eye + p.intrinsics.f*p.extrinsics.dir;
						let mut right = glm::normalize(&glm::cross(&p.extrinsics.dir, &p.extrinsics.up));
						right = glm::rotate_vec3(
							&right, math::deg2rad!(delta.x*1./3.), &p.extrinsics.up
						);
						p.extrinsics.eye = target - p.intrinsics.f*glm::cross(&p.extrinsics.up, &right);
						p.extrinsics.up = glm::rotate_vec3(
							&p.extrinsics.up, math::deg2rad!(delta.y*-1./3.), &right
						);
						p.extrinsics.dir = glm::cross(&p.extrinsics.up, &right);
						p.extrinsics.eye = target - p.intrinsics.f*p.extrinsics.dir;
					}
					handled = true;
				}
				if info.button(egui::PointerButton::Secondary)
				{
					// We only borrow the camera parameters inside a scope where we're sure we'll be changing something,
					// as the camera WILL recalculate internal state after a mutable borrow
					let p = camera.parameters_mut();
					let speed = p.intrinsics.f * delta*1./512.;
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
					// We only borrow the camera parameters inside a scope where we're sure we'll be changing something,
					// as the camera WILL recalculate internal state after a mutable borrow
					let p = camera.parameters_mut();
					let movement = p.intrinsics.f*delta.y*1./256. * p.extrinsics.dir;
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
						let p = camera.parameters_mut();
						adjustFov(&mut p.extrinsics, &mut p.intrinsics, info.amount.y);
					} else {
						let p = camera.parameters_mut();
						adjustZoom(&mut p.extrinsics, &mut p.intrinsics, info.amount.y);
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
				player.unprojectPointAtSurfacePixel_async(
					info.position,
					move |point| {
						if let Some(point) = point {
							tracing::debug!("Double-click to new focus: {:?}", point);
							let old = extr.eye + f*extr.dir;
							this.focusChange = Some(FocusChange {old, new: *point, prev: old, t: 0., player});
							player.pushContinuousRedrawRequest();
						}
					}
				);
				EventOutcome::HandledExclusively(/* redraw */true)
			},

			_ => EventOutcome::NotHandled
		}
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
