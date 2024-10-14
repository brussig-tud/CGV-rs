
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
use crate::view::*;
use crate::util;
use crate::util::math;
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
	lastMousePos: Option<glm::Vec2>
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
			lastMousePos: None
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

impl Camera for OrbitCamera
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

	fn update (&mut self) {
		self.view = glm::look_at(&self.eye, &self.target, &self.up);
	}

	fn input (&mut self, event: &WindowEvent) -> bool
	{
		match event
		{
			WindowEvent::ModifiersChanged(modifiers) => {
				self.roll = modifiers.state().shift_key();
				false // we did taste the event, but not fully consume it - let others also have a bite
			},

			WindowEvent::MouseInput {state, button, ..}
			=> {
				match *button {
					MouseButton::Left => { self.dragLMB = *state == ElementState::Pressed; true },
					MouseButton::Middle => { self.dragMMB = *state == ElementState::Pressed; true },
					MouseButton::Right => { self.dragRMB = *state == ElementState::Pressed; true },
					_ => false // we didn't consume the event
				}
			},

			WindowEvent::CursorMoved {position, ..}
			=> {
				let delta = self.processMouseMove(position);
				let dist = self.target - self.eye;
				if self.dragLMB {
					let fore = dist.normalize();
					if self.roll {
						self.up = glm::rotate_vec3(&self.up, math::deg2rad!(0.5*delta.y), &fore);
					}
					else {
						let right = glm::normalize(&glm::cross(&fore, &self.up));
						/* rotate horizontally */ {
							let newRight = glm::rotate_vec3(&right, math::deg2rad!(delta.x), &self.up);
							let newFore = glm::cross(&self.up, &newRight);
							self.eye = self.target - dist.norm()*newFore;
						}
						/* rotate vertically */ {
							let dist = self.target - self.eye;
							let fore = dist.normalize();
							let right = glm::normalize(&glm::cross(&fore, &self.up));
							let newUp = glm::rotate_vec3(&self.up, math::deg2rad!(-delta.y), &right);
							let newFore = glm::cross(&newUp, &right);
							self.up = newUp;
							self.eye = self.target - dist.norm()*newFore;
						}
					}
					true
				}
				else if self.dragMMB {
					let fore = dist.norm()*delta.y*0.0625 * dist.normalize();
					self.target += fore;
					self.eye += fore;
					true
				}
				else if self.dragRMB {
					let speed = dist.norm() * delta*0.03125;
					let right = glm::normalize(&glm::cross(&dist, &self.up));
					let diff = speed.x*right + speed.y*self.up;
					self.target += diff;
					self.eye += diff;
					true
				}
				else
					{false} // we didn't consume the event
			},

			WindowEvent::MouseWheel {delta, ..}
			=> {
				let toEye = self.eye - self.target;
				match delta
				{
					MouseScrollDelta::LineDelta(_, y) => {
						self.eye = self.target + toEye*(1.+y*-0.125);
						true
					},
					MouseScrollDelta::PixelDelta(delta) => {
						self.eye = self.target + toEye*(1.+(delta.y as f32)*(-1./1024.));
						true
					}
				}
			},

			// We didn't consume the event
			_ => false
		}
	}
}
