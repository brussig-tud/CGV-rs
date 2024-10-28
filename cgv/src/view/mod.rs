
//////
//
// Imports
//

// Standard library
/* Nothing here yet */

// Winit library
use winit::event::WindowEvent;

// Local imports
use crate::*;



//////
//
// Module definitions
//

/// The internal submodule for the MonoCamera implementation
mod monocamera;
pub use monocamera::MonoCamera; // re-export

/// The internal submodule for the OrbitCamera implementation
mod orbitcamera;
pub use orbitcamera::OrbitCamera; // re-export



//////
//
// Enums
//

/// Enum representing either a perspective or orthographic field-of-view in the vertical direction.
#[derive(Debug)]
pub enum FoV {
	// The FoV represents a perspective opening angle
	Perspective(f32),

	// The FoV represents an orthographic extent
	Orthographic(f32)
}



//////
//
// Traits
//

/// A camera that can produce images of the scene.
pub trait Camera
{
	/// Report a viewport change to the camera. The framework guarantees that the *active* camera will get this method
	/// called at least once before it gets asked to declare any render passes. For manually managed cameras which are
	/// *inactive* as far as the [`Player`] is concerned, resizing is the responsibility of the [`Application`].
	///
	/// # Arguments
	///
	/// * `context` – The graphics context.
	/// * `viewportDims` – The dimensions of the viewport the camera should produce images for.
	fn resize (&mut self, context: &Context, viewportDims: &glm::UVec2);

	/// Make the camera declare the global passes it needs to perform to produce its output image.
	fn declareGlobalPasses (&self) -> &[GlobalPassDeclaration];
}

/// A camera that can take input and start full scene render passes with its desired projection and view matrices.
pub trait CameraInteractor
{
	/// Borrow a reference to the current projection matrix.
	fn projection (&self) -> &glm::Mat4;

	/// Borrow a reference to the current view matrix.
	fn view (&self) -> &glm::Mat4;

	/// Report a viewport change to the camera. The framework guarantees that any active camera will get this method
	/// called at least once before it gets asked to declare any render passes.
	///
	/// # Arguments
	///
	/// * `viewportDims` – The dimensions of the viewport the camera should manage viewing for.
	fn resize (&mut self, viewportDims: &glm::Vec2);

	/// Indicates that the camera should perform any calculations needed to synchronize its internal state, e.g. compute
	/// transformation matrices from higher-level parameters etc. This is guaranteed to be called at least once before
	/// any active camera is supposed to report any matrices.
	fn update (&mut self);

	/// Report a window event to the camera.
	///
	/// # Arguments
	///
	/// * `event` – The event that the camera should inspect and potentially act upon.
	/// * `player` – Access to the CGV-rs player instance, useful for more involved reactions to input.
	///
	/// # Returns
	///
	/// The outcome of the event processing.
	fn input (&mut self, event: &WindowEvent, player: &'static Player) -> crate::EventOutcome;
}
