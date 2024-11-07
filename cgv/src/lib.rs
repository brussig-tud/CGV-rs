
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

// The module encapsulating all low-level graphics objects.
mod context;
pub use context::Context; // re-export

// A submodule implementing a self-contained clear operation.
mod clear;

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
pub use anyhow::Error as Error;
pub use anyhow::anyhow as anyhow;
pub mod time {
	pub use web_time::{Instant as Instant, Duration as Duration};
}
pub use winit::event;
pub use egui_wgpu::wgpu as wgpu;



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
use clear::{ClearColor, ClearDepth};
use hal::DepthStencilFormat;
#[allow(unused_imports)] // prevent this warning in WASM. ToDo: investigate
use view::Camera;



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
	pub renderState: &'a mut RenderState,
	pub completionCallback: Option<Box<dyn FnMut(&'static Context, u32)>>
}

/// Collects all bind group layouts availabel for interfacing with the managed [render pipeline](wgpu::RenderPipeline)
/// setup of the *CGV-rs* [`Player`].
pub struct ManagedBindGroupLayouts {
	/// The layout of the bind group for the [viewing](ViewingStruct) uniforms.
	pub viewing: wgpu::BindGroupLayout
}

/// Collects all rendering setup provided by the *CGV-rs* [`Player`] for applications to use, in case they want to
/// interface with the managed [render pipeline](wgpu::RenderPipeline) setup.
pub struct RenderSetup
{
	/// The color format used for the render targets of managed [global passes](GlobalPass).
	colorFormat: wgpu::TextureFormat,

	/// The depth/stencil format used for the render targets of managed [global passes](GlobalPass).
	depthStencilFormat: wgpu::TextureFormat,

	/// The clear color that will be used on the main framebuffer in case no [`Application`] requests a specific one.
	defaultClearColor: wgpu::Color,

	/// The bind groups provided for interfacing with centrally managed uniforms.
	bindGroupLayouts: ManagedBindGroupLayouts
}
impl RenderSetup
{
	pub(crate) fn new (context: &Context, colorFormat: wgpu::TextureFormat, depthStencilFormat: DepthStencilFormat)
		-> Self
	{
		Self {
			colorFormat, depthStencilFormat: depthStencilFormat.into(),
			defaultClearColor: wgpu::Color{r: 0.3, g: 0.5, b: 0.7, a: 1.},
			bindGroupLayouts: ManagedBindGroupLayouts {
				viewing: ViewingUniformGroup::createBindGroupLayout(
					context, wgpu::ShaderStages::VERTEX_FRAGMENT, Some("CGV__ViewingBindGroupLayout")
				)
			}
		}
	}

	pub fn colorFormat(&self) -> wgpu::TextureFormat { self.colorFormat }

	pub fn depthStencilFormat(&self) -> wgpu::TextureFormat { self.depthStencilFormat }

	pub fn defaultClearColor(&self) -> &wgpu::Color { &self.defaultClearColor }

	pub fn bindGroupLayouts(&self) -> &ManagedBindGroupLayouts { &self.bindGroupLayouts }
}



///////
//
// Classes
//

////
// ApplicationFactory

pub trait ApplicationFactory {
	fn create(&self, context: &Context, renderSetup: &RenderSetup) -> Result<Box<dyn Application>>;
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
	fn render (&mut self, context: &Context, renderState: &RenderState, globalPass: &GlobalPass) -> anyhow::Result<()>;
}


////
// Player

/// The central application host class.
pub struct Player
{
	eventLoop: Option<EventLoop<UserEvent>>,
	eventLoopProxy: EventLoopProxy<UserEvent>,

	#[cfg(target_arch="wasm32")]
	canvas: Option<web_sys::Element>,

	context: Option<Context>,
	redrawOnceOnWait: bool,

	egui: Option<egui_wgpu::Renderer>,

