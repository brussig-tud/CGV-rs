
//////
//
// Language config
//

// No point allowing unstable features if we still get warnings
#![allow(incomplete_features)]

// Eff this convention. Probably the worst aspect of Rust after the lack of a standardized ABI
#![allow(non_snake_case)]

// Experimental language features
#![feature(let_chains)]          // required for view::FoV
#![feature(generic_const_exprs)] // required for util::Phony



//////
//
// Module definitions
//

// The module implementing the Player
mod player;
pub use player::{Player, InputEvent, EventOutcome, RenderSetup, ManagedBindGroupLayouts}; // re-export

// The module encapsulating all low-level graphics objects.
mod context;
pub use context::Context; // re-export

// The module encapsulating rendering-related higher-level managed render state (common uniform buffers etc.)
pub mod renderstate;
pub use renderstate::RenderState; // re-export

/// The parent module of all GPU abstractions.
pub mod hal;

/// The module containing all viewing functionality.
pub mod view;

/// The module providing various reusable UI components
pub mod gui;

/// The module containing utilities used throughout (i.e. not specific to any other module).
#[allow(unused)]         // some of our utils are mainly useful for clients
pub mod util;

/// Make sure we can access glm functionality as such
pub extern crate nalgebra_glm as glm;

/// Re-export important 3rd party libraries/library components
pub use tracing;
pub use anyhow::Result as Result;
pub use anyhow::Error as Error;
pub use anyhow::anyhow as anyhow;
pub mod time {
	pub use web_time::{Instant as Instant, Duration as Duration};
}
pub use eframe::wgpu as wgpu;



//////
//
// Imports
//

// Standart library
use std::any::Any;

// Ctor library
#[cfg(not(target_arch="wasm32"))]
use ctor;

// Tracing library
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};



//////
//
// Vault
//

// Populate the vault
#[cfg(not(target_arch="wasm32"))]
#[ctor::ctor]
fn initTracingProxy () {
	initTracing()
}

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
	Custom(Box<dyn Any>)
}

/// T.b.d.
pub struct GlobalPassDeclaration<'info> {
	pub info: GlobalPassInfo<'info>,
	pub completionCallback: Option<Box<dyn FnMut(&Context, u32)>>
}

/// T.b.d.
pub struct GlobalPassInfo<'rs> {
	pub pass: GlobalPass,
	pub renderState: &'rs RenderState,
	pub clearColor: wgpu::Color,
	pub depthClearValue: f32
}



///////
//
// Traits
//

////
// Application

/// An application that can be [run](Player::run) by a [`Player`].
pub trait Application
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
	/// * `context` – The graphics context.
	/// * `player` – Access to the *CGV-rs* [`Player`] instance, useful for more involved actions.
	///
	/// # Returns
	///
	/// `Ok` if successful, or some descriptive error detailing the failure if not.
	fn preInit (&mut self, context: &Context, player: &Player) -> Result<()>;

	/// Called when the [`Player`] changed global render state, e.g. because a new [`view::Camera`] became active. Since
	/// this could mean framebuffers with a different format and depth testing strategy, applications should (re-)create
	/// their pipelines accordingly. The `Player`] guarantees that this will be called at least once before the
	/// application is asked to render its contribution to the scene.
	///
	/// **ToDo:** Make the framework detect compatible changes and only notify for incompatible global pass changes
	///
	/// # Arguments
	///
	/// * `context` – The graphics context.
	/// * `renderSetup` – The global render setup of the *CGV-rs* [`Player`].
	/// * `globalPasses` – The list of global passes the application will need to render to.
	/// * `player` – Access to the *CGV-rs* [`Player`] instance, useful for more involved actions.
	fn recreatePipelines (
		&mut self, context: &Context, renderSetup: &RenderSetup, globalPasses: &[&GlobalPassInfo], _: &Player
	);

	/// Called once on creation of the application, after it was asked to create its pipelines.
	///
	/// # Arguments
	///
	/// * `context` – The graphics context.
	/// * `player` – Access to the *CGV-rs* [`Player`] instance, useful for more involved actions.
	///
	/// # Returns
	///
	/// `Ok` if successful, or some descriptive error detailing the failure if not.
	fn postInit (&mut self, context: &Context, player: &Player) -> Result<()>;

	/// Called when there is user input that can be processed.
	///
	/// # Arguments
	///
	/// * `event` – The input event that the application should inspect and possibly act upon.
	/// * `player` – Access to the *CGV-rs* [`Player`] instance, useful for more involved reactions to input.
	///
	/// # Returns
	///
	/// The [outcome](EventOutcome) of the event processing.
	fn input (&mut self, event: &InputEvent, player: &Player) -> EventOutcome;

	/// Called when the main framebuffer was resized.
	///
	/// # Arguments
	///
	/// * `context` – The graphics context, useful e.g. to re-create resources when they're affected by the resize.
	/// * `newSize` – The new main framebuffer size, in pixels.
	/// * `player` – Access to the *CGV-rs* [`Player`] instance, useful for more involved reactions to resizing.
	fn resize (&mut self, context: &Context, newSize: glm::UVec2, player: &Player);

	/// Called when the [player](Player) wants to prepare a new frame for rendering.
	///
	/// # Arguments
	///
	/// * `context` – The graphics context for rendering.
	/// * `player` – Access to the *CGV-rs* [`Player`] instance, useful for more involved updates.
	///
	/// # Returns
	///
	/// `true` when the application deems a scene redraw is required, `false` otherwise.
	fn update (&mut self, context: &Context, player: &Player) -> bool;

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
}


////
// ApplicationFactory

pub trait ApplicationFactory {
	/// Create an instance of the target application.
	///
	/// # Arguments
	///
	/// * `context` – The graphics context.
	/// * `renderSetup` – The global render setup of the *CGV-rs* [`Player`].
	///
	/// # Returns
	///
	/// A boxed instance of the application if successful, or some descriptive error detailing the failure if no
	/// instance could be created.
	fn create (&self, context: &Context, renderSetup: &RenderSetup) -> Result<Box<dyn Application>>;
}
