
//////
//
// Module definitions
//

/// Submodule providing the [`RenderSetup`].
mod rendersetup;
pub use rendersetup::RenderSetup; // - re-export

/// Submodule providing the [`ViewportCompositor`]
mod viewportcompositor;
use viewportcompositor::*;



//////
//
// Imports
//

// Standard library
use std::{sync::Arc, any::Any};

// Winit library
#[cfg(all(not(target_arch="wasm32"),not(target_os="windows"),not(target_os="macos"),feature="wayland"))]
#[allow(unused_imports)]
use winit::platform::wayland::EventLoopBuilderExtWayland;
#[cfg(all(not(target_arch="wasm32"),not(target_os="windows"),not(target_os="macos"),feature="x11"))]
#[allow(unused_imports)]
use winit::platform::x11::EventLoopBuilderExtX11;

// WGPU API
use wgpu;

// Egui library and framework
use egui;
use eframe::egui_wgpu;
use eframe::epaint;

// Local imports
use crate::*;



///////
//
// Enums and structs
//

/// Struct containing information about a key event. Essentially replicates [`egui::Event::Key`].
#[derive(Debug)]
pub struct KeyInfo
{
	/// The key code of the key the event relates to. See [`egui::Event::Key`] for details.
	pub key: egui::Key,

	/// Whether this is a *press* event (`true`) or *release* (`false`). See [`egui::Event::Key`] for details.
	pub pressed: bool,

	/// When [`pressed`] is `true`, indicates whether this is a generated *repeat* event due to the user holding the key
	/// down. See [`egui::Event::Key`] for details.
	pub repeat: bool,

	/// The key modifiers that are currently also pressed. See [`egui::Event::Key`] for details.
	pub modifiers: egui::Modifiers
}

/// Struct containing information about a click.
#[derive(Debug)]
pub struct ClickInfo {
	/// The pointer button the click originated from.
	button: egui::PointerButton,

	/// The pointer coordinates within the main viewport at the time of the click, in pixels.
	position: glm::UVec2
}

/// Struct containing information about a drag event.
#[derive(Debug)]
pub struct DragInfo {
	/// Which pointer buttons are down. Should be query using the [`egui::PointerButton`] enum.
	pub buttons: [bool; 5],

	/// The direction of the drag, using logical screen points as unit.
	pub direction: glm::Vec2
}

/// Enumeration of input events.
#[derive(Debug)]
pub enum InputEvent
{
	/// An event related to keyboard state. See [`KeyInfo`].
	Key(KeyInfo),

	/// A simple click or tap.
	Click(ClickInfo),

	/// A double click or tap.
	DoubleClick(ClickInfo),

	/// A triple click or tap.
	TripleClick(ClickInfo),

	/// A pre-processed drag motion (including touch screen swipes). See [`DragInfo`].
	Dragged(DragInfo)
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

/// Collects all bind group layouts available for interfacing with the managed [render pipeline](wgpu::RenderPipeline)
/// setup of the *CGV-rs* [`Player`].
pub struct ManagedBindGroupLayouts {
	/// The layout of the bind group for the [viewing](ViewingStruct) uniforms.
	pub viewing: wgpu::BindGroupLayout
}



//////
//
// Classes
//

////
// Player

/// The central application host class.
pub struct Player
{
	/*eventLoop: Option<EventLoop<UserEvent>>,
	eventLoopProxy: EventLoopProxy<UserEvent>,

	#[cfg(target_arch="wasm32")]
	canvas: Option<web_sys::Element>,

	context: Option<Context>,
	redrawOnceOnWait: bool,*/

	themeSet: bool,
	activeSidePanel: u32,

	egui: egui::Context,
	context: Context,
	renderSetup: RenderSetup,
	prevFramebufferDims: glm::UVec2,
	mainFramebuffer: /*Rc<*/hal::Framebuffer/*>*/,
	viewportCompositor: ViewportCompositor,

	applicationFactory: Box<dyn ApplicationFactory>,
	activeApplication: Option<Box<dyn Application>>,
	applications: Vec<Box<dyn Application>>,

