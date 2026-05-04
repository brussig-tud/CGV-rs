
//////
//
// Language config
//

// Eff this convention.
#![allow(non_snake_case)]



//////
//
// Module definitions
//

// The module implementing the Player
pub mod player;
pub use player::{Player, InputEvent, EventOutcome, RenderSetup, ManagedBindGroupLayouts}; // re-export

// The module encapsulating all low-level graphics objects.
mod context;
pub use context::Context; // re-export

// The module encapsulating rendering-related higher-level managed render state (common uniform buffers etc.)
pub mod renderstate;
pub use renderstate::RenderState; // re-export

/// The parent module of all GPU abstractions.
pub mod hal;

/// The module providing implementations for various common gpu compute tasks.
pub mod gpu;

/// The module containing high-level rendering facilities.
pub mod renderer;
pub use renderer::Renderer; // re-export

/// The module containing all viewing functionality.
pub mod view;

/// The module providing various reusable UI components
pub mod gui;

/// The module providing functionality related to data handling
pub mod data;

/// Re-export cgv-shader
pub use cgv_shader as shader;

/// Make sure we can access glm functionality as such
pub extern crate nalgebra_glm as glm;

/// Re-export important 3rd party libraries/library components
pub use tracing;
pub use anyhow::{anyhow, Result, Error};
pub mod time {
	pub use web_time::{Instant as Instant, Duration as Duration};
}
pub use eframe::{wgpu as wgpu, egui as egui};
pub use egui_extras;

/// Unit tests
#[cfg(test)]
mod tests;



//////
//
// Imports
//

// Standard library
use std::any::Any;

// Ctor library
#[cfg(not(target_arch="wasm32"))]
use ctor;

// Tracing library
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// CGV imports
pub use cgv_util as util; // re-export
pub use cgv_runenv as run; // re-export



//////
//
// Vault
//

// Populate the vault.
#[cfg(not(target_arch="wasm32"))]
#[ctor::ctor(unsafe)]
fn initTracingProxy ()
{
	#[cfg(target_os="windows")]
		let ansiTermResult = ansi_term::enable_ansi_support();
	initTracing();
	#[cfg(target_os="windows")]
		if let Err(err) = ansiTermResult {
			tracing::error!("Failed to enable ANSI terminal support: {err}");
		}
}

// Encapsulate common init tasks for the tracing crate.
fn initTracing ()
{
	// Set up logging
	let mut envFilterBuilder = EnvFilter::builder();
	#[cfg(debug_assertions)] {
		envFilterBuilder = envFilterBuilder.with_default_directive(tracing::Level::DEBUG.into());
	}
	#[cfg(not(debug_assertions))] {
		envFilterBuilder = envFilterBuilder.with_default_directive(tracing::Level::INFO.into());
	}
	let envFilter = envFilterBuilder
		.from_env_lossy()
		.add_directive("wgpu_core::device::resource=warn".parse().expect(
			"Failed to set up logging/tracing facilities!"
		));

	let subscriber = tracing_subscriber::registry().with(envFilter);
	#[cfg(target_arch="wasm32")] {
		use tracing_wasm::{WASMLayer, WASMLayerConfig};

		console_error_panic_hook::set_once();
		let wasm_layer = WASMLayer::new(WASMLayerConfig::default());

		subscriber.with(wasm_layer).init();
	}
	#[cfg(not(target_arch="wasm32"))] {
		let fmt_layer = tracing_subscriber::fmt::Layer::default();
		subscriber.with(fmt_layer).init();
	}
}



///////
//
// Enums and structs
//

/// The common color type.<br />
/// **TODO**: put into to-be-done `media` module/crate.
pub type RGBA = egui::ecolor::Rgba;

/// Holds information about the eye(s) in a stereo render pass.
#[derive(Debug)]
pub struct StereoEye {
	/// The index of the eye currently being rendered.
	pub current: u32,

	/// The maximum eye index in the current stereo render.
	pub max: u32
}

/// Enumerates the kinds of global render passes over the scene.
#[derive(Debug)]
pub enum GlobalPass
{
	/// A simple, straight-to-the-target global pass.
	Simple,

	/// A stereo pass - the encapsulated value indicates which eye exactly is being rendered currently.
	Stereo(StereoEye),

