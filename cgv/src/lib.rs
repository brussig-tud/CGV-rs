
//////
//
// Language config
//

// Eff this convention. Probably the worst aspect of Rust after the lack of a standardized ABI
#![allow(non_snake_case)]

// Eff this "feature" as well.
/*#![allow(unused_must_use)]*/

// And this one... the macros are there for clients! Why should the library have to use every single one? WTF...
#![allow(unused_macros)]



//////
//
// Module definitions
//

/// Separate module exposing all kinds of information and CGV-specific functionality for use by dependent build scripts.
pub mod build;

// The module encapsulating all low-level graphics state.
mod context;
pub use context::Context; // re-export

// The module encapsulating rendering-related higher-level managed render state (common uniform buffers etc.)
mod renderstate;
pub use renderstate::RenderState; // re-export

/// The parent module of all GPU abstractions.
pub mod hal;

/// The module containing all viewing functionality
pub mod view;

/// The module containing utilities used throughout (i.e. not specific to any other module).
pub mod util;

/// Make sure we can access glm functionality as such
pub extern crate nalgebra_glm as glm;

/// Re-export important 3rd party libraries/library components
pub use tracing;
pub use anyhow::Result as Result;
pub use winit::event;
pub use wgpu;



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

// Winit library
use winit::{
	application::ApplicationHandler,
	event::*, event_loop::*, keyboard::*, window::*
};

// Local imports
use crate::{context::*, renderstate::*};
use crate::hal::RenderTarget;
//////
//
// Vault
//

// Populate the vault
#[cfg(not(target_arch = "wasm32"))]
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
}



///////
//
// Enums
//

// The type used for our user-defined event.
enum UserEvent {
	ContextReady(Result<Context>)
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

pub struct GlobalPassDeclaration<'a>
{
	pub pass: GlobalPass,
	pub renderTarget: Option<&'a RenderTarget>,
	pub completionCallback: Option<Box<dyn FnMut(&'static Context, &GlobalPass)>>
}



///////
//
// Classes
//

////
// ApplicationFactory

pub trait ApplicationFactory {
	fn create(&self, context: &Context, renderState: &RenderState) -> Result<Box<dyn Application>>;
}


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
	fn onInput (&mut self, event: &WindowEvent) -> EventOutcome;

	/// Called when the main window surface was resized.
	///
	/// # Arguments
	///
	/// * `newSize` – The new main window surface size, in pixels.
	fn onResize (&mut self, newSize: &glm::Vec2);

	/// Called when the [player](Player) wants to prepare a new frame for rendering.
	fn update (&mut self);

	/// Called when the [player](Player) needs the application to render its contents.
	///
	/// # Arguments
	///
	/// * `device` – The active device for rendering.
	/// * `queue` – A queue from the active device for submitting commands.
	/// * `globalPass` – Identifies the global render pass over the scene that spawned this call to `render`.
	fn render (&mut self, context: &Context, renderState: &RenderState, globalPass: &GlobalPass)
		-> anyhow::Result<()>;
}


////
// Player

/// The central application host class.
pub struct Player
{
	eventLoop: Option<EventLoop<UserEvent>>,
	eventLoopProxy: EventLoopProxy<UserEvent>,

	#[cfg(target_arch = "wasm32")]
	canvas: Option<web_sys::Element>,

	context: Option<Context>,
	redrawOnceOnWait: bool,

	applicationFactory: Option<Box<dyn ApplicationFactory>>,
	application: Option<Box<dyn Application>>,

	renderState: Option<renderstate::RenderState>,

