
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
mod orbitinteractor;
pub use orbitinteractor::OrbitInteractor; // re-export



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
// Classes
//

#[derive(Clone, Copy)]
pub struct Viewport {
	pub min: glm::UVec2,
	pub extend: glm::UVec2
}
impl Viewport {
	pub fn transformFromClip (&self, clipSpaceXY: &glm::Vec2) -> glm::UVec2 {
		todo!("implement forward viewspace transform")
	}
	pub fn transformToClip (&self, viewportSpaceXY: &glm::UVec2) -> glm::Vec2 {
		let pixelCoords_vpRel = *viewportSpaceXY - self.min;
		let pixelCoords_clip01 =
			glm::vec2(pixelCoords_vpRel.x as f32, pixelCoords_vpRel.y as f32).component_div(
				&glm::vec2(self.extend.x as f32, self.extend.y as f32)
			);
		let pixelCoords_clip = pixelCoords_clip01*2f32 - glm::vec2(1f32, 1f32);
		glm::vec2(pixelCoords_clip.x, -pixelCoords_clip.y)
	}
}

pub struct DepthReadbackDispatcher<'a> {
	pixelCoords: glm::UVec2,
	viewport: Viewport,
	projection: &'a glm::Mat4,
	view: &'a glm::Mat4,
	depthTexture: &'a hal::Texture
}
impl<'a> DepthReadbackDispatcher<'a>
{
	pub fn new (
		pixelCoords: &glm::UVec2, viewport: &Viewport, projection: &'a glm::Mat4, view: &'a glm::Mat4,
		depthTexture: &'a hal::Texture
	) -> Self {
		Self { pixelCoords: *pixelCoords, viewport: *viewport, projection, view, depthTexture }
	}

	pub fn getDepthValue_async<Closure: FnOnce(f32) + wgpu::WasmNotSend + 'static> (
		&self, context: &Context, callback: Closure
	){
		let pixelCoords = self.pixelCoords;
		self.depthTexture.readbackAsync(context, move |texels, rowStride| {
			let loc = pixelCoords.y as usize *rowStride + pixelCoords.x as usize;
			callback(hal::decodeDepth(loc, texels));
		});
	}

	pub fn unprojectPointH_async<Closure: FnOnce(Option<&glm::Vec4>) + wgpu::WasmNotSend + 'static> (
		&self, context: &Context, callback: Closure
	){
		let pixelCoords = self.pixelCoords;
		let pixelCoords_clip = self.viewport.transformToClip(&pixelCoords);
		let projection = util::statify(self.projection);
		let view = util::statify(self.view);
		self.depthTexture.readbackAsync(context, move |texels, rowStride| {
			let loc = pixelCoords.y as usize *rowStride + pixelCoords.x as usize;
			let projected = glm::vec4(
				pixelCoords_clip.x, pixelCoords_clip.y, hal::decodeDepth(loc, texels), 1.
			);
			if projected.z < 1. {
				let unprojected = glm::inverse(projection) * projected;
				let unviewed = glm::inverse(view) * unprojected;
				callback(Some(&unviewed));
			}
			else { callback(None); }
		});
	}

	pub fn unprojectPoint_async<Closure: FnOnce(Option<&glm::Vec3>) + wgpu::WasmNotSend + 'static> (
		&self, context: &Context, callback: Closure
	){
		let pixelCoords = self.pixelCoords;
		let pixelCoords_clip = self.viewport.transformToClip(&pixelCoords);
		let projection = util::statify(self.projection);
		let view = util::statify(self.view);
		self.depthTexture.readbackAsync(context, move |texels, rowStride| {
			let loc = pixelCoords.y as usize *rowStride + pixelCoords.x as usize;
			let projected = glm::vec4(
				pixelCoords_clip.x, pixelCoords_clip.y, hal::decodeDepth(loc, texels), 1.
			);
			if projected.z < 1. {
				let unprojected = glm::inverse(projection) * projected;
				let unviewed = glm::inverse(view) * unprojected;
				callback(Some(&(glm::vec4_to_vec3(&unviewed) / unviewed.w)));
			}
			else { callback(None); }
		});
	}
}



//////
//
// Traits
//

/// A camera that can produce images of the scene.
pub trait Camera
{
	/// Borrow the projection matrix for the given global pass.
	///
	/// # Arguments
	///
	/// * `pass` – The declaration of the global pass the [`Player`] requires the projection matrix for. The [`Player`]
	///            will only ever query matrices for passes the camera [declared itself](Camera::declareGlobalPasses).
	fn projection (&self, pass: &GlobalPassDeclaration) -> &glm::Mat4;

