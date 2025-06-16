
//////
//
// Module definitions
//

/// The internal submodule of the CameraParameters implementation
mod cameraparameters;
pub use cameraparameters::{Intrinsics, Extrinsics, CameraParameters}; // re-export

/// The internal submodule for the MonoCamera implementation
mod monocamera;
pub use monocamera::MonoCamera; // re-export

/// The internal submodule for the OrbitInteractor implementation
mod orbitinteractor;
pub use orbitinteractor::OrbitInteractor; // re-export

/// The internal submodule for the WASDInteractor implementation
mod wasdinteractor;
pub use wasdinteractor::WASDInteractor; // re-export



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



//////
//
// Enums
//

/// Enum representing either a perspective or orthographic field-of-view in the vertical direction.
#[derive(Clone, Copy, Debug)]
pub enum FoV {
	/// The FoV represents a perspective opening angle, in radians
	Perspective(f32),

	/// The FoV represents an orthographic extent.
	Orthographic(f32)
}
impl FoV
{
	/// Check if the current variant is [`Perspective`](FoV::Perspective).
	pub fn isPerspective (&self) -> bool {
		if let FoV::Perspective(_) = self { true } else { false }
	}

	/// Check if the current variant is [`Orthographic`](FoV::Orthographic).
	pub fn isOrthographic (&self) -> bool {
		if let FoV::Orthographic(_) = self { true } else { false }
	}
}
impl PartialEq for FoV {
	fn eq (&self, other: &Self) -> bool
	{
		if let FoV::Perspective(fov_self) = self && let FoV::Perspective(fov_other) = other {
			fov_self == fov_other
		}
		else if let FoV::Orthographic(h_self) = self && let FoV::Orthographic(h_other) = other {
			h_self == h_other
		}
		else {
			false
		}
	}
}



//////
//
// Classes
//

/// A helper object for managing animated camera focus changes.
struct FocusChange {
	camDir: glm::Vec3,
	pub oldEye: glm::Vec3,
	pub newEye: glm::Vec3,
	pub oldF: f32,
	pub newF: f32,
	pub t: f32,
	pub speed: f32
}
impl FocusChange
{
	/// Create a focus change manager that will adjust the camera positioning and focus from the given start camera
	/// parameters. Before it can be used to animate a transition to a new focus point, that focus point must be set
	/// via [`setNewFocus`](Self::setNewFocus).
	///
	/// # Arguments
	///
	/// * `cameraParameters` – The camera parameters representing the current state before the transition.
	/// * `timespan` – The time, in seconds, that the transition to the new focus should take.
	///
	/// # Returns
	///
	/// A new `FocusChange` instance initialized for the given parameters.
	pub fn new (cameraParameters: &CameraParameters, timespan: f32) -> Self { Self {
		camDir: cameraParameters.extrinsics.dir, oldEye: cameraParameters.extrinsics.eye, newEye: glm::Vec3::default(),
		oldF: cameraParameters.intrinsics.f, newF: f32::default(), t: 0., speed: 1./timespan
	}}

	/// Set the desired focus point to animate onto. Must be called before the first call to [`update`].
	///
	/// # Arguments
	///
	/// * `newFocus` – The desired new focus point.
	pub fn setNewFocus (&mut self, newFocus: &glm::Vec3) {
		let proj = (self.oldEye - newFocus).dot(&self.camDir);
		self.newEye = newFocus + proj*self.camDir;
		self.newF = (newFocus - self.newEye).norm();
	}

