
//////
//
// Imports
//

// Standard library
/* Nothing here yet */

// Winit library
use winit::dpi;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};

// Local imports
use crate::*;
use {view::*, util::math};
use EventOutcome::*;



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
// Classes
//

////
// OrbitCamera

/// A camera for orbital movements around a focal point.
#[derive(Debug)]
pub struct OrbitCamera {
	eye: glm::Vec3,
	target: glm::Vec3,
	up: glm::Vec3,
	aspect: f32,
	fov: FoV,
	zNear: f32,
	zFar: f32,
	proj: glm::Mat4,
	view: glm::Mat4,
	dragLMB: bool,
	dragMMB: bool,
	dragRMB: bool,
	roll: bool,
	lastMousePos: Option<glm::Vec2>,
	dirty: bool
}

impl OrbitCamera
{
	pub fn new () -> Self
	{
		OrbitCamera {
			eye: glm::Vec3::new(0., 0., 2.),
			target: glm::Vec3::zeros(),
			up: glm::Vec3::new(0., 1., 0.),
			aspect: 1.,
			fov: FoV::Perspective(util::math::deg2rad!(60.)),
			zNear: 0.01,
			zFar: 100.,
			proj: glm::Mat4::identity(),
			view: glm::Mat4::identity(),
			dragLMB: false, dragMMB: false, dragRMB: false, roll: false,
			lastMousePos: None,
			dirty: true
		}
	}

	pub fn processMouseMove (&mut self, newPos: &dpi::PhysicalPosition<f64>) -> glm::Vec2
	{
		if let Some(oldPos) = self.lastMousePos {
			let newPos = glm::Vec2::new(newPos.x as f32, newPos.y as f32);
			self.lastMousePos = Some(newPos);
			glm::vec2(oldPos.x - newPos.x, newPos.y - oldPos.y)
		}
		else {
			self.lastMousePos = Some(glm::Vec2::new(newPos.x as f32, newPos.y as f32));
			glm::Vec2::zeros()
		}
	}
}

impl CameraInteractor for OrbitCamera
{
	fn projection (&self) -> &glm::Mat4 {
		&self.proj
	}

	fn view (&self) -> &glm::Mat4 {
		&self.view
	}

	fn resize (&mut self, viewportDims: &glm::Vec2)
	{
		self.aspect = viewportDims.x / viewportDims.y;
		self.proj = match self.fov
		{
			FoV::Perspective(fov)
			=> CLIPSPACE_TRANSFORM_OGL2WGPU * glm::perspective(self.aspect, fov, self.zNear, self.zFar),

			FoV::Orthographic(fov)
			=> {
				let halfHeight = fov*0.5;
				let halfWidth = halfHeight*self.aspect;
				CLIPSPACE_TRANSFORM_OGL2WGPU * glm::ortho(
					-halfWidth, halfWidth, -halfHeight, halfHeight, self.zNear, self.zFar
				)
			}
		};
	}

	fn update (&mut self) -> bool {
		if self.dirty {
			self.view = glm::look_at(&self.eye, &self.target, &self.up);
			self.dirty = false;
			true
		}
		else { false }
	}

	fn input (&mut self, event: &WindowEvent, _: &'static Player) -> EventOutcome
	{
		match event
		{
			WindowEvent::ModifiersChanged(modifiers) => {
				self.roll = modifiers.state().shift_key();
				HandledDontClose(/* redraw */true)
			},

			WindowEvent::MouseInput {state, button, ..}
			=> {
				match *button {
					MouseButton::Left => {
						self.dragLMB = *state == ElementState::Pressed;
						HandledExclusively(/* redraw */false)
					},
					MouseButton::Middle => {
						self.dragMMB = *state == ElementState::Pressed;
						HandledExclusively(/* redraw */false)
					},
					MouseButton::Right => {
						self.dragRMB = *state == ElementState::Pressed;
						HandledExclusively(/* redraw */false)
					},
					_ => NotHandled // we didn't consume the event
				}
			},

			WindowEvent::CursorMoved {position, ..}
			=> {
				let delta = self.processMouseMove(position);
				let dist = self.target - self.eye;

				// Orbital motion
				if self.dragLMB {
					let fore = dist.normalize();
					if self.roll {
						self.up = glm::rotate_vec3(
							&self.up, math::deg2rad!(delta.y*-1./3.), &fore
						);
					}
					else {
						let mut right = glm::normalize(&glm::cross(&fore, &self.up));
						right = glm::rotate_vec3(
							&right, math::deg2rad!(delta.x*0.5), &self.up
						);
						self.eye = self.target - dist.norm()*glm::cross(&self.up, &right);
						self.up = glm::rotate_vec3(
							&self.up, math::deg2rad!(delta.y*-0.5), &right
						);
						self.eye =    self.target
						           - (self.target-self.eye).norm()*glm::cross(&self.up, &right);
					}
					self.dirty = true;
					return HandledExclusively(/* redraw */true);
				}

				// Forward/backward motion
				if self.dragMMB {
					let fore = dist.norm()*delta.y*0.0078125 * dist.normalize();
					self.target += fore;
					self.eye += fore;
					self.dirty = true;
					return HandledExclusively(/* redraw */true);
				}

				// Panning motion
				if self.dragRMB {
					let speed = dist.norm() * delta*0.00390625;
					let right = glm::normalize(&glm::cross(&dist, &self.up));
					let diff = speed.x*right + speed.y*self.up;
					self.target += diff;
					self.eye += diff;
					self.dirty = true;
					return HandledExclusively(/* redraw */true);
				}

				// We didn't consume the event
				NotHandled
			},

			WindowEvent::MouseWheel {delta, ..}
			=> {
				let toEye = self.eye - self.target;
				match delta
				{
					MouseScrollDelta::LineDelta(_, y) => {
						self.eye = self.target + toEye*(1.+y*-0.125);
						self.dirty = true;
						HandledExclusively(/* redraw */true)
					},
					MouseScrollDelta::PixelDelta(delta) => {
						self.eye = self.target + toEye*(1.+(delta.y as f32)*(-1./1024.));
						self.dirty = true;
						HandledExclusively(/* redraw */true)
					}
				}
			},

			// We didn't consume the event
			_ => NotHandled
		}
	}
}
