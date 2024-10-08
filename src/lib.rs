
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
use std::sync::Arc;

// WASM Bindgen
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Anyhow
use anyhow::Result;

// Tracing
use tracing::Level;
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

enum UserEvent {
	StateReady(state::State)
}

pub struct App {
	state: Option<state::State>,
	event_loop_proxy: EventLoopProxy<UserEvent>
}

impl App {
	pub fn new(event_loop: &EventLoop<UserEvent>) -> Self {
		Self {
			state: None,
			event_loop_proxy: event_loop.create_proxy(),
		}
	}
}

impl ApplicationHandler<UserEvent> for App {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		tracing::info!("Resumed");
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
			self.state = Some(pollster::block_on(state::State::new(window)));
		}
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		match event {
			WindowEvent::CloseRequested
			| WindowEvent::KeyboardInput {
				event:
				KeyEvent {
					state: ElementState::Pressed,
					physical_key: PhysicalKey::Code(KeyCode::Escape),
					..
				},
				..
			} => {
				tracing::info!("Exited!");
				event_loop.exit()
			}
			_ => {}
		}
	}
}



//////
//
// Functions
//

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn wasm_start() {
	// For sanity-checking whether the module started properly in the browser
	tracing::info!("WASM started!");

	// Make sure we panic (to JavaScript console) in case run fails
	run().unwrap();
}

pub fn run() -> Result<()>
{
	let env_filter = EnvFilter::builder()
		.with_default_directive(Level::INFO.into())
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

	let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
	let mut app = App::new(&event_loop);

	event_loop.run_app(&mut app)?;
	Ok(())
}