	camera: Box<dyn view::Camera>,
	cameraInteractor: Box<dyn view::CameraInteractor>,

	continousRedrawRequests: u32,
	startInstant: time::Instant,
	prevFrameElapsed: time::Duration,
	prevFrameDuration: time::Duration
}

impl Player
{
	pub fn new (applicationFactory: Box<dyn ApplicationFactory>, cc: &eframe::CreationContext) -> Result<Self>
	{
		tracing::info!("Initializing Player...");

		if cc.wgpu_render_state.is_none() {
			return Err(anyhow!("eframe is not configured to use the WGPU backend"));
		}
		let eguiRs = cc.wgpu_render_state.as_ref().unwrap();

		// Adjust GUI font sizes (leave at original for now)
		cc.egui_ctx.all_styles_mut(|style| {
			for (_, fontId) in style.text_styles.iter_mut() {
				fontId.size *= 1.;
			}
		});

		// Launch main event loop. Most initialization is event-driven and will happen in there.
		/*let eventLoop = EventLoop::<UserEvent>::with_user_event().build()?;
		eventLoop.set_control_flow(ControlFlow::Wait);*/

		// Create context
		let context = Context::new(&context::WgpuSetup {
			adapter: &eguiRs.adapter,
			device: &eguiRs.device,
			queue: &eguiRs.queue
		});

		// Log render setup
		let renderSetup = RenderSetup::new(
			&context, eguiRs.target_format, eguiRs.target_format, hal::DepthStencilFormat::D32,
			wgpu::Color{r: 0.3, g: 0.5, b: 0.7, a: 1.}, 1., wgpu::CompareFunction::Less
		);

		let mainFramebuffer = hal::FramebufferBuilder::withDims(&glm::vec2(1, 1))
			.withLabel("CGV__MainFramebuffer")
			.attachColor(wgpu::TextureFormat::Bgra8Unorm, Some(wgpu::TextureUsages::TEXTURE_BINDING))
			.attachDepthStencil(hal::DepthStencilFormat::D32, Some(wgpu::TextureUsages::COPY_SRC))
			.build(&context);

		tracing::info!("Startup complete.");

		// Done, now construct
		Ok(Self {
			/*eventLoopProxy: eventLoop.create_proxy(),
			eventLoop: Some(eventLoop),*/

			egui: cc.egui_ctx.clone(),
			//redrawOnceOnWait: false,

			themeSet: false,
			activeSidePanel: 0,

			prevFramebufferDims: Default::default(),
			viewportCompositor: ViewportCompositor::new(
				&context, &renderSetup, mainFramebuffer.color0(), Some("CGV__MainViewportCompositor")
			),

			applicationFactory,
			activeApplication: None,
			applications: Vec::new(),

			camera: Box::new(view::MonoCamera::new(
				&context, view::RenderTarget::Provided(util::statify(&mainFramebuffer)),
				&renderSetup, Some("CGV__MainCamera")
			)),
			mainFramebuffer,
			context,
			renderSetup,
			cameraInteractor: Box::new(view::OrbitInteractor::new()),
			//clearers: Vec::new(),

			continousRedrawRequests: 0,
			startInstant: time::Instant::now(),
			prevFrameElapsed: time::Duration::from_secs(0),
			prevFrameDuration: time::Duration::from_secs(0),
		})
	}

