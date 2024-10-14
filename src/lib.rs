
//////
//
// Language config
//

// Eff this convention. Probably the worst aspect of Rust after the lack of a standardized ABI
#![allow(non_snake_case)]

// Eff this "feature" as well.
/*#![allow(unused_must_use)]*/

// And this one...
#![allow(unused_macros)]



//////
//
// Module definitions
//

/// The parent module of all GPU abstractions.
pub mod hal;

/// The module containing all viewing functionality
pub mod view;

/// The module containing utilities used throughout (i.e. not specific to any other module).
pub mod util;

/// The module encapsulating all low-level state of an application.
pub mod state;

/// Make sure we can access glm functionality as such
extern crate nalgebra_glm as glm;



//////
//
// Imports
//

// Standard library
/* nothing here yet */

// WASM Bindgen
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Anyhow library
use anyhow::Result;

// Tracing library
use tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// Winit library
use winit::{
	application::ApplicationHandler,
	event::*, event_loop::*, keyboard::*, window::*
};



///////
//
// Enums
//

/// The type used for our user-defined event.
pub enum UserEvent {
	StateReady(Result<state::State>)
}

/// Enumeration of possible event handling outcomes.
pub enum EventOutcome
{
	/// The event was handled and should be closed. The wrapped `bool` indicates whether a redraw
	/// needs to happen as a result of the processing that was done.
	HandledExclusively(bool),

	/// The event was handled but should not be closed (i.e. subsequent handlers will also receive
	/// it). The wrapped `bool` indicates whether a redraw needs to happen as a result of the
	/// processing that was done.
	HandledDontClose(bool),

	/// The event was not handled.
	NotHandled
}



///////
//
// Classes
//

////
// Application

/// An application that can be [run](Player::run) by a [`Player`].
pub trait Application
{
	/// Called when there is user input that can be processed.
	///
	/// # Arguments
	///
	/// * `event` – The input event that the application should inspect and possibly act upon.
	///
	/// # Returns
	///
	/// The outcome of the [event processing](EventOutcome).
	fn input (&mut self, event: &WindowEvent) -> EventOutcome;

	/// Called when the [player](Player) wants to prepare a new frame for rendering.
	fn update (&mut self);

	/// Called when the [player](Player) needs the application to render its contents.
	fn render (&mut self) -> anyhow::Result<()>;
}


////
// Player

/// The central application host class.
pub struct Player {
	state: Option<state::State>,
	eventLoopProxy: EventLoopProxy<UserEvent>,
	redrawOnceOnWait: bool,

	#[cfg(target_arch = "wasm32")]
	canvas: Option<web_sys::Element>
}

impl Player {
	pub fn new (eventLoop: &EventLoop<UserEvent>) -> Self {
		Self {
			state: None,
			eventLoopProxy: eventLoop.create_proxy(),
			redrawOnceOnWait: false,

			#[cfg(target_arch = "wasm32")]
			canvas: None
		}
	}

	pub fn run (&self, application: &dyn Application) -> anyhow::Result<()> {
		Ok(())
	}

	pub fn exit (&self, eventLoop: &ActiveEventLoop) {
		tracing::info!("Exiting...");
		eventLoop.exit();
	}
}

impl ApplicationHandler<UserEvent> for Player
{
	fn resumed (&mut self, event_loop: &ActiveEventLoop)
	{
		if self.state.is_none() {
			tracing::info!("Main loop created.")
		}
		else {
			tracing::info!("Main loop resumed.");
		}

		let windowAttribs = Window::default_attributes();
		let window = event_loop
			.create_window(windowAttribs)
			.expect("Couldn't create window.");

		#[cfg(target_arch = "wasm32")] {
			use web_sys::Element;
			use winit::platform::web::WindowExtWebSys;

			web_sys::window()
				.and_then(|win| win.document())
				.and_then(|doc| {
					self.canvas = Some(Element::from(window.canvas()?));
					let canvas = self.canvas.as_ref().unwrap();
					doc.body()?.append_child(&canvas).ok()?;
					Some(())
				})
				.expect("Couldn't append canvas to document body.");
		}

