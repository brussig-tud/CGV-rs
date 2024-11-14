
//////
//
// Language config
//

// No point allowing unstable features if we still get warnings
#![allow(incomplete_features)]

// Required for util::Phony
#![feature(generic_const_exprs)]

// Eff this convention. Probably the worst aspect of Rust after the lack of a standardized ABI
#![allow(non_snake_case)]

// And this one... the macros are there for clients! Why should the library have to use every single one? WTF...
#![allow(unused_macros)]



//////
//
// Module definitions
//

// The module implementing the Player
mod player;
pub use player::{Player, EventOutcome, ManagedBindGroupLayouts, RenderSetup}; // re-export

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

/// The module containing utilities used throughout (i.e. not specific to any other module).
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

pub struct GlobalPassDeclaration<'rs>
{
	pub pass: GlobalPass,
	pub renderState: &'rs mut RenderState<'rs>,
	pub completionCallback: Option<Box<dyn FnMut(&'static Context, u32)>>
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
	/*/// Called when there is user input that can be processed.
	///
	/// # Arguments
	///
	/// * `event` – The input event that the application should inspect and possibly act upon.
	///
	/// # Returns
	///
	/// The outcome of the [event processing](EventOutcome).
	fn onInput (&mut self, event: &WindowEvent) -> EventOutcome;

	/// Called when the main window surface was resized.
	///
	/// # Arguments
	///
	/// * `newSize` – The new main window surface size, in pixels.
	fn onResize (&mut self, newSize: &glm::UVec2);

	/// Called when the [player](Player) wants to prepare a new frame for rendering.
	fn update (&mut self);

	/// Called when the [player](Player) needs the application to render its contents.
	///
	/// # Arguments
	///
	/// * `device` – The active device for rendering.
	/// * `queue` – A queue from the active device for submitting commands.
	/// * `globalPass` – Identifies the global render pass over the scene that spawned this call to `render`.
	fn render (&mut self, context: &Context, renderState: &RenderState, globalPass: &GlobalPass) -> anyhow::Result<()>;*/
}


////
// ApplicationFactory

pub trait ApplicationFactory {
	fn create (self, context: &Context/*, renderSetup: &RenderSetup*/) -> Result<Box<dyn Application>>;
}