	#[cfg(not(target_arch="wasm32"))]
	pub fn run<F: ApplicationFactory + 'static> (applicationFactory: F) -> Result<()>
	{
		// Log that we have begun the startup process
		tracing::info!("Starting up...");

		// Set the application factory
		//self.applicationFactory = Some(Box::new(applicationFactory));

		// Run the event loop
		//self.eventLoop.take().unwrap().run_app(&mut self)?;
		let options = eframe::NativeOptions {
			viewport: egui::ViewportBuilder::default().with_inner_size([1152., 720.]),
			vsync: false,
			multisampling: 0,
			//depth_buffer: 0,
			//stencil_buffer: 0,
			hardware_acceleration: eframe::HardwareAcceleration::Required,
			renderer: eframe::Renderer::Wgpu,
			//..Default::default()
			//run_and_return: false,
			event_loop_builder: Some(Box::new(|elBuilder| {
				// Conditional code for the two supported display protocols on *nix. Wayland takes precedence in case
				// both protocols are enabled.
				#[cfg(all(not(target_os="windows"),not(target_os="macos")))] {
					// - Wayland (either just Wayland or both)
					#[cfg(all(feature="wayland"))]
					elBuilder.with_wayland();
					// - just X11
					#[cfg(all(feature="x11",not(feature="wayland")))]
					elBuilder.with_x11();
					// - neither - invalid configuration!
					#[cfg(all(not(feature="wayland"),not(feature="x11")))]
					compile_error!("Must enable one of `x11` or `wayland` for Unix builds!");
				}
			})),
			//centered: false,
			wgpu_options: egui_wgpu::WgpuConfiguration {
				//present_mode: Default::default(),
				//desired_maximum_frame_latency: None,
				wgpu_setup: egui_wgpu::WgpuSetup::CreateNew {
					#[cfg(all(not(target_os="windows"),not(target_os="macos")))]
					supported_backends: wgpu::Backends::VULKAN,
					#[cfg(target_os="windows")]
					supported_backends: wgpu::Backends::DX12 | wgpu::Backends::VULKAN,
					#[cfg(target_os="macos")]
					supported_backends: wgpu::Backends::METAL,
					power_preference: wgpu::PowerPreference::HighPerformance,
					device_descriptor: Arc::new(|_| wgpu::DeviceDescriptor {
						label: Some("CGV__WgpuDevice"),
						//required_features: Default::default(),
						//required_limits: Default::default(),
						//memory_hints: Default::default(),
						..Default::default()
					}),
				},
				..Default::default()
			},
			//persist_window: false,
			//persistence_path: None,
			dithering: true,
			..Default::default()
		};

		// Run and report result
		match eframe::run_native(
			"CGV-rs Player", options, Box::new(
				move |cc| Ok(Box::new(Player::new(Box::new(applicationFactory), cc)?))
			)
		){
			Ok(_) => {
				tracing::info!("Shutdown complete.");
				Ok(())
			},
			Err(error) => Err(anyhow::anyhow!("{:?}", error))
		}
	}

	#[cfg(target_arch="wasm32")]
	pub fn run<F: ApplicationFactory + 'static> (applicationFactory: F) -> Result<()>
	{
		// In case of WASM, make sure the JavaScript console is set up for receiving log messages first thing (for non-
		// WASM targets, tracing/logging is already being set up at module loading time)
		initTracing();

		// Log that we have begun the startup process
		tracing::info!("Starting up...");

		let webOptions = eframe::WebOptions {
			//depth_buffer: 0,
			wgpu_options: egui_wgpu::WgpuConfiguration {
				//present_mode: Default::default(),
				//desired_maximum_frame_latency: None,
				wgpu_setup: egui_wgpu::WgpuSetup::CreateNew {
					supported_backends: wgpu::Backends::BROWSER_WEBGPU,
					power_preference: wgpu::PowerPreference::HighPerformance,
					device_descriptor: Arc::new(|_| wgpu::DeviceDescriptor {
						label: Some("CGV__WgpuDevice"),
						//required_features: Default::default(),
						//required_limits: Default::default(),
						//memory_hints: Default::default(),
						..Default::default()
					}),
				},
				..Default::default()
			},
			dithering: true,
			..Default::default()
		};

		use eframe::wasm_bindgen::JsCast as _;
		use eframe::web_sys;
		wasm_bindgen_futures::spawn_local(async move {
			let document = web_sys::window()
				.expect("No window")
				.document()
				.expect("No document");

			let canvas = document
				.get_element_by_id("cgvRsCanvas")
				.expect("Failed to find target canvas with id=`cgvRsCanvas`!")
				.dyn_into::<web_sys::HtmlCanvasElement>()
				.expect("Element with id=`cgvRsCanvas` was not a HtmlCanvasElement!");

			let startResult = eframe::WebRunner::new()
				.start(
					canvas,
					webOptions,
					Box::new(|cc| Ok(Box::new(Player::new(Box::new(applicationFactory), cc)?)))
				)
				.await;

			// Remove the loading text and spinner:
			match startResult {
				Ok(_) => if let Some(loadingIndicator) =
					document.get_element_by_id("cgvLoadingIndicator") { loadingIndicator.remove(); },
				Err(error) => {
					let msgDetail = if let Some(errorDesc) = error.as_string()
					{ errorDesc }
					else
					{ format!("{:?}", error) };
					if let Some(loadingIndicator) =
						document.get_element_by_id("cgvLoadingIndicator") {
						let msg = format!(
							"<p>The CGV-rs Player has crashed.<br/>Reason: {:?}</p><p>See the developer console for details. </p>", msgDetail
						);
						loadingIndicator.set_inner_html(msg.as_str());
					}
					panic!("FATAL: failed to start CGV-rs Player:\n{msgDetail}");
				}
			};
		});

		// Done (although we won't actually ever reach this code)
		Ok(())
	}