		#[cfg(target_arch = "wasm32")] {
			let state_future = state::State::new(window);
			let eventLoopProxy = self.eventLoopProxy.clone();
			let future = async move {
				let state = state_future.await;
				assert!(eventLoopProxy
					.send_event(UserEvent::StateReady(state))
					.is_ok());
			};
			wasm_bindgen_futures::spawn_local(future);
		}
		#[cfg(not(target_arch = "wasm32"))] {
			let state = pollster::block_on(state::State::new(window));
			assert!(self
				.eventLoopProxy
				.send_event(UserEvent::StateReady(state))
				.is_ok());
		}
	}

	/// The user event hook. For now, only used to commit a new (asynchronously initialized) application state.
	fn user_event (&mut self, eventLoop: &ActiveEventLoop, event: UserEvent)
	{
		// Apply newly initialized state
		let UserEvent::StateReady(state) = event;
		match state
		{
			Ok(state) => {
				tracing::info!("Application state ready.");
				self.state = Some(state);

				// WASM, for some reason, needs a resize event for the main surface to become fully
				// configured. Since we need to hook up the size of the canvas hosting the surface to the
				// browser window anyway, this is a good opportunity for dispatching that initial resize.
				#[cfg(target_arch = "wasm32")]
				self.canvas.as_ref().unwrap().set_attribute(
					"style", "width:100% !important; height:100% !important"
				).unwrap();

			}
			Err(error) => {
				tracing::error!("Unable to create application state: {:?}", error);
				eventLoop.exit();
			}
		}
	}

	fn window_event (&mut self, eventLoop: &ActiveEventLoop, _: WindowId, event: WindowEvent)
	{
		match &event
		{
			// Main window resize
			WindowEvent::Resized(physical_size)
			=> {
				if let Some(state) = self.state.as_mut() {
					state.resize(*physical_size);
				}
				#[cfg(not(target_arch="wasm32"))] {
					self.redrawOnceOnWait = true;
				}
			}

			// Application close
			WindowEvent::CloseRequested  => self.exit(eventLoop),

			// Main window redraw
			WindowEvent::RedrawRequested
			=> {
				if let Some(state) = self.state.as_mut() {
					if !state.surfaceConfigured {
						tracing::debug!("Surface not yet configured - skipping redraw!");
						return;
					}
					tracing::debug!("Redrawing");
					state.update();
					match state.render() {
						Ok(()) => {}
						// Reconfigure the surface if it's lost or outdated
						Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
							state.resize(state.size);
						}
						// The system is out of memory, we should probably quit
						Err(wgpu::SurfaceError::OutOfMemory) => {
							tracing::error!("OutOfMemory");
							eventLoop.exit();
						}

						// This happens when the frame takes too long to present
						Err(wgpu::SurfaceError::Timeout) => {
							tracing::warn!("Surface timeout");
						}
					}
				}
			},

			// User interaction
			  WindowEvent::KeyboardInput{..} | WindowEvent::MouseInput{..} | WindowEvent::CursorMoved{..}
			| WindowEvent::MouseWheel{..} | WindowEvent::ModifiersChanged{..}
			=> {
				// GUI gets first dibs
				/* nothing here yet */

				// Camera is next
				if let Some(state) = self.state.as_mut() {
					if state.input(&event) {
						state.window.request_redraw();
						return
					}
				}

				// Exit on ESC
				if let WindowEvent::KeyboardInput {
					event: KeyEvent {
						state: ElementState::Pressed,
						physical_key: PhysicalKey::Code(KeyCode::Escape), ..
					}, ..
				} = event {
					self.exit(eventLoop)
				}
			}

			// We'll ignore this
			_ => {}
		}
	}

	fn about_to_wait (&mut self, _: &ActiveEventLoop)
	{
		if let Some(ref state) = self.state {
			if self.redrawOnceOnWait {
				self.redrawOnceOnWait = false;
				tracing::debug!("Scheduling additional redraw");
				state.window.request_redraw();
			}
		};
	}
}



//////
//
// Functions
//

/// The entry point from the browser for WASM builds.
#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn wasm_start() {
	// Make sure we panic (to the JavaScript console) in case run fails
	run().unwrap();
}

/// The main function for handling control flow, including initialization, window creation and the main event loop.
pub fn run() -> Result<()>
{
	// Set up logging
	let mut envFilterBuilder = EnvFilter::builder();
	#[cfg(debug_assertions)] {
		envFilterBuilder = envFilterBuilder.with_default_directive(tracing::Level::DEBUG.into());
	}
	#[cfg(not(debug_assertions))] {
		envFilterBuilder = envFilterBuilder.with_default_directive(tracing::Level::INFO.into());
	}
	let env_filter = envFilterBuilder
		.from_env_lossy()
		.add_directive("wgpu_core::device::resource=warn".parse()?);
	let subscriber = tracing_subscriber::registry().with(env_filter);
	#[cfg(target_arch = "wasm32")] {
		use tracing_wasm::{WASMLayer, WASMLayerConfig};

		console_error_panic_hook::set_once();
		let wasm_layer = WASMLayer::new(WASMLayerConfig::default());

		subscriber.with(wasm_layer).init();
	}
	#[cfg(not(target_arch = "wasm32"))] {
		let fmt_layer = tracing_subscriber::fmt::Layer::default();
		subscriber.with(fmt_layer).init();
	}

	tracing::info!("Starting...");

	// Launch main event loop. Most initialization is event-driven and will happen in there.
	let eventLoop = EventLoop::<UserEvent>::with_user_event().build()?;
	eventLoop.set_control_flow(ControlFlow::Wait);
	let mut app = Player::new(&eventLoop);
	eventLoop.run_app(&mut app)?;

	// Done!
	Ok(())
}
