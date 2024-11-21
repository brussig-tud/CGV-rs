
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
// Imports
//

// Standard library
/* Nothing here yet */

// Local imports
use crate::*;
use crate::util::math;
//////
//
// Enums
//

/// Enum representing either a perspective or orthographic field-of-view in the vertical direction.
#[derive(Clone, Copy, Debug)]
pub enum FoV {
	// The FoV represents a perspective opening angle, in radians
	Perspective(f32),

	// The FoV represents an orthographic extent
	Orthographic(f32)
}



//////
//
// Classes
//


#[derive(Clone, Copy)]
pub struct Intrinsics {
	pub fovY: FoV,
	pub aspect: f32,
	pub f: f32,
	pub zNear: f32,
	pub zFar: f32
}
impl Intrinsics {
	fn defaultWithAspect (aspect: f32) -> Self {
		Self {
			aspect, f: 2., zNear: 0.01, zFar: 100.,
			fovY: FoV::Perspective(math::deg2rad!(60.)),
		}
	}
}

#[derive(Clone, Copy)]
pub struct Extrinsics {
	pub eye: glm::Vec3,
	pub dir: glm::Vec3,
	pub up: glm::Vec3,
}
impl Default for Extrinsics {
	fn default () -> Self {
		Self {
			eye: glm::Vec3::zeros(),
			dir: glm::vec3(0., 0., -1.),
			up: glm::vec3(0., 1., 0.)
		}
	}
}

#[derive(Clone, Copy)]
pub struct Viewport {
	pub min: glm::UVec2,
	pub extend: glm::UVec2
}
impl Viewport
{
	pub fn transformFromClip (&self, clipSpaceXY: &glm::Vec2) -> glm::UVec2
	{
		let scaled =
			((glm::vec2(clipSpaceXY.x, -clipSpaceXY.y) + glm::vec2(1f32, 1f32)) * 0.5).component_mul(
				&glm::vec2(self.extend.x as f32, self.extend.y as f32)
			);
		glm::vec2(scaled.x as u32, scaled.y as u32)
	}
	pub fn transformToClip (&self, screenSpaceXY: &glm::UVec2) -> glm::Vec2
	{
		let pixelCoords_vpRel = *screenSpaceXY - self.min;
		let pixelCoords_clip =
			  glm::vec2(pixelCoords_vpRel.x as f32, pixelCoords_vpRel.y as f32).component_div(
			  	&glm::vec2(self.extend.x as f32, self.extend.y as f32)
			  )
			* 2f32  -  glm::vec2(1f32, 1f32);
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
				let unviewed = glm::inverse(&(projection * view)) * projected;
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
				let unviewed = glm::inverse(&(projection * view)) * projected;
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

	/// Get the projection matrix that is effective at the given pixel coordinates.
	fn projectionAt (&self, pixelCoords: glm::UVec2) -> &glm::Mat4;

	/// Borrow the view matrix for the given global pass.
	///
	/// # Arguments
	///
	/// * `pass` – The declaration of the global pass the [`Player`] requires the view matrix for. The [`Player`] will
	///            only ever query matrices for passes the camera [declared itself](Camera::declareGlobalPasses).
	fn view (&self, pass: &GlobalPassDeclaration) -> &glm::Mat4;

	/// Get the view matrix that is effective at the given pixel coordinates.
	fn viewAt (&self, pixelCoords: glm::UVec2) -> &glm::Mat4;

	/// Borrow the current camera intrinsics.
	fn intrinsics (&self) -> &Intrinsics;

	/// Borrow the current camera extrinsics.
	fn extrinsics (&self) -> &Extrinsics;

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
	fn resize (&mut self, context: &Context, viewportDims: glm::UVec2, interactor: &dyn CameraInteractor);

	/// Indicates that the camera should perform any calculations needed to synchronize its internal state, e.g. update
	/// transformation matrices or anything else it might need to provide [render state](RenderState) to the
	/// [global passes over the scene](Camera::declareGlobalPasses) it declared. The framework guarantees that the
	/// *active* camera will get this method called at least once before any rendering, and whenever the *active*
	/// [`CameraInteractor`] changed something. For manually managed cameras which are *inactive* as far as the
	/// [`Player`] is concerned, updating is the responsibility of the [`Application`].
	fn update (&mut self, interactor: &dyn CameraInteractor) -> bool;

	/// Make the camera declare the global passes it needs to perform to produce its output image.
	fn declareGlobalPasses (&self) -> &[GlobalPassDeclaration];

	/// Reference the framebuffer containing the rendering of the scene acquired by the camera.
	fn framebuffer (&self) -> &hal::Framebuffer;

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
	fn getDepthReadbackDispatcher (&self, pixelCoords: glm::UVec2) -> Option<DepthReadbackDispatcher>;
}

/// A camera that can take input and start full scene render passes with its desired projection and view matrices.
pub trait CameraInteractor
{
	/// Report a short title for the interactor that it will be selectable by.
	///
	/// # Returns
	///
	/// A string slice containing a short descriptive title for the interactor.
	fn title (&self) -> &str;

	/// Compute a projection matrix from internal state.
	///
	/// # Arguments
	///
	/// * `viewportDims` – The dimensions of the viewport the matrix should project to.
	fn projection (&self, viewportDims: glm::UVec2) -> glm::Mat4;

	/// Borrow a reference to the view matrix for the current internal state of the interactor.
	fn view (&self) -> &glm::Mat4;

	/// Indicates that the camera should perform any calculations needed to synchronize its internal state, e.g. compute
	/// transformation matrices from higher-level parameters etc. This is guaranteed to be called at least once before
	/// the interactor is asked to calculate any matrices.
	///
	/// # Arguments
	///
	/// * `camera` – the camera which to interact with.
	/// * `player` – Access to the CGV-rs player instance, useful e.g. to request or stop continous redraws when
	///              animating the camera.
	///
	/// # Returns
	///
	/// `true` if any update to the extrinsic or intrinsic camera parameters occured, `false` otherwise. This
	/// information required to decide whether full scene redraws are necessary.
	fn update (&mut self, camera: &dyn Camera, player: &Player) -> bool;

	/// Report a window event to the camera.
	///
	/// # Arguments
	///
	/// * `event` – The event that the camera should inspect and potentially act upon.
	/// * `camera` – the camera which to interact with.
	/// * `player` – Access to the *CGV-rs* [`Player`] instance, useful for more involved reactions to input.
	///
	/// # Returns
	///
	/// The [outcome](EventOutcome) of the event processing.
	fn input  (&mut self, event: &InputEvent, camera: &dyn Camera, player: &'static Player) -> EventOutcome;
}