	/// A custom pass, with a custom value.
	Custom(Box<dyn Any + Send>)
}
impl GlobalPass {
	/// Construct a `GlobalPass::Stereo` value for the given eye index, with the [`StereoEye::max`] field set to a
	/// *don't care* value. Useful for interacting with various [`view::Camera`] APIs that require you to state a global
	/// pass for the operation.
	///
	/// # Arguments
	///
	/// * `eye` – The eye index the stereo pass should refer to.
	///
	/// # Returns
	///
	/// A `GlobalPass::Stereo` value that refers to the specified eye index.
	///
	/// # Example
	///
	/// **TODO: add once [`view::Camera`] is overhauled**
	pub fn stereoQuery (eye: u32) -> Self {
		Self::Stereo(StereoEye { current: eye, max: u32::default() })
	}
}

/// Configuration for a global render pass.
pub struct GlobalPassInfo {
	pub pass: GlobalPass,
	/// Index of the associated [`RenderState`] within [`GlobalPasses::renderStates`].
	pub renderState: u32,
	pub clearColor: wgpu::Color,
	pub depthClearValue: f32,
	completionCallback: std::cell::Cell<Option<Box<dyn FnMut(&Context, u32) + Send>>>,
}

/// Global passes and their associated render state, defined by a camera.
pub struct GlobalPasses<'cam>
{
	pub info: &'cam [GlobalPassInfo],
	pub renderStates: &'cam [RenderState],
}


///////
//
// Traits
//

/// Base trait for different kinds of objects stored in the [`Player`], such as [applications](Application) and
/// [camera interactors](view::CameraInteractor).
/// To allow runtime downcasts, implementors must be `'static`, i.e. not have any lifetime parameters.
pub trait Component: Any + Send {}
impl<T> Component for T where T: Any + Send + ?Sized {}

////
// Application

/// An application that can be [run](Player::run) by a [`Player`].
pub trait Application: Component
{
	/// Report a short title for the application that can be displayed in the application tab bar of the [`Player`].
	///
	/// # Returns
	///
	/// A string slice containing a short descriptive title for the application.
	fn title (&self) -> &str;

	/// Called once on creation of the application, before it's asked to create its pipelines.
	///
	/// # Arguments
	///
	/// * `player` – The global *CGV-rs* [`Player`] instance.
	///
	/// # Returns
	///
	/// `Ok` if successful, or some descriptive error detailing the failure if not.
	fn preInit (&mut self, player: &mut Player) -> Result<()>;

	/// Called when the [`Player`] changed global render state, e.g. because a new [`view::Camera`] became active. Since
	/// this could mean framebuffers with a different format and depth testing strategy, applications should (re-)create
	/// their pipelines accordingly. The [`Player`] guarantees that this will be called at least once before the
	/// application is asked to render its contribution to the scene.
	///
	/// **ToDo:** Make the framework detect compatible changes and only notify for incompatible global pass changes
	///
	/// # Arguments
	///
	/// * `context` – The graphics context.
	/// * `renderSetup` – The global render setup of the *CGV-rs* [`Player`].
	/// * `globalPasses` – The list of global passes the application will need to render to.
	fn recreatePipelines (&mut self, context: &Context, renderSetup: &RenderSetup, globalPasses: &GlobalPasses);

	/// Called once on creation of the application, after it was asked to create its pipelines.
	///
	/// # Arguments
	///
	/// * `player` – The global *CGV-rs* [`Player`] instance.
	///
	/// # Returns
	///
	/// `Ok` if successful, or some descriptive error detailing the failure if not.
	fn postInit (&mut self, player: &mut Player) -> Result<()>;

	/// Called when there is user input that can be processed.
	///
	/// # Arguments
	///
	/// * `event` – The input event that the application should inspect and possibly act upon.
	/// * `player` – The global *CGV-rs* [`Player`] instance.
	/// * `this` - Provides access to `self` from outside this function, e.g. in an asynchronous callback.
	///
	/// # Returns
	///
	/// The [outcome](EventOutcome) of the event processing.
	fn input (&mut self, event: &InputEvent, player: &mut Player, this: player::Handle) -> EventOutcome;

	/// Called when the main framebuffer was resized.
	///
	/// # Arguments
	///
	/// * `context` – The graphics context, useful e.g. to re-create resources when they're affected by the resize.
	/// * `newSize` – The new main framebuffer size, in pixels.
	fn resize (&mut self, context: &Context, newSize: glm::UVec2);

	/// Called when the [player](Player) wants to prepare a new frame for rendering.
	///
	/// # Arguments
	///
	/// * `context` – The graphics context for rendering.
	/// * `player` – The global *CGV-rs* [`Player`] instance.
	/// * `this` - Provides access to `self` from outside this function, e.g. in an asynchronous callback.
	///
	/// # Returns
	///
	/// `true` when the application deems a scene redraw is required, `false` otherwise.
	fn update (&mut self, player: &mut Player, this: player::Handle) -> bool;