	fn prepareEvents (&self, ui: &egui::Ui, viewportResponse: &egui::Response) -> Vec<InputEvent>
	{
		// Pre-allocate event list
		let mut preparedEvents = Vec::with_capacity(4); // <-- heuristically chosen

		// Dragging action
		if viewportResponse.dragged()
		{
			let dm = viewportResponse.drag_motion();
			if dm.length_sq() > 0. {
				preparedEvents.push(InputEvent::Dragged(DragInfo {
					buttons: ui.input(|state| {
						let p = &state.pointer; [
							p.primary_down(), p.secondary_down(), p.middle_down(),
							p.button_down(egui::PointerButton::Extra1), p.button_down(egui::PointerButton::Extra2)
						]
					}),
					direction: glm::vec2(dm.x, dm.y)
				}));
			}
		}

		// Clicks
		let pointerPos = viewportResponse.interact_pointer_pos().map(|pos_egui| {
			let pos_egui = pos_egui - viewportResponse.rect.min;
			glm::vec2(pos_egui.x as u32, pos_egui.y as u32)
		});
		for button in [
			egui::PointerButton::Primary, egui::PointerButton::Secondary, egui::PointerButton::Middle,
			egui::PointerButton::Extra1, egui::PointerButton::Extra2
		]{
			if viewportResponse.clicked_by(button) {
				preparedEvents.push(InputEvent::Click(ClickInfo {
					button, position: pointerPos.as_ref().unwrap().clone()
				}));
			}
			if viewportResponse.double_clicked_by(button) {
				preparedEvents.push(InputEvent::DoubleClick(ClickInfo {
					button, position: pointerPos.as_ref().unwrap().clone()
				}));
			}
			if viewportResponse.triple_clicked_by(button) {
				preparedEvents.push(InputEvent::TripleClick(ClickInfo {
					button, position: pointerPos.as_ref().unwrap().clone()
				}));
			}
		}

		// Report result
		preparedEvents
	}

	fn dispatchTranslatedEvent (&mut self, event: &InputEvent)
	{
		// - create the 'static reference to self that we will pass to the various callback functions
		let this = util::mutify(self);

		// Applications get first dibs
		// - the active (foreground) application
		if matches!(self.activeApplication.as_deref_mut().map(
			            	|app| app.input(&event, this)
			            ).unwrap_or(EventOutcome::NotHandled),
			   /* == */ EventOutcome::HandledExclusively(_)
			){
			// Event was closed by the receiver!
			return;
		}
		// - now any background applications in some undefined order.
		for app in self.applications.as_mut_slice()
		{
			if matches!(app.input(&event, this), EventOutcome::HandledExclusively(_)) {
				// Event was closed by the receiver!
				return;
			}
		}

		// Finally, the active camera interactor
		self.cameraInteractor.input(&event, this);
	}

