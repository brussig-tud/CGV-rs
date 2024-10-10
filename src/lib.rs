
//////
//
// Language config
//

// Fuck this convention. Probably the worst aspect of Rust after the lack of a standardized ABI
#![allow(non_snake_case)]



//////
//
// Module definitions
//

/// The module containing utilities used throughout (i.e. not specific to any other module).
pub mod util;

/// The module encapsulating all low-level state of an application.
pub mod state;



//////
//
// Imports
//

// Standard library
/* nothing here yet */

// WASM Bindgen
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Anyhow
use anyhow::Result;

// Tracing
use tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// Winit
use winit::{
	application::ApplicationHandler,
	event::*, event_loop::*, keyboard::*, window::*
};



///////
//
// Classes
//

pub enum UserEvent {
	StateReady(state::State)
}

pub struct App {
	state: Option<state::State>,
	event_loop_proxy: EventLoopProxy<UserEvent>,
	redrawOnceOnWait: bool
}

impl App {
	pub fn new (event_loop: &EventLoop<UserEvent>) -> Self {
		Self {
			state: None,
			event_loop_proxy: event_loop.create_proxy(),
			redrawOnceOnWait: false
		}
	}
}

impl ApplicationHandler<UserEvent> for App
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
			use winit::{dpi::PhysicalSize, platform::web::WindowExtWebSys};

			web_sys::window()
				.and_then(|win| win.document())
				.and_then(|doc| {
					let canvas = Element::from(window.canvas()?);
					canvas.set_attribute("style", "width:100% !important; height:100% !important").unwrap();
					doc.body()?.append_child(&canvas).ok()?;
					Some(())
				})
				.expect("Couldn't append canvas to document body.");

			// Winit prevents sizing with CSS, so we have to set
			// the size manually when on web.
			//let _ = window.request_inner_size(PhysicalSize::new(450, 400));
		}

		#[cfg(target_arch = "wasm32")] {
			let state_future = state::State::new(window);
			let event_loop_proxy = self.event_loop_proxy.clone();
			let future = async move {
				let state = state_future.await;
				assert!(event_loop_proxy
					.send_event(UserEvent::StateReady(state))
					.is_ok());
			};
			wasm_bindgen_futures::spawn_local(future);
		}
		#[cfg(not(target_arch = "wasm32"))] {
			let state = pollster::block_on(state::State::new(window));
			assert!(self
				.event_loop_proxy
				.send_event(UserEvent::StateReady(state))
				.is_ok());
		}
	}

	fn user_event (&mut self, _: &ActiveEventLoop, event: UserEvent) {
		tracing::info!("Application state ready.");
		let UserEvent::StateReady(state) = event;
		self.state = Some(state);
	}

	fn window_event(
		&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent
	){
		match event
		{
			// Main window resize
			WindowEvent::Resized(physical_size) => {
				if let Some(state) = self.state.as_mut() {
					state.resize(physical_size);
				}
				#[cfg(not(target_arch="wasm32"))]{
					self.redrawOnceOnWait = true
				}
			}

			// Application close
			WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
				event: KeyEvent {
					state: ElementState::Pressed,
					physical_key: PhysicalKey::Code(KeyCode::Escape),
					..
				}, ..
			} => {
				tracing::info!("Exiting...");
				event_loop.exit()
			}

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
							event_loop.exit();
						}

						// This happens when the frame takes too long to present
						Err(wgpu::SurfaceError::Timeout) => {
							tracing::warn!("Surface timeout");
						}
					}
				}
			}
			_ => {}
		}
	}

	fn about_to_wait (&mut self, _: &ActiveEventLoop) {
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

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn wasm_start() {
	// Make sure we panic (to JavaScript console) in case run fails
	run().unwrap();
}

pub fn run() -> Result<()>
{
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
	#[cfg(target_arch = "wasm32")]
	{
		use tracing_wasm::{WASMLayer, WASMLayerConfig};

		console_error_panic_hook::set_once();
		let wasm_layer = WASMLayer::new(WASMLayerConfig::default());

		subscriber.with(wasm_layer).init();
	}
	#[cfg(not(target_arch = "wasm32"))]
	{
		let fmt_layer = tracing_subscriber::fmt::Layer::default();
		subscriber.with(fmt_layer).init();
	}
	tracing::info!("Starting...");

	let eventLoop = EventLoop::<UserEvent>::with_user_event().build()?;
	eventLoop.set_control_flow(ControlFlow::Wait);
	let mut app = App::new(&eventLoop);

	eventLoop.run_app(&mut app)?;
	Ok(())
}