	camera: Option<Box<dyn view::Camera>>,
	cameraInteractor: Box<dyn view::CameraInteractor>
}
unsafe impl Send for Player {}
unsafe impl Sync for Player {}

impl Player
{
	pub fn new () -> Result<Self>
	{
		// In case of WASM, make sure the JavaScript console is set up for receiving log messages first thing (for non-
		// WASM targets, tracing/logging is already being set up at module loading time)
		#[cfg(target_arch="wasm32")]
		initTracing();

		// Log that we have begun the startup process
		tracing::info!("Starting...");

		// Launch main event loop. Most initialization is event-driven and will happen in there.
		let eventLoop = EventLoop::<UserEvent>::with_user_event().build()?;
		eventLoop.set_control_flow(ControlFlow::Wait);

		// Done, now construct
		Ok(Self {
			eventLoopProxy: eventLoop.create_proxy(),
			eventLoop: Some(eventLoop),

			#[cfg(target_arch = "wasm32")]
			canvas: None,

			context: None,
			redrawOnceOnWait: false,

			applicationFactory: None,
			application: None,

			renderState: None,

			camera: None,
			cameraInteractor: Box::new(view::OrbitCamera::new())
		})
	}

	pub fn run<F: ApplicationFactory + 'static> (mut self, applicationFactory: F) -> Result<()>
	{
		// Set the application factory
		self.applicationFactory = Some(Box::new(applicationFactory));

		// Run the event loop
		self.eventLoop.take().unwrap().run_app(&mut self)?;

		// Done!
		Ok(())
	}

	fn context (&mut self) -> &mut Context {
		self.context.as_mut().unwrap()
	}

	// Performs the actual redrawing logic
	fn redraw (&mut self) -> Result<()>
	{
		// Obtain context
		let context = util::statify(self.context.as_ref().unwrap());

		// Update the camera
		self.cameraInteractor.update();

		// Determine the global passes we need to make
		let passes = self.camera.as_ref().unwrap().declareGlobalPasses();
		for pass in passes
		{
			/* Update managed render state */ {
				// Obtain render state reference
				let rs = self.renderState.as_mut().unwrap();

				// Uniforms
				// - viewing
				rs.viewingUniforms.data.projection = *self.cameraInteractor.projection();
				rs.viewingUniforms.data.view = *self.cameraInteractor.view();
				rs.viewingUniforms.upload(context, true);

				// Commit to GPU
				context.queue.submit([]);
			};

			// Finish the pass
			if let Some(callback) = util::mutify(&pass.completionCallback) {
				callback(context, &pass.pass);
			}
		}

		if let Some(application) = self.application.as_mut() {
			application.render(&context, &self.renderState.as_ref().unwrap(), &GlobalPass::Simple)?;
		}

		Ok(())
	}

	pub fn withContext<Closure: FnOnce(&'static mut Player, &'static mut Context)> (&mut self, codeBlock: Closure) {
		let this = util::mutify(self);
		codeBlock(util::mutify(self), this.context.as_mut().unwrap());
	}

	pub fn withRenderState<Closure: FnOnce(&'static mut Player, &'static mut RenderState)> (
		&mut self, codeBlock: Closure
	){
		let this = util::mutify(self);
		codeBlock(util::mutify(self), this.renderState.as_mut().unwrap());
	}

	pub fn withContextAndRenderState<Closure: FnOnce(&'static mut Player, &'static mut Context, &'static mut RenderState)> (
		&mut self, codeBlock: Closure
	){
		let this = util::mutify(self);
		codeBlock(util::mutify(self), this.context.as_mut().unwrap(), this.renderState.as_mut().unwrap());
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
		if self.context.is_none() {
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
			let state_future = Context::new(window);
			let eventLoopProxy = self.eventLoopProxy.clone();
			let future = async move {
				let state = state_future.await;
				assert!(eventLoopProxy
					.send_event(UserEvent::ContextReady(state))
					.is_ok());
			};
			wasm_bindgen_futures::spawn_local(future);
		}
		#[cfg(not(target_arch = "wasm32"))] {
			let context = pollster::block_on(Context::new(window));
			assert!(self
				.eventLoopProxy
				.send_event(UserEvent::ContextReady(context))
				.is_ok());
		}
	}