	fn dispatchEvents (&mut self, events: &[egui::Event], complexEvents: &[InputEvent])
	{
		// Gather key events
		let mut translatedEvents =  events.iter().filter_map(|event| {
			match event
			{
				egui::Event::Key {key, /*physical_key, */pressed, repeat, modifiers , ..}
				=> {
					Some(InputEvent::Key(KeyInfo {
						key: *key, pressed: *pressed, repeat: *repeat, modifiers: *modifiers
					}))
				},
				_ => None
			}
		});

		// Gather mouse events
		/* t.b.d. */

		// Dispatch eventsthis
		translatedEvents.for_each(|ref event| self.dispatchTranslatedEvent(event));
		complexEvents.iter().for_each(|event| self.dispatchTranslatedEvent(event));
	}

	// Performs the actual redrawing logic
	/*fn redraw (&mut self) -> Result<()>
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

		// Render egui
		self.renderEgui();

		// Done!
		Ok(())
	}*/

	#[inline(always)]
	pub fn context (&self) -> &Context {
		&self.context
	}

	#[inline(always)]
	pub fn egui (&self) -> &egui::Context {
		&self.egui
	}

	pub fn pushContinuousRedrawRequest (&self)
	{
		let this = util::mutify(self);
		if this.continousRedrawRequests < 1 {
			this.prevFrameElapsed = this.startInstant.elapsed();
			this.prevFrameDuration = time::Duration::from_secs(0);
			tracing::info!("Starting continuous redrawing");
			this.egui.request_repaint();
		}
		this.continousRedrawRequests += 1;
	}

	pub fn dropContinuousRedrawRequest (&self)
	{
		let this = util::mutify(self);
		if this.continousRedrawRequests < 1 {
			panic!("logic error - more continuous redraw requests dropped than were pushed");
		}
		this.continousRedrawRequests -= 1;
		if this.continousRedrawRequests < 1 {
			this.prevFrameDuration = time::Duration::from_secs(0);
			tracing::info!("Stopping continuous redrawing");
		}
	}

	pub fn postRedraw (&self) {
		if self.continousRedrawRequests < 1 {
			self.egui.request_repaint();
		}
	}

	pub fn lastFrameTime (&self) -> f32 {
		self.prevFrameDuration.as_secs_f32()
	}

	pub fn exit (&self, eguiContext: &egui::Context) {
		tracing::info!("Exiting...");
		eguiContext.send_viewport_cmd(egui::ViewportCommand::Close);
	}