	fn projectionAt (&self, pixelCoords: &glm::UVec2) -> &glm::Mat4;

	/// Borrow the view matrix for the given global pass.
	///
	/// # Arguments
	///
	/// * `pass` – The declaration of the global pass the [`Player`] requires the view matrix for. The [`Player`] will
	///            only ever query matrices for passes the camera [declared itself](Camera::declareGlobalPasses).
	fn view (&self, pass: &GlobalPassDeclaration) -> &glm::Mat4;

	fn viewAt (&self, pixelCoords: &glm::UVec2) -> &glm::Mat4;

	/// Report a viewport change to the camera. The framework guarantees that the *active* camera will get this method
	/// called at least once before it gets asked to declare any render passes for the first time. For manually managed
	/// cameras which are *inactive* as far as the [`Player`] is concerned, resizing is the responsibility of the
	/// [`Application`].
	///
	/// # Arguments
	///
	/// * `context` – The graphics context.
	/// * `viewportDims` – The dimensions of the viewport the camera should produce images for.
	/// * `interactor` – The currently active camera interactor.
	fn resize (&mut self, context: &Context, viewportDims: &glm::UVec2, interactor: &dyn CameraInteractor);

	/// Indicates that the camera should perform any calculations needed to synchronize its internal state, e.g. update
	/// transformation matrices for the current camera interactor or anything else it might need to
	/// [declare the global passes over the scene](Camera::declareGlobalPasses) it needs. The framework guarantees that
	/// the *active* camera will get this method whenever the *active* [`CameraInteractor`] changes something before
	/// being asked to declare any render passes for the current frame. For manually managed cameras which are
	/// *inactive* as  far as the [`Player`] is concerned, updating is the responsibility of the [`Application`].
	///
	/// # Arguments
	///
	/// * `interactor` – The camera interactor providing base intrinsic and extrinsic parameters to the camera.
	fn update (&mut self, interactor: &dyn CameraInteractor);

	/// Make the camera declare the global passes it needs to perform to produce its output image.
	fn declareGlobalPasses (&self) -> &[GlobalPassDeclaration];

	/// Report the individual name of the camera instance.
	///
	/// # Returns
	///
	/// The name given to the camera instance (usually upon creation).
	fn name (&self) -> &str;

	/// Obtain a dispatcher for asynchronously reading back the depth value at the given pixel coordinates.
	///
	/// Dispatchers are tailored towards the pixel coordinates requested. The reason for this is that cameras might
	/// compose the final image from several viewports, and not actually attach any depth at all to the main surface,
	/// but the render targets for the individual viewports *could* have depth attached. Providing the coordinates the
	/// caller is interested in here ensures that the camera can still provide depth in such situations.
	///
	/// # Arguments
	///
	/// * `pixelCoords` – The pixel coordinates at which to get the depth value.
	///
	/// # Returns
	///
	/// `Some` dispatcher if the camera can provide depth for the given pixel coordinates, `None` otherwise.
	fn getDepthReadbackDispatcher (&self, pixelCoords: &glm::UVec2) -> Option<DepthReadbackDispatcher>;
}

/// A camera that can take input and start full scene render passes with its desired projection and view matrices.
pub trait CameraInteractor
{
	/// Compute a projection matrix from internal state.
	///
	/// # Arguments
	///
	/// * `viewportDims` – The dimensions of the viewport the matrix should project to.
	fn projection (&self, viewportDims: &glm::UVec2) -> glm::Mat4;

	/// Borrow a reference to the view matrix for the current internal state of the interactor.
	fn view (&self) -> &glm::Mat4;

	/// Indicates that the camera should perform any calculations needed to synchronize its internal state, e.g. compute
	/// transformation matrices from higher-level parameters etc. This is guaranteed to be called at least once before
	/// the interactor is asked to calculate any matrices.
	///
	/// # Arguments
	///
	/// * `player` – Access to the CGV-rs player instance, useful e.g. to schedule redraws when animating the camera.
	///
	/// # Returns
	///
	/// `true` if any update to the extrinsic or intrinsic camera parameters occured, `false` otherwise. This
	/// information is utilized to optimize managed bind group updates.
	fn update (&mut self, player: &'static Player) -> bool;

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