	applicationFactory: Option<Box<dyn ApplicationFactory>>,
	application: Option<Box<dyn Application>>,

	renderSetup: Option<RenderSetup>,

	camera: Option<Box<dyn view::Camera>>,
	cameraInteractor: Box<dyn view::CameraInteractor>,
	clearers: Vec<clear::Clear>,

	continousRedrawRequests: u32,
	prevFrameInstant: time::Instant,
	prevFrameDuration: time::Duration
}
unsafe impl Sync for Player {}
unsafe impl Send for Player {}

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

			#[cfg(target_arch="wasm32")]
			canvas: None,

			context: None,
			redrawOnceOnWait: false,

			egui: None,

			applicationFactory: None,
			application: None,

			renderSetup: None,

			camera: None,
			cameraInteractor: Box::new(view::OrbitInteractor::new()),
			clearers: Vec::new(),

			continousRedrawRequests: 0,
			prevFrameInstant: time::Instant::now(),
			prevFrameDuration: time::Duration::from_secs(0)
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

		// Update the active camera
		if self.cameraInteractor.update(util::statify(self)) {
			self.camera.as_mut().unwrap().update(self.cameraInteractor.as_ref());
		}

		// Determine the global passes we need to make
		let passes = self.camera.as_ref().unwrap().declareGlobalPasses();
		let cameraName = self.camera.as_ref().unwrap().name();
		for passNr in 0..passes.len()
		{
			// Get actual pass information
			let pass = &passes[passNr];
			let renderState = util::mutify(pass.renderState);

			// Update surface
			util::mutify(pass.renderState).beginGlobalPass(context);

			// Clear surface
			let mut passPrepCommands = context.device.create_command_encoder(
				&wgpu::CommandEncoderDescriptor { label: Some("CGV__PrepareGlobalPassCommandEncoder") }
			);
			self.clearers[passNr].clear(
				&mut passPrepCommands, &renderState.getMainSurfaceColorAttachment(),
				&renderState.getMainSurfaceDepthStencilAttachment()
			);

			/* Update managed render state */ {
				// Uniforms
				// - viewing
				let camera = self.camera.as_ref().unwrap().as_ref();
				renderState.viewingUniforms.data.projection = *camera.projection(pass);
				renderState.viewingUniforms.data.view = *camera.view(pass);
				renderState.viewingUniforms.upload(context);
			};

			// Commit preparation work to GPU
			context.queue.submit([passPrepCommands.finish()]);

			let mut renderResult = Ok(());
			if let Some(application) = self.application.as_mut() {
				renderResult = application.render(&context, pass.renderState, &pass.pass);
			}

			// Finish the pass
			renderState.endGlobalPass();
			match &renderResult
			{
				Ok(()) => {
					tracing::debug!("Camera[{:?}]: Global pass #{passNr} ({:?}) done", cameraName, pass.pass);
					if let Some(callback) = util::mutify(&pass.completionCallback) {
						callback(context, passNr as u32);
					}
				}
				Err(error) => {
					tracing::error!(
						"Camera[{:?}]: Global pass #{passNr} ({:?}) failed!\n\tReason: {:?}",
						cameraName, pass.pass, error
					);
					return renderResult;
				}
			}
		}

		// Done!
		Ok(())
	}

	pub fn pushContinuousRedrawRequest (&self)
	{
		let this = util::mutify(self);
		if self.continousRedrawRequests < 1 {
			this.prevFrameInstant = time::Instant::now();
			this.prevFrameDuration = time::Duration::from_secs(0);
			tracing::info!("Starting continuous redrawing");
			self.context.as_ref().unwrap().window().request_redraw();
		}
		this.continousRedrawRequests += 1;
	}

	pub fn dropContinuousRedrawRequest (&self)
	{
		if self.continousRedrawRequests < 1 {
			panic!("logic error - more continuous redraw requests dropped than were pushed");
		}
		let this = util::mutify(self);
		this.continousRedrawRequests -= 1;
		if self.continousRedrawRequests < 1 {
			this.prevFrameDuration = time::Duration::from_secs(0);
			tracing::info!("Stopping continuous redrawing");
		}
	}

	pub fn postRedraw (&self)
	{
		if self.continousRedrawRequests < 1 {
			self.context.as_ref().unwrap().window.request_redraw();
		}
	}

	pub fn lastFrameTime (&self) -> f32 {
		self.prevFrameDuration.as_secs_f32()
	}

	pub fn withContext<ReturnType, Closure: FnOnce(&'static Player, &'static Context) -> ReturnType> (
		&self, codeBlock: Closure
	) -> ReturnType
	{
		let this = util::statify(self);
		codeBlock(this, this.context.as_ref().unwrap())
	}

	pub fn withContextMut<ReturnType, Closure: FnOnce(&'static mut Player, &'static mut Context) -> ReturnType> (
		&mut self, codeBlock: Closure
	) -> ReturnType
	{
		let this = util::mutify(self);
		codeBlock(util::mutify(self), this.context.as_mut().unwrap())
	}

	pub fn exit (&self, eventLoop: &ActiveEventLoop) {
		tracing::info!("Exiting...");
		eventLoop.exit();
	}

	pub fn getDepthAtSurfacePixelAsync<Closure: FnOnce(Option<f32>) + wgpu::WasmNotSend + 'static> (
		&self, pixelCoords: &glm::UVec2, callback: Closure
	){
		if let Some(dispatcher) =
			self.camera.as_ref().unwrap().getDepthReadbackDispatcher(pixelCoords) {
				dispatcher.getDepthValue_async(self.context.as_ref().unwrap(), |depth| {
					callback(Some(depth));
				})
			}
		else {
			callback(None)
		}
	}

	pub fn unprojectPointAtSurfacePixelH_async<Closure: FnOnce(Option<&glm::Vec4>) + wgpu::WasmNotSend + 'static> (
		&self, pixelCoords: &glm::UVec2, callback: Closure
	){
		if let Some(dispatcher) =
			self.camera.as_ref().unwrap().getDepthReadbackDispatcher(pixelCoords) {
			dispatcher.unprojectPointH_async(self.context.as_ref().unwrap(), |point| {
				callback(point);
			})
		}
		else {
			callback(None)
		}
	}

	pub fn unprojectPointAtSurfacePixel_async<Closure: FnOnce(Option<&glm::Vec3>) + wgpu::WasmNotSend + 'static> (
		&self, pixelCoords: &glm::UVec2, callback: Closure
	){
		if let Some(dispatcher) =
			self.camera.as_ref().unwrap().getDepthReadbackDispatcher(pixelCoords) {
			dispatcher.unprojectPoint_async(self.context.as_ref().unwrap(), |point| {
				callback(point);
			})
		}
		else {
			callback(None)
		}
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

		#[cfg(target_arch="wasm32")] {
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

		#[cfg(target_arch="wasm32")] {
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
		#[cfg(not(target_arch="wasm32"))] {
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

					// On non-WASM on the other hand, the surface is correctly configured for the initial size. However,
					// we do need to schedule a single redraw to not get garbage on the screen as the surface is
					// displayed for the first time for some reason...
					#[cfg(not(target_arch="wasm32"))] {
						self.redrawOnceOnWait = true;
					}

					// Create render setup
					self.renderSetup = Some(RenderSetup::new(context, context.config.format, DepthStencilFormat::D32));

					// Attach egui
					let eguiConfig = egui_wgpu::WgpuConfiguration {
						present_mode: context.config.present_mode,
						desired_maximum_frame_latency: Some(context.config.desired_maximum_frame_latency),
						wgpu_setup: egui_wgpu::WgpuSetup::Existing {
							instance: context.instance.clone(), adapter: context.adapter.clone(),
							device: context.device.clone(), queue: context.queue.clone(),
						},
						..Default::default()
					};
					self.egui = Some(egui_wgpu::Renderer::new(
						&context.device, context.config.format,
						Some(self.renderSetup.as_ref().unwrap().depthStencilFormat), 1, false
					));

					// Initialize camera
					// - create default camera
					self.camera = Some({
						#[allow(unused_mut)] // prevent the warning in WASM builds (we need mutability in non-WASM)
						let mut camera = view::MonoCamera::new(
							context, None, self.renderSetup.as_ref().unwrap(), Some("MainCamera")
						);

						// On non-WASM, we don't get an initial resize so we have to initialize the camera manually.
						#[cfg(not(target_arch="wasm32"))] {
							camera.resize(
								context, &glm::vec2(context.size.width, context.size.height),
								self.cameraInteractor.as_ref()
							);
						}
						camera
					});
					/* - initialize global pass resources */ {
						let passes =
							util::statify(self.camera.as_ref().unwrap()).declareGlobalPasses();
						for pass in passes {
							let depthClearing =
								if let Some(dsa) = &pass.renderState.depthStencilAttachment {
									Some(ClearDepth { value: 1., attachment: dsa })
								}
								else { None };
							self.clearers.push(clear::Clear::new(
								context,
								Some(&ClearColor {
									value: self.renderSetup.as_ref().unwrap().defaultClearColor,
									attachment: &pass.renderState.colorAttachment
								}),
								depthClearing.as_ref()
							));
						}
					}

					// Create the application
					let appCreationResult = self.applicationFactory.take().unwrap().create(
						context, self.renderSetup.as_ref().unwrap()
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
					let newSize = glm::vec2(newPhysicalSize.width, newPhysicalSize.height);
					self.withContextMut(|this, context| {
						context.resize(*newPhysicalSize);
						this.camera.as_mut().unwrap().resize(context, &newSize, this.cameraInteractor.as_ref());
						this.application.as_mut().unwrap().onResize(&newSize);
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
							// Perform actual redrawing inside Player implementation
							if let Err(error) = self.redraw() {
								tracing::error!("Error while redrawing: {:?}", error);
							}

							// Swap buffers
							self.context.as_mut().unwrap().endFrame().present();
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

					// Update frame stats
					if self.continousRedrawRequests > 0 {
						let now = time::Instant::now();
						self.prevFrameDuration = now - self.prevFrameInstant;
						self.prevFrameInstant = now;
						self.context.as_ref().unwrap().window().request_redraw();
					}
				}
			},

			// User interaction
			  WindowEvent::KeyboardInput{..} | WindowEvent::MouseInput{..} | WindowEvent::CursorMoved{..}
			| WindowEvent::MouseWheel{..} | WindowEvent::ModifiersChanged{..}
			=> {
				let player = util::statify(self);
				if let Some(_) = self.context.as_mut()
				{
					// GUI gets first dibs
					/* nothing here yet */

					// Camera is next
					match self.cameraInteractor.input(&event, player)
					{
						EventOutcome::HandledExclusively(redraw) => {
							if redraw {
								self.postRedraw();
							}
							return;
						}
						EventOutcome::HandledDontClose(redraw)
						=> if redraw {
							self.postRedraw()
						}

						EventOutcome::NotHandled => {}
					}

					// Finally, the application
					if let Some(app) = self.application.as_mut()
					{
						match app.onInput(&event)
						{
							EventOutcome::HandledExclusively(redraw) => {
								if redraw { self.postRedraw() }
								return;
							}
							EventOutcome::HandledDontClose(redraw)
							=> if redraw { self.postRedraw() }

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
		if let Some(_) = self.context.as_ref() {
			if self.redrawOnceOnWait {
				self.redrawOnceOnWait = false;
				tracing::debug!("Scheduling additional redraw");
				self.postRedraw();
			}
		};
	}
}