	pub fn getDepthAtSurfacePixelAsync<Closure: FnOnce(Option<f32>) + wgpu::WasmNotSend + 'static> (
		&self, pixelCoords: &glm::UVec2, callback: Closure
	){
		if let Some(dispatcher) = self.camera.getDepthReadbackDispatcher(pixelCoords) {
			dispatcher.getDepthValue_async(&self.context, |depth| {
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
		if let Some(dispatcher) = self.camera.getDepthReadbackDispatcher(pixelCoords) {
			dispatcher.unprojectPointH_async(&self.context, |point| {
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
		if let Some(dispatcher) = self.camera.getDepthReadbackDispatcher(pixelCoords) {
			dispatcher.unprojectPoint_async(&self.context, |point| {
				callback(point);
			})
		}
		else {
			callback(None)
		}
	}
}

impl eframe::App for Player
{
	fn update (&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
	{
		////
		// Menu bar

		egui::TopBottomPanel::top("menu_bar").show(ctx, |ui|
			egui::ScrollArea::horizontal().show(ui, |ui|
			{
				egui::menu::bar(ui, |ui|
				{
					// The global [ESC] quit shortcut
					let quit_shortcut =
						egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Escape);
					if ui.input_mut(|i| i.consume_shortcut(&quit_shortcut)) {
						self.exit(ui.ctx());
					}

					// Menu bar
					ui.menu_button("File", |ui| {
						ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
						#[cfg(not(target_arch="wasm32"))]
						if ui.add(
							egui::Button::new("Quit").shortcut_text(ui.ctx().format_shortcut(&quit_shortcut))
						).clicked()
						{
							self.exit(ui.ctx());
						}
						#[cfg(target_arch="wasm32")]
						ui.label("<nothing here>");
					});
					ui.separator();

					/* Dark/Light mode toggle */ {
						let mut themePref = ui.ctx().options(|opt| opt.theme_preference);
						if !self.themeSet && themePref == egui::ThemePreference::System {
							if ui.ctx().style().visuals.dark_mode { themePref = egui::ThemePreference::Dark; }
							else { themePref = egui::ThemePreference::Light; }
						}
						if ui.button(match themePref {
							egui::ThemePreference::Dark => "Theme 🌙",
							egui::ThemePreference::Light => "Theme ☀",
							egui::ThemePreference::System => "Theme 💻"
						}).clicked() {
							ui.ctx().set_theme(match themePref {
								egui::ThemePreference::System => egui::ThemePreference::Dark,
								egui::ThemePreference::Dark => egui::ThemePreference::Light,
								egui::ThemePreference::Light => { self.themeSet = true; egui::ThemePreference::System }
							});
						};
					}
					ui.separator();

					// Application focus switcher
					/* nothing here yet */
				});
			}));


		////
		// Side panel

		egui::SidePanel::right("CGV__sidePanel")
			.resizable(true)
			.default_width(256.)
			.show(ctx, |ui|
			{
				egui::ScrollArea::both().show(ui, |ui|
				{
					ui.horizontal(|ui|
					{
						ui.vertical(|ui| { match self.activeSidePanel
						{
							0 => {
								// Player UI
								ui.vertical(|ui| {
									ui.label("<nothing here yet>");
								});
							},
							1 => {
								// Camera UI
								ui.vertical(|ui| {
									ui.label("<nothing here yet>")
								});
							},
							2 => {
								// Application UI
								ui.vertical(|ui| {
									ui.label("<nothing here yet>")
								});
							},
							_ => unreachable!("INTERNAL LOGIC ERROR: UI state corrupted!")
						}});
					});
					ui.allocate_space(ui.available_size());
				});
			});


		////
		// 3D viewport

		// Update viewport frame style
		let mut frame = egui::Frame::central_panel(&ctx.style());
		frame.inner_margin = egui::Margin::ZERO;

		// Draw actual viewport panel
		egui::CentralPanel::default().frame(frame).show(ctx, |ui|
		{
			// Keep track of reasons to force a scene redraw
			let mut forceRedrawScene = false;

			// Update framebuffer size
			let availableSpace_egui = ui.available_size();
			let availableSpace = glm::vec2(availableSpace_egui.x as u32, availableSpace_egui.y as u32);
			if availableSpace != self.prevFramebufferDims && availableSpace.x > 0 && availableSpace.y > 0
			{
				self.mainFramebuffer.resize(&self.context, &availableSpace);
				self.viewportCompositor.updateSource(&self.context, self.mainFramebuffer.color0());
				self.prevFramebufferDims = availableSpace;
				tracing::info!("Main framebuffer resized to {:?}", availableSpace);
				self.camera.resize(&self.context, &availableSpace, self.cameraInteractor.as_ref());
				forceRedrawScene = true; // we'll need to redraw the scene in addition to the UI
			}

			// Gather the complex (composed by egui) events that we want to expose to our own components
			// (we can't do it in the .input() block further down as the egui context is locked there)
			let (rect, response) =
				ui.allocate_exact_size(availableSpace_egui, egui::Sense::click_and_drag());
			let complexEvents = self.prepareEvents(ui, &response);

			// Actually process the events
			if response.hovered() { ui.input(|state| {
				// Remove panel border from the focus area
				// ToDo: validate that we really do need that for consistent interaction with the viewport
				let focused = if state.pointer.has_pointer() {
					if let Some(latestPos) = &state.pointer.latest_pos() {
						   latestPos.x > rect.min.x && latestPos.y > rect.min.y
						&& latestPos.x < rect.max.x && latestPos.y < rect.max.y
					}
					else if let Some(latestInteract) = &state.pointer.interact_pos() {
						   latestInteract.x > rect.min.x && latestInteract.y > rect.min.y
						&& latestInteract.x < rect.max.x && latestInteract.y < rect.max.y
					} else {
						false
					}
				} else {
					false
				};
				if focused {
					self.dispatchEvents(&state.events, &complexEvents);
				}
			})}

			// Hand off remaining logic to render manager
			ui.painter().add(egui_wgpu::Callback::new_paint_callback(
				rect, RenderManager {
					forceRedrawScene, viewportCompositor: util::statify(&self.viewportCompositor),
				}
			));
		});
	}
}

/*impl ApplicationHandler<UserEvent> for Player
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

					// Create egui state
					self.egui = Some(Egui::new(context));

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
		// GUI gets very first dibs
		if let Some(context) = &self.context {
			let exclusive = context.eguiPlatform.captures_event(&event);
			util::mutify(context).eguiPlatform.handle_event(&event);
			if exclusive {
				return;
			}
		}

		match &event
		{
			// Main window resize
			WindowEvent::Resized(newPhysicalSize)
			=> {
				if self.context.is_some() {
					let newSize = glm::vec2(newPhysicalSize.width, newPhysicalSize.height);
					self.withContextMut(|this, context| {
						context.resize(*newPhysicalSize);
						context.eguiScreenDesc.size_in_pixels = [newSize.x, newSize.y];
						context.eguiPlatform.handle_event(&event);
						this.camera.as_mut().unwrap().resize(context, &newSize, this.cameraInteractor.as_ref());
						this.application.as_mut().unwrap().onResize(&newSize);
					});
				}
				#[cfg(not(target_arch="wasm32"))] {
					self.redrawOnceOnWait = true;
				}
			}

			// Main window DPI change
			WindowEvent::ScaleFactorChanged {scale_factor, ..}
			=> if let Some(context) = &mut self.context {
				context.eguiScreenDesc.pixels_per_point = *scale_factor as f32;
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
					let context = self.context.as_mut().unwrap();
					match context.newFrame()
					{
						// All fine, we can draw
						Ok(()) => {
							// Advance egui
							context.eguiPlatform.update_time(self.startInstant.elapsed().as_secs_f64());

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
						let elapsed = self.startInstant.elapsed();
						self.prevFrameDuration = elapsed - self.prevFrameElapsed;
						self.prevFrameElapsed = elapsed;
						self.context.as_ref().unwrap().window().request_redraw();
					}
				}
			},

			// User interaction
			  WindowEvent::KeyboardInput{..} | WindowEvent::MouseInput{..} | WindowEvent::CursorMoved{..}
			| WindowEvent::MouseWheel{..} | WindowEvent::ModifiersChanged{..}
			=> {
				let player = util::statify(self);
				if self.context.is_some()
				{
					// Camera first
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
}*/

struct RenderManager<'vc> {
	viewportCompositor: &'vc ViewportCompositor,
	forceRedrawScene: bool
}
impl<'vc> egui_wgpu::CallbackTrait for RenderManager<'vc>
{
	fn prepare (
		&self, _device: &wgpu::Device, _queue: &wgpu::Queue, _screenDesc: &egui_wgpu::ScreenDescriptor,
		_eguiEncoder: &mut wgpu::CommandEncoder, _callbackResources: &mut egui_wgpu::CallbackResources
	) -> Vec<wgpu::CommandBuffer>
	{
		/* doNothing() */
		Vec::new()
	}

	fn finish_prepare (
		&self, _device: &wgpu::Device, _queue: &wgpu::Queue, _eguiEncoder: &mut wgpu::CommandEncoder,
		_callbackResources: &mut egui_wgpu::CallbackResources
	) -> Vec<wgpu::CommandBuffer>
	{
		/* doNothing() */
		Vec::new()
	}

	fn paint (
		&self, _info: epaint::PaintCallbackInfo, renderPass: &mut wgpu::RenderPass<'static>,
		_callbackResources: &egui_wgpu::CallbackResources
	){
		// Composit rendering result to egui viewport
		self.viewportCompositor.composit(renderPass);
	}
}
