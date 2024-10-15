
//////
//
// Imports
//

// Standard library
/* Nothing here yet */

// Winit library
use winit::event::WindowEvent;



//////
//
// Module definitions
//

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

/// A camera that can take input and compute projection and view matrices from that.
pub trait Camera
{
	/// Borrow a reference of the current projection matrix.
	fn projection (&self) -> &glm::Mat4;

	/// Borrow a reference of the current view matrix.
	fn view (&self) -> &glm::Mat4;

	/// Report a viewport change to the camera. The framework guarantees that any active camera will get this method
	/// called at least once before it needs to report any matrices.
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
	///
	/// # Returns
	///
	/// The outcome of the event processing.
	fn input (&mut self, event: &WindowEvent) -> crate::EventOutcome;
}
