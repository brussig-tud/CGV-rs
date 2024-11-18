
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
// Statics
//

pub const CLIPSPACE_TRANSFORM_OGL2WGPU: glm::Mat4 = glm::Mat4::new(
	1.0, 0.0, 0.0, 0.0,
	0.0, 1.0, 0.0, 0.0,
	0.0, 0.0, 0.5, 0.5,
	0.0, 0.0, 0.0, 1.0,
);



//////
//
// Structs
//

#[derive(Debug)]
struct FocusChange {
	pub old: glm::Vec3,
	pub new: glm::Vec3,
	pub t: f32
}



//////
//
// Classes
//

////
// OrbitCamera

/// A camera interactor for orbital movement around a focal point.
#[derive(Debug)]
pub struct OrbitInteractor
{
	eye: glm::Vec3,
	target: glm::Vec3,
	up: glm::Vec3,
	fov: FoV,
	zNear: f32,
	zFar: f32,
	view: glm::Mat4,

	focusChange: Option<FocusChange>,
	dirty: bool,
}

impl OrbitInteractor
{
	pub fn new () -> Self
	{
		OrbitInteractor {
			eye: glm::Vec3::new(0., 0., 2.),
			target: glm::Vec3::zeros(),
			up: glm::Vec3::new(0., 1., 0.),
			fov: FoV::Perspective(math::deg2rad!(60.)),
			zNear: 0.01,
			zFar: 100.,
			view: glm::Mat4::identity(),
			focusChange: None,
			dirty: true,
		}
	}
}

impl CameraInteractor for OrbitInteractor
{
	fn title (&self) -> &str {
		"Orbit"
	}

	fn projection (&self, viewportDims: glm::UVec2) -> glm::Mat4
	{
		let aspect = viewportDims.x as f32 / viewportDims.y as f32;
		match self.fov
		{
			FoV::Perspective(fov)
			=> CLIPSPACE_TRANSFORM_OGL2WGPU * glm::perspective(aspect, fov, self.zNear, self.zFar),

			FoV::Orthographic(fov)
			=> {
				let halfHeight = fov*0.5;
				let halfWidth = halfHeight*aspect;
				CLIPSPACE_TRANSFORM_OGL2WGPU * glm::ortho(
					-halfWidth, halfWidth, -halfHeight, halfHeight, self.zNear, self.zFar
				)
			}
		}
	}

	fn view (&self) -> &glm::Mat4 {
		&self.view
	}

	fn update (&mut self, player: &Player) -> bool
	{
		if let Some(focusChange) = &mut self.focusChange
		{
			focusChange.t = f32::min(focusChange.t + player.lastFrameTime()*2f32, 1f32);
			let targetCur = math::smoothLerp3(&focusChange.old, &focusChange.new, focusChange.t);
			let offset = targetCur - self.target;
			self.target = targetCur;
			self.eye += offset;
			if targetCur == focusChange.new {
				self.focusChange = None;
				player.dropContinuousRedrawRequest();
			}
			self.dirty = true;
		}
		if self.dirty {
			self.view = glm::look_at(&self.eye, &self.target, &self.up);
			self.dirty = false;
			true
		}
		else { false }
	}

	fn input (&mut self, event: &InputEvent, player: &'static Player) -> EventOutcome
	{
		match event
		{
			InputEvent::Dragged(info)
			=> {
				// Preamble
				let delta = glm::vec2(-info.direction.x, info.direction.y);
				let dist = self.target - self.eye;
				let mut handled= false;
				if info.button(egui::PointerButton::Primary)
				{
					let fore = dist.normalize();
					if info.modifiers.shift {
						self.up = glm::rotate_vec3(
							&self.up, math::deg2rad!(delta.y*-1./4.), &fore
						);
					}
					else {
						let mut right = glm::normalize(&glm::cross(&fore, &self.up));
						right = glm::rotate_vec3(
							&right, math::deg2rad!(delta.x*1./3.), &self.up
						);
						self.eye = self.target - dist.norm()*glm::cross(&self.up, &right);
						self.up = glm::rotate_vec3(
							&self.up, math::deg2rad!(delta.y*-1./3.), &right
						);
						self.eye =    self.target
							- (self.target-self.eye).norm()*glm::cross(&self.up, &right);
					}
					self.dirty = true;
					handled = true;
				}
				if info.button(egui::PointerButton::Secondary)
				{
					let speed = dist.norm() * delta*1./512.;
					let right = glm::normalize(&glm::cross(&dist, &self.up));
					let diff = speed.x*right + speed.y*self.up;
					self.target += diff;
					self.eye += diff;
					self.dirty = true;
					if self.focusChange.is_some() {
						self.focusChange = None;
						player.dropContinuousRedrawRequest();
					}
					handled = true;
				}
				if info.button(egui::PointerButton::Middle) {
					let fore = dist.norm()*delta.y*1./256. * dist.normalize();
					self.target += fore;
					self.eye += fore;
					self.dirty = true;
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

			InputEvent::MouseWheel(amount) => {
				let toEye = self.eye - self.target;
				self.eye = self.target + toEye*(1.+amount*-1./256.);
				self.dirty = true;
				EventOutcome::HandledExclusively(/* redraw */true)
			},

			InputEvent::DoubleClick(info) => {
				let this = util::mutify(self);
				player.unprojectPointAtSurfacePixel_async(
					info.position,
					move |point| {
						if let Some(point) = point {
							tracing::debug!("Double-click to new focus: {:?}", point);
							this.focusChange = Some(FocusChange {
								old: this.target, new: *point, t: 0.
							});
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