	/// The hook for all custom events.
	fn user_event (&mut self, eventLoop: &ActiveEventLoop, event: UserEvent)
	{
		// Apply newly initialized state
		match event
		{
			UserEvent::ContextReady(contextCreationResult)
			=> match contextCreationResult {
				Ok(context)
				=> {
					// Commit context
					tracing::info!("Graphics context ready.");
					self.context = Some(context);
					let context = self.context.as_ref().unwrap();

					// WASM, for some reason, needs a resize event for the main surface to become fully configured.
					// Since we need to hook up the size of the canvas hosting the surface to the browser window anyway,
					// this is a good opportunity for dispatching that initial resize.
					#[cfg(target_arch="wasm32")]
					self.canvas.as_ref().unwrap().set_attribute(
						"style", "width:100% !important; height:100% !important"
					).unwrap();

					// On non-WASM on the other hand, the surface is correctly configured for the initial size so we
					// need to inform the camera separately. However, we do need to schedule a single redraw to not get
					// garbage on the screen as the surface is displayed for the first time for some reason...
					#[cfg(not(target_arch="wasm32"))] {
						self.cameraInteractor.resize(&glm::vec2(context.size.width as f32, context.size.height as f32));
						self.redrawOnceOnWait = true;
					}

					/* Create base render state */ {
						let descriptor = wgpu::TextureDescriptor {
							label: None,
							size: wgpu::Extent3d {width: 1, height: 1, depth_or_array_layers: 1},
							mip_level_count: 1,
							sample_count: 1,
							dimension: wgpu::TextureDimension::D2,
							format: wgpu::TextureFormat::Rgba8Unorm,
							usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
							view_formats: &[],
						};
						self.renderState = Some(RenderState::new(
							context,
							ColorAttachment::Surface(util::statify(
								&context.device.create_texture(&descriptor).create_view(
									&wgpu::TextureViewDescriptor::default()
								)
							)),
							Some(hal::DepthStencilFormat::D32)
						));
					}

					// Initialize camera
					self.camera = Some(view::MonoCamera::new(context, None, Some("MainCamera")));

					// Create the application
					let appCreationResult = self.applicationFactory.take().unwrap().create(
						context, self.renderState.as_ref().unwrap()
					);
					match appCreationResult {
						Ok(application) => self.application = Some(application),
						Err(error) => {
							tracing::error!("Failed to create application: {:?}", error);
							self.exit(eventLoop);
						}
					}
				}
				Err(error) => {
					tracing::error!("Graphics context initialization failure: {:?}", error);
					self.exit(eventLoop);
				}
			}
		}
	}

	fn window_event (&mut self, eventLoop: &ActiveEventLoop, _: WindowId, event: WindowEvent)
	{
		match &event
		{
			// Main window resize
			WindowEvent::Resized(newPhysicalSize)
			=> {
				if self.context.is_some() {
					let newSize = glm::vec2(newPhysicalSize.width as f32, newPhysicalSize.height as f32);
					self.withContextAndRenderState(|this, context, renderState| {
						context.resize(*newPhysicalSize);
						renderState.updateSize(this.context.as_mut().unwrap());
						this.camera.as_mut().unwrap().resize(
							context, &glm::vec2(newSize.x as u32, newSize.y as u32)
						);
						this.cameraInteractor.resize(&newSize);
						this.application.as_mut().unwrap().onResize(
							&glm::vec2(newPhysicalSize.width as f32, newPhysicalSize.height as f32)
						);
					});
				}
				#[cfg(not(target_arch="wasm32"))] {
					self.redrawOnceOnWait = true;
				}
			}

			// Application close
			WindowEvent::CloseRequested => self.exit(eventLoop),

			// Main window redraw
			WindowEvent::RedrawRequested
			=> {
				if self.context.is_some()
				{
					if !self.context().surfaceConfigured {
						tracing::debug!("Surface not yet configured - skipping redraw!");
						return;
					}
					tracing::debug!("Redrawing");

					// Update main surface attachments to draw on them
					match self.context.as_mut().unwrap().newFrame()
					{
						// All fine, we can draw
						Ok(()) => {
							// Update main color attachment for new frame
							self.renderState.as_mut().unwrap().updateMainSurfaceColorAttachment(
								self.context.as_ref().unwrap()
							);

							// Perform actual redrawing
							if let Err(error) = self.redraw() {
								tracing::error!("Error while redrawing: {:?}", error);
							}

							// Swap buffers
							let x = self.context.as_mut().unwrap().endFrame();
							x.present();
						}

						// Reconfigure the surface if it's lost or outdated
						Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated)
						=> {
							let context = self.context();
							context.resize(context.size)
						}

						// The system is out of memory, we should probably quit
						Err(wgpu::SurfaceError::OutOfMemory) => {
							tracing::error!("OutOfMemory");
							eventLoop.exit();
						}

						// This happens when the frame takes too long to present
						Err(wgpu::SurfaceError::Timeout) =>
							{ tracing::warn!("Surface timeout") }
					};

					/////////////////////
					//- TESTING ////////////////////////////////////////////////////////

					self.withContextAndRenderState(|_, context, renderState| {
						let da = renderState.depthStencilAttachment.as_ref().unwrap();
						let mut enc = context.device.create_command_encoder(
							&wgpu::CommandEncoderDescriptor {label: Some("ReadbackTestCommandEncoder")}
						);
						enc.copy_texture_to_buffer(
							*da.texture.readbackView_tex.as_ref().unwrap(),
							*da.texture.readbackView_buf.as_ref().unwrap(), da.texture.descriptor.size
						);
						context.queue.submit(Some(enc.finish()));
						{
							tracing::debug!("Mapping...");
							let buf = da.texture.readbackBuffer.as_ref().unwrap().as_ref();
							buf.slice(0..da.texture.size.actual).map_async(
								wgpu::MapMode::Read, |result| {
									if result.is_ok() {
										let bufView = buf.slice(..).get_mapped_range_mut();
										/*let floats: &mut [f32] = view.iter().as_slice();
										floats.fill(42.0);
										drop(view);
										capturable.unmap();*/
										tracing::debug!("Mapped!!!");
									}
									else {
										tracing::debug!("Mapping Failure!!!");
									}
								}
							);
							tracing::debug!("Polling...");
							context.device.poll(wgpu::Maintain::Wait);
							context.queue.submit([]);
							buf.unmap();
							tracing::debug!("Unmapped!!!");
						}
					});

					//- [END] TESTING //////////////////////////////////////////////////
					/////////////////////
				}
			},

			// User interaction
			  WindowEvent::KeyboardInput{..} | WindowEvent::MouseInput{..} | WindowEvent::CursorMoved{..}
			| WindowEvent::MouseWheel{..} | WindowEvent::ModifiersChanged{..}
			=> {
				let player = util::statify(self);
				if let Some(context) = self.context.as_mut()
				{
					// GUI gets first dibs
					/* nothing here yet */

					// Camera is next
					match self.cameraInteractor.input(&event, player)
					{
						EventOutcome::HandledExclusively(redraw) => {
							if redraw { context.window.request_redraw() }
							return;
						}
						EventOutcome::HandledDontClose(redraw)
						=> if redraw { context.window.request_redraw() }

						EventOutcome::NotHandled => {}
					}

					// Finally, the application
					if let Some(app) = self.application.as_mut()
					{
						match app.onInput(&event)
						{
							EventOutcome::HandledExclusively(redraw) => {
								if redraw { context.window.request_redraw() }
								return;
							}
							EventOutcome::HandledDontClose(redraw)
							=> if redraw { context.window.request_redraw() }

							EventOutcome::NotHandled => {}
						}
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
		if let Some(context) = self.context.as_ref() {
			if self.redrawOnceOnWait {
				self.redrawOnceOnWait = false;
				tracing::debug!("Scheduling additional redraw");
				context.window.request_redraw();
			}
		};
	}
}
