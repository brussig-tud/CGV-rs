
//////
//
// Language config
//

// Fuck this convention. Probably the worst aspect of Rust after the lack of a standardized ABI
#![allow(non_snake_case)]



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

#[derive(Default)]
pub struct App {
	window: Option<Window>
}

impl ApplicationHandler for App {
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

		self.window = Some(window);
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		let Some(ref window) = self.window else {
			return;
		};
		if window_id != window.id() {
			return;
		}
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

/*pub fn run() -> Result<()>
{
	////
	// Init

	// Logging
	cfg_if::cfg_if! {
		if #[cfg(target_arch = "wasm32")] {
			std::panic::set_hook(Box::new(console_error_panic_hook::hook));
			console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
		} else {
			env_logger::Builder::from_env(
				env_logger::Env::default().default_filter_or("info")
			).init();
		}
	}

	// Events
	let eventLoop = EventLoop::new()?;
	eventLoop.set_control_flow(ControlFlow::Wait);

	// Main window
	let window = WindowBuilder::new().build(&eventLoop)?;

	// WASM: Create canvas for Winit
	#[cfg(target_arch = "wasm32")]
	{
		// Winit prevents sizing with CSS, so we have to set
		// the size manually when on web.
		use winit::dpi::PhysicalSize;
		let _ = window.request_inner_size(PhysicalSize::new(450, 400));

		use winit::platform::web::WindowExtWebSys;
		web_sys::window()
			.and_then(|win| win.document())
			.and_then(|doc| {
				let dst = doc.get_element_by_id("wasm-example")?;
				let canvas = web_sys::Element::from(window.canvas()?);
				dst.append_child(&canvas).ok()?;
				Some(())
			})
			.expect("Couldn't append canvas to document body.");
	}



	////
	// Main event loop

	// Dispatch
	eventLoop.run(move |event, controlFlow| match event
	{
		Event::WindowEvent {
			ref event,
			window_id,
		} if window_id == window.id() => match event {
			WindowEvent::CloseRequested => {
				log!(Level::Info, "Request to close main window {:?} received.", window_id);
				controlFlow.exit()
			},
			WindowEvent::KeyboardInput {
				event:
				KeyEvent {
					state: ElementState::Pressed,
					physical_key: PhysicalKey::Code(KeyCode::Escape),
					..
				},
				..
			} => {
				log!(Level::Info, "Escape key pressed – exiting.");
				controlFlow.exit()
			},
			_ =>
				log!(Level::Info, "Not handling event: {:?}", event)
		},
		_ =>
			log!(Level::Info, "Not handling event: {:?}", event)
	})?;


	////
	// Shutdown

	// Done! Exit successfully.
	Ok(())
}*/
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

	let event_loop = EventLoop::new()?;
	let mut app = App::default();

	event_loop.run_app(&mut app)?;
	Ok(())
}