	/// Update the focus transition for the given change in time, manipulating the provided camera parameters.
	///
	/// # Arguments
	///
	/// * `dt` – The time passed since the previous update, in seconds.
	/// * `cameraParameters` – The camera parameters to update.
	///
	/// # Returns
	///
	/// `true` if the transition is complete, `false` otherwise.
	pub fn update (&mut self, dt: f32, cameraParameters: &mut CameraParameters) -> bool
	{
		self.t = f32::min(self.t + self.speed*dt, 1f32);
		cameraParameters.extrinsics.eye = util::math::smoothLerp3(&self.oldEye, &self.newEye, self.t);
		cameraParameters.intrinsics.f = util::math::smoothLerp(self.oldF, self.newF, self.t);
		cameraParameters.extrinsics.eye == self.newEye
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

	/// Borrow the current camera parameters.
	fn parameters (&self) -> &CameraParameters;

	/// Mutably borrow the current camera parameters. Implementations should assume that their camera parameters will
	/// have changed when the borrow expires and take appropriate measures to mark their internal state as dirty.
	fn parameters_mut (&mut self) -> &mut CameraParameters;

	/// Report a viewport change to the camera. The framework guarantees that the *active* camera will get this method
	/// called at least once before it gets asked to declare any render passes for the first time. For manually managed
	/// cameras which are *inactive* as far as the [`Player`] is concerned, resizing is the responsibility of the
	/// [`Application`].
	///
	/// # Arguments
	///
	/// * `context` – The graphics context.
	/// * `viewportDims` – The dimensions of the viewport the camera should produce images for.
	fn resize (&mut self, context: &Context, viewportDims: glm::UVec2);

	/// Indicates that the camera should perform any calculations needed to synchronize its internal state, e.g. update
	/// transformation matrices or anything else it might need to provide [render state](RenderState) to the
	/// [global passes over the scene](Camera::declareGlobalPasses) it declared. The framework guarantees that the
	/// *active* camera will get this method called at least once before any rendering. For manually managed cameras
	/// which are *inactive* as far as the [`Player`] is concerned, the [`Application`] is responsible for updating.
	fn update (&mut self) -> bool;

	/// Make the camera declare the global passes it needs to perform to produce its output image.
	fn declareGlobalPasses (&self) -> &[GlobalPassDeclaration<'_>];

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
	fn getDepthReadbackDispatcher (&self, pixelCoords: glm::UVec2) -> Option<DepthReadbackDispatcher<'_>>;
}

/// An object that can take user input and manipulate a [`Camera`]'s parameters accordingly. 
pub trait CameraInteractor
{
	/// Report a short title for the interactor that it will be selectable by.
	///
	/// # Returns
	///
	/// A string slice containing a short descriptive title for the interactor.
	fn title (&self) -> &str;

	/// Indicates that the camera interactor should perform any calculations needed to prepare the passed-in camera for
	/// rendering the next frame.
	///
	/// # Arguments
	///
	/// * `camera` – the camera to interact with.
	/// * `player` – Access to the CGV-rs player instance, useful e.g. to request or stop continuous redraws when
	///              animating the camera.
	fn update (&mut self, camera: &mut dyn Camera, player: &Player);

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
	fn input (&mut self, event: &InputEvent, camera: &mut dyn Camera, player: &'static Player) -> EventOutcome;

	fn ui (&mut self, assignedCamera: &mut dyn Camera, ui: &mut egui::Ui);
}



//////
//
// Functions
//

/// Efficiently add a transformation to the passed in (right-handed) projection matrix that transforms *OpenGL* clip
/// space ($z=-1..1$) into *WebGPU* clip space ($z=0..1$).
///
/// # Arguments
///
/// * `oglProjection` – Mutable reference to the projection matrix that should receive the added transformation.
///
/// # Returns
///
/// A mutable reference to the same matrix that was referenced via `oglProjection`, with the transformation from
/// *OpenGL* clip space to *WebGPU* clip space applied.
pub fn transformClipspaceOGL2WGPU (oglProjection: &glm::Mat4) -> glm::Mat4
{
	const CLIPSPACE_TRANSFORM_OGL2WGPU: glm::Mat4 = glm::Mat4::new(
		1.0, 0.0, 0.0, 0.0,
		0.0, 1.0, 0.0, 0.0,
		0.0, 0.0, 0.5, 0.5,
		0.0, 0.0, 0.0, 1.0,
	);

	// ToDo: investigate why any attempt to boil this down to individual component updates failed so far
	CLIPSPACE_TRANSFORM_OGL2WGPU  *  *oglProjection
}