	/// Called when the [player](Player) is about to ask the application to render its contribution to the scene within
	/// a [global render pass](GlobalPassInfo).
	///
	/// # Arguments
	///
	/// * `context` – The graphics context for rendering.
	/// * `renderState` – The render state for the ongoing global render pass over the scene.
	/// * `globalPass` – Identifies the global render pass over the scene that spawned this call to `render`.
	///
	/// # Returns
	///
	/// `Some` array of command buffers containing any commands the application might need to perform before being able
	/// to render, or `None` if no preparation is required.
	fn prepareFrame (&mut self, context: &Context, renderState: &RenderState, globalPass: &GlobalPass)
		-> Option<Vec<wgpu::CommandBuffer>>;

	/// Called when the [player](Player) needs the application to render its contents.
	///
	/// # Arguments
	///
	/// * `context` – The graphics context for rendering.
	/// * `renderState` – The render state for the ongoing global render pass over the scene.
	/// * `renderPass` – The *WGPU* render pass to the internally managed framebuffer that the application can add
	///                  draw calls to.
	/// * `globalPass` – Identifies the global pass over the scene that the render pass is for.
	fn render (
		&mut self, context: &Context, renderState: &RenderState, managedRenderPass: &mut wgpu::RenderPass,
		globalPass: &GlobalPass
	) -> Option<Vec<wgpu::CommandBuffer>>;

	/// Called when the [player](Player) needs the application to define its graphical main UI (which goes in the
	/// player's application panel). Independent or free-floating UI should be drawn in [`Application::freeUi`] instead.
	///
	/// # Arguments
	///
	/// * `ui` – The *egui* UI object on which to define the application graphical UI.
	/// * `player` – The global *CGV-rs* [`Player`] instance.
	fn ui (&mut self, ui: &mut egui::Ui, ps: &mut Player);

	/// Called when the [player](Player) asks the application to define its free/independent UI (e.g. floating windows
	/// that should stay open even if the app loses player focus). Can be left unimplemented when not needed.
	///
	/// # Arguments
	///
	/// * `ui` – The *egui* UI object on which to define the application graphical UI.
	/// * `player` – The global *CGV-rs* [`Player`] instance.
	#[expect(unused_variables)]
	fn freeUi (&mut self, ui: &mut egui::Ui, ps: &mut Player) {}
}


////
// ApplicationFactory

/// An object that can create instances of an applications that can be run by the [`Player`].
pub trait ApplicationFactory
{
	/// Create an instance of the target application.
	///
	/// # Arguments
	///
	/// * `context` – The graphics context.
	/// * `renderSetup` – The global render setup of the *CGV-rs* [`Player`].
	/// * `environment` – Runtime environment information (like the shader path) for the to-be-created application.
	///
	/// # Returns
	///
	/// A boxed instance of the application if successful, or some descriptive error detailing the failure if no
	/// instance could be created.
	fn create (
		&self, context: &Context, renderSetup: &RenderSetup, environment: run::Environment
	) -> Result<Box<dyn Application>>;
}
impl<F> ApplicationFactory for F
where
	F: for<'a, 'b> Fn(&'a Context, &'b RenderSetup, run::Environment)->Result<Box<dyn Application>>
{
	fn create (&self, context: &Context, renderSetup: &RenderSetup, environment: run::Environment)
	-> Result<Box<dyn Application>> {
		self(context, renderSetup, environment)
	}
}



///////
//
// Functions
//

///
#[cfg(feature="slang_runtime")]
pub fn obtainShaderCompileEnvironment () -> shader::compile::Environment<shader::slang::EnvModule>
{
	// Imports we only need here, when the feature `slang_runtime` is enabled
	use std::sync::LazyLock;
	use util::{uuid::Uuid, unique::Realm};

	// Statically keep the environment in memory
	static SHADER_LIB_ENVIRONMENT: LazyLock<shader::compile::Environment<shader::slang::EnvModule>> = LazyLock::new(||
		shader::compile::Environment::deserialize(util::sourceGeneratedBytes!("/coreshaderlib.env")).expect(
			"core shader library environment could not be deserialized"
		)
	);
	static SHADER_LIB_COUNTER: util::unique::RealmU32 = util::unique::RealmU32::one();
	let newCount = SHADER_LIB_COUNTER.newEntity();
	let newUuid = Uuid::from_u128(SHADER_LIB_ENVIRONMENT.uuid().as_u128() + newCount as u128);
	SHADER_LIB_ENVIRONMENT.cloneWithNewUuid(
		newUuid, &format!("{}_instance{newCount}", SHADER_LIB_ENVIRONMENT.label())
	)
}
