
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
use std::sync::Arc;

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
use crate::view::{Camera, CameraInteractor, CameraParameters};



///////
//
// Constants
//

// For consistent labeling of UI theme-related stuff
const LIGHT_ICON: &str = "☀"; // ToDo: consider 💡
const DARK_ICON: &str = "🌙";
const SYSTEM_ICON: &str = "💻";



///////
//
// Enums and structs
//

/// Struct containing information about a key event. Essentially replicates [`egui::Event::Key`].
#[derive(Debug)]
pub struct KeyInfo<'mods>
{
	/// The key code of the key the event relates to. See [`egui::Event::Key`] for details.
	pub key: egui::Key,

	/// Whether this is a *press* event (`true`) or *release* (`false`). See [`egui::Event::Key`] for details.
	pub pressed: bool,

	/// When [`pressed`] is `true`, indicates whether this is a generated *repeat* event due to the user holding the key
	/// down. See [`egui::Event::Key`] for details.
	pub repeat: bool,

	/// The key modifiers that are currently also pressed. See [`egui::Event::Key`] for details.
	pub modifiers: &'mods egui::Modifiers
}

/// Struct containing information about a click.
#[derive(Debug)]
pub struct ClickInfo<'mods> {
	/// The pointer button the click originated from.
	pub button: egui::PointerButton,

	/// The key modifiers that are currently also pressed. See [`egui::Event::Key`] for details.
	pub modifiers: &'mods egui::Modifiers,

	/// The pointer coordinates within the main viewport at the time of the click, in pixels.
	pub position: glm::UVec2
}

/// Struct containing information about a mouse wheel event
#[derive(Debug)]
pub struct MouseWheelInfo<'mods> {
	/// The amount of scrolling in logical screen points along each axis that the wheel movement(s) are equivalent to.
	pub amount: glm::Vec2,

	/// The key modifiers that are currently also pressed. See [`egui::Event::Key`] for details.
	pub modifiers: &'mods egui::Modifiers
}

/// Struct containing information about a drag event.
#[derive(Debug)]
pub struct DragInfo<'mods> {
	/// Which pointer buttons are down. Should be queried using the [`egui::PointerButton`] enum.
	pub buttons: [bool; 5],

	/// The key modifiers that are currently also pressed. See [`egui::Event::Key`] for details.
	pub modifiers: &'mods egui::Modifiers,

	/// The direction of the drag, using logical screen points as unit.
	pub direction: glm::Vec2
}
impl DragInfo<'_> {
	/// Convenience method for querying the [`buttons`](DragInfo::buttons) field.
	#[inline(always)]
	pub fn button (&self, button: egui::PointerButton) -> bool {
		self.buttons[button as usize]
	}
}

/// Enumeration of input events.
#[derive(Debug)]
pub enum InputEvent<'mods>
{
	/// An event related to keyboard state.
	Key(KeyInfo<'mods>),

	/// A simple click or tap.
	Click(ClickInfo<'mods>),

	/// A double click or tap.
	DoubleClick(ClickInfo<'mods>),

	/// A triple click or tap.
	TripleClick(ClickInfo<'mods>),

	/// A mouse wheel / scroll event.
	MouseWheel(MouseWheelInfo<'mods>),

	/// A pre-processed drag motion (including touch screen swipes).
	Dragged(DragInfo<'mods>)
}

/// Enumeration of possible event handling outcomes.
#[derive(Debug)]
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

/// Collects all bind group layouts available for interfacing with the managed [render passes](GlobalPassInfo) over the
/// scene as set up by the *CGV-rs* [`Player`].
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
	activeSidePanel: usize,

	egui: egui::Context,
	context: Context,
	renderSetup: RenderSetup,
	prevFramebufferResolution: glm::UVec2,

	viewportCompositor: ViewportCompositor,

	camera: Box<dyn Camera>,
	globalPasses: &'static [GlobalPassDeclaration<'static>],

	cameraInteractors: Vec<Box<dyn CameraInteractor>>,
	activeCameraInteractor: usize,

	applicationFactory: Option<Box<dyn ApplicationFactory>>,
	activeApplication: Option<Box<dyn Application>>,
	applications: Vec<Box<dyn Application>>,

	pendingRedraw: bool,
	continousRedrawRequests: u32,
	startInstant: time::Instant,
	prevFrameElapsed: time::Duration,
	prevFrameDuration: time::Duration
}
unsafe impl Send for Player {}
unsafe impl Sync for Player {}

impl Player
{
	pub fn new (applicationFactory: Box<dyn ApplicationFactory>, cc: &eframe::CreationContext) -> Result<Self>
	{
		tracing::info!("Initializing Player...");

		// Get necessary context handles from eframe
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

		// Create context
		let context = Context::new(eguiRs);

		// Log render setup
		let renderSetup = RenderSetup::new(
			&context, eguiRs.target_format, eguiRs.target_format, hal::DepthStencilFormat::D32,
			wgpu::Color{r: 0.3, g: 0.5, b: 0.7, a: 1.}, 1.,
			wgpu::CompareFunction::Less
		);

		// Create stateful rendering components
		let camera = Box::new(view::MonoCamera::new(
			&context, &renderSetup, glm::vec2(2, 2), renderSetup.defaultColorFormat(),
			renderSetup.defaultDepthStencilFormat().into(), Some("MonoCamera0")
		));
		let globalPasses = util::statify(&camera).declareGlobalPasses();
		let viewportCompositor = ViewportCompositor::new(
			&context, &renderSetup, camera.framebuffer().color0(), Some("CGV__MainViewportCompositor")
		);

		// Now construct
		let mut player = Self {
			egui: cc.egui_ctx.clone(),
			activeSidePanel: 0,

			context,
			renderSetup,

			prevFramebufferResolution: glm::vec2(0u32, 0u32),

			camera,
			globalPasses,

			cameraInteractors: vec![Box::new(view::OrbitInteractor::new()), Box::new(view::WASDInteractor::new())],
			activeCameraInteractor: 0,

			viewportCompositor,

			applicationFactory: Some(applicationFactory),
			activeApplication: None,
			applications: Vec::new(),

			pendingRedraw: false,
			continousRedrawRequests: 0,
			startInstant: time::Instant::now(),
			prevFrameElapsed: time::Duration::from_secs(0),
			prevFrameDuration: time::Duration::from_secs(0),
		};
		let player_ref = util::statify(&player);

		// Init application(s)
		let mut activeApplication = player.applicationFactory.take().unwrap().create(
			&player.context, &player.renderSetup
		)?;
		activeApplication.preInit(player.context(), player_ref)?;
		activeApplication.recreatePipelines(
			&player.context, &player.renderSetup,
			Self::extractInfoFromGlobalPassDeclarations(player.globalPasses).as_slice(), player_ref
		);
		activeApplication.postInit(player.context(), player_ref)?;
		player.activeApplication = Some(activeApplication);
		player.activeSidePanel = 2;

		// Done!
		tracing::info!("Startup complete.");
		Ok(player)
	}

	fn extractInfoFromGlobalPassDeclarations<'gpd> (globalPassDeclarations: &'gpd [GlobalPassDeclaration])
		-> Vec<&'gpd GlobalPassInfo<'gpd>>
	{
		let mut passInfos = Vec::with_capacity(globalPassDeclarations.len());
		passInfos.extend(globalPassDeclarations.iter().map(|gpd| &gpd.info));
		passInfos
	}

	#[cfg(not(target_arch="wasm32"))]
	pub fn run<F: ApplicationFactory + 'static> (applicationFactory: F) -> Result<()>
	{
		// Log that we have begun the startup process
		tracing::info!("Starting up...");

		// Run the event loop
		let options = eframe::NativeOptions {
			viewport: egui::ViewportBuilder::default().with_inner_size([1216., 800.]),
			vsync: false,
			multisampling: 0,
			//depth_buffer: 0,
			//stencil_buffer: 0,
			hardware_acceleration: eframe::HardwareAcceleration::Required,
			renderer: eframe::Renderer::Wgpu,
			//run_and_return: false,
			#[allow(unused_variables)] // in Windows builds, we're not using `elBuilder` in the next line
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
				//native_adapter_selector: None,
				//trace_path: None,
				wgpu_setup: egui_wgpu::WgpuSetup::CreateNew( egui_wgpu::WgpuSetupCreateNew {
					instance_descriptor: wgpu::InstanceDescriptor {
						#[cfg(all(not(target_os="windows"),not(target_os="macos")))]
							backends: wgpu::Backends::VULKAN,
						#[cfg(target_os="windows")]
							backends: wgpu::Backends::DX12 | wgpu::Backends::VULKAN,
						#[cfg(target_os="macos")]
							backends: wgpu::Backends::METAL,
						..Default::default()
					},
					power_preference: wgpu::PowerPreference::HighPerformance,
					device_descriptor: Arc::new(|_| wgpu::DeviceDescriptor {
						label: Some("CGV__WgpuDevice"),
						//required_features: Default::default(),
						//required_limits: Default::default(),
						//memory_hints: Default::default(),
						..Default::default()
					}),
					..Default::default()
				}),
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
				wgpu_setup: egui_wgpu::WgpuSetup::CreateNew( egui_wgpu::WgpuSetupCreateNew {
					instance_descriptor: wgpu::InstanceDescriptor {
						backends: wgpu::Backends::BROWSER_WEBGPU,
						..Default::default()
					},
					power_preference: wgpu::PowerPreference::HighPerformance,
					device_descriptor: Arc::new(|_| wgpu::DeviceDescriptor {
						label: Some("CGV__WgpuDevice"),
						//required_features: Default::default(),
						//required_limits: Default::default(),
						//memory_hints: Default::default(),
						..Default::default()
					}),
					..Default::default()
				}),
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

	fn prepareEvents<'is> (
		&self, inputState: &'is egui::InputState, viewportResponse: &egui::Response, highDpiScaleFactor: f32
	) -> Vec<InputEvent<'is>>
	{
		// Pre-allocate event list
		let mut preparedEvents = Vec::with_capacity(4); // <-- heuristically chosen

		// Dragging action
		if viewportResponse.dragged()
		{
			let dm = viewportResponse.drag_motion();
			if dm.length_sq() > 0.
			{
				preparedEvents.push(InputEvent::Dragged(DragInfo {
					buttons: {
						let p = &inputState.pointer; [
							p.primary_down(), p.secondary_down(), p.middle_down(),
							p.button_down(egui::PointerButton::Extra1), p.button_down(egui::PointerButton::Extra2)
						]
					},
					modifiers: &inputState.modifiers,
					direction: glm::vec2(dm.x, dm.y)
				}));
			}
		}

		// Mouse wheel
		if inputState.smooth_scroll_delta.y.abs() > 0. {
			preparedEvents.push(InputEvent::MouseWheel(MouseWheelInfo {
				amount: glm::vec2(inputState.smooth_scroll_delta.x, inputState.smooth_scroll_delta.y),
				modifiers: &inputState.modifiers
			}));
		}

		// Clicks
		let pointerPos = viewportResponse.interact_pointer_pos().map(|pos_egui| {
			let pos_egui = (pos_egui-viewportResponse.rect.min) * highDpiScaleFactor;
			glm::vec2(pos_egui.x as u32, pos_egui.y as u32)
		});
		if let Some(pointerPos) = pointerPos
		{
			for button in [
				egui::PointerButton::Primary, egui::PointerButton::Secondary, egui::PointerButton::Middle,
				egui::PointerButton::Extra1, egui::PointerButton::Extra2
			]{
				let clickInfo = ClickInfo {
					button, modifiers: util::statify(&inputState.modifiers), position: pointerPos
				};
				if viewportResponse.triple_clicked_by(button) {
					preparedEvents.push(InputEvent::TripleClick(clickInfo));
				}
				else if viewportResponse.double_clicked_by(button) {
					preparedEvents.push(InputEvent::DoubleClick(clickInfo));
				}
				else if viewportResponse.clicked_by(button) {
					preparedEvents.push(InputEvent::Click(clickInfo));
				}
			}
		}

		// Report result
		preparedEvents
	}

	fn dispatchTranslatedEvent (&mut self, event: &InputEvent) -> bool
	{
		// Create the 'static reference to self that we will pass to the various callback functions
		let this = util::statify(self);

		// Keep track of whether a full scene redraw is needed
		let mut redraw = false;

		// Applications get first dibs
		// - the active (foreground) application
		match self.activeApplication.as_deref_mut()
			.map(|app| app.input(&event, this)).unwrap_or(EventOutcome::NotHandled)
		{
			// Event was closed by the receiver
			EventOutcome::HandledExclusively(redrawRequested) => return redrawRequested,

			// Event was acted upon but others may react to it too
			EventOutcome::HandledDontClose(redrawRequested) => redraw |= redrawRequested,

			// Event was ignored
			EventOutcome::NotHandled => {}
		}
		// - now any background applications in some undefined order.
		for app in self.applications.as_mut_slice()
		{
			match app.input(&event, this)
			{
				// Event was closed by the receiver
				EventOutcome::HandledExclusively(redrawRequested) => return redrawRequested,

				// Event was acted upon but others may react to it too
				EventOutcome::HandledDontClose(redrawRequested) => redraw |= redrawRequested,

				// Event was ignored
				EventOutcome::NotHandled => {}
			}
		}

		// Finally, the active camera interactor
		match self.cameraInteractors[self.activeCameraInteractor].input(
			&event, util::mutify(self.camera.as_mut()), this
		){
			// Event was handled
			  EventOutcome::HandledExclusively(redrawRequested)
			| EventOutcome::HandledDontClose(redrawRequested) => redraw | redrawRequested,

			// Event was ignored
			EventOutcome::NotHandled => false
		}
	}

	fn dispatchEvents (&mut self, events: &[egui::Event], complexEvents: &[InputEvent]) -> bool
	{
		// Gather key events
		let translatedEvents =  events.iter().filter_map(|event| {
			match event
			{
				egui::Event::Key {
					key, /*physical_key, */pressed, repeat, modifiers, ..
				}
				=> Some(InputEvent::Key(KeyInfo { key: *key, pressed: *pressed, repeat: *repeat, modifiers })),

				_ => None
			}
		});

		// Gather mouse events
		/* t.b.d. */

		// Dispatch events
		let mut redraw = false;
		translatedEvents.for_each(|ref event| redraw |= self.dispatchTranslatedEvent(event));
		complexEvents.iter().for_each(|event| redraw |= self.dispatchTranslatedEvent(event));
		redraw
	}

	/// Performs the logic for preparing applications for rendering the scene.
	fn prepare (
		&self, _: &wgpu::Device, _: &wgpu::Queue, _: &mut wgpu::CommandEncoder
	) -> Vec<wgpu::CommandBuffer>
	{
		// Make all global passes needed by the active camera
		let mut cmdBuffers = Vec::with_capacity(8);
		let cameraName = self.camera.name();
		for passNr in 0..self.globalPasses.len()
		{
			// Get actual pass information
			let pass = &self.globalPasses[passNr];
			let renderState = util::mutify(pass.info.renderState);
			tracing::debug!("Camera[{:?}]: Preparing global pass #{passNr} ({:?})", cameraName, pass.info.pass);

			/* Update managed render state */ {
				// Uniforms
				// - viewing
				let viewingUniforms = renderState.viewingUniforms.borrowData_mut();
				viewingUniforms.projection = * self.camera.projection(pass);
				viewingUniforms.view = * self.camera.view(pass);
				renderState.viewingUniforms.upload(&self.context);
			};

			// Prepare the active application (if any)
			if let Some(application) = util::mutify(&self.activeApplication) {
				if let Some(newCommands) = application.prepareFrame(
					&self.context, pass.info.renderState, &pass.info.pass
				){
					cmdBuffers.extend(newCommands);
				}
			}

			// Prepare the other applications
			self.applications.iter().fold(
				&mut cmdBuffers, |commands, app| {
					if let Some(newCommands) = util::mutify(app).prepareFrame(
						&self.context, pass.info.renderState, &pass.info.pass
					){
						commands.extend(newCommands);
					}
					commands
				}
			);
		}

		// Done!
		cmdBuffers
	}

	/// Performs the logic for letting applications render their contribution to the scene.
	fn redraw (
		&self, _: &wgpu::Device, _: &wgpu::Queue, _: &mut wgpu::CommandEncoder
	) -> Vec<wgpu::CommandBuffer>
	{
		// Make all global passes needed by the active camera
		let mut cmdBuffers = Vec::with_capacity(8);
		let mut cmdEncoder = self.context.device().create_command_encoder(&Default::default());
		let cameraName = self.camera.name();
		for passNr in 0..self.globalPasses.len()
		{
			// Get actual pass information
			let pass = &self.globalPasses[passNr];
			let renderState = util::mutify(pass.info.renderState);

			/* Update managed render state */ {
				// Uniforms
				// - viewing
				let viewingUniforms = renderState.viewingUniforms.borrowData_mut();
				viewingUniforms.projection = *self.camera.projection(pass);
				viewingUniforms.view = *self.camera.view(pass);
				renderState.viewingUniforms.upload(&self.context);
			};

			// Create the managed render pass for this global pass
			let desc = wgpu::RenderPassDescriptor {
				label: Some("CGV__ManagedSceneRenderPass"),
				color_attachments: &[
					renderState.getMainColorAttachment(Some(&pass.info.clearColor))
				],
				depth_stencil_attachment: renderState.getMainDepthStencilAttachment(Some(pass.info.depthClearValue)),
				occlusion_query_set: None,
				timestamp_writes: None,
			};
			let mut renderPass = cmdEncoder.begin_render_pass(&desc);

			// Render the active application (if any)
			if let Some(application) = util::mutify(&self.activeApplication) {
				application.render(&self.context, pass.info.renderState, &mut renderPass, &pass.info.pass);
			}

			// Prepare the other applications
			for app in util::mutify(&self.applications) {
				app.render(&self.context, pass.info.renderState, &mut renderPass, &pass.info.pass);
			}

			// Finish the pass
			tracing::debug!("Camera[{:?}]: Global pass #{passNr} ({:?}) done", cameraName, pass.info.pass);
		}

		// Done!
		cmdBuffers.push(cmdEncoder.finish());
		cmdBuffers
	}

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
		// We have to use volatile writes throughout this function for some reason to make sure the optimizer doesn't
		// interfere with our hacky but (presumably) faster-than-RefCell interior mutability scheme
		// ToDo: remove code smell
		let this = util::mutify(self);
		if this.continousRedrawRequests < 1
		{
			unsafe {
				std::ptr::write_volatile(
					&mut this.prevFrameElapsed as *mut time::Duration, this.startInstant.elapsed()
				);
				std::ptr::write_volatile(
					&mut this.prevFrameDuration as *mut time::Duration, time::Duration::from_secs(0)
				);
			}
			tracing::info!("Starting continuous redrawing");
			this.egui.request_repaint();
		}
		unsafe {
			std::ptr::write_volatile(&mut this.continousRedrawRequests as *mut u32, this.continousRedrawRequests+1)
		}
	}

	pub fn dropContinuousRedrawRequest (&self)
	{
		// We have to use volatile writes throughout this function for some reason to make sure the optimizer doesn't
		// interfere with our hacky but (presumably) faster-than-RefCell interior mutability scheme
		// ToDo: remove code smell
		let this = util::mutify(self); // we use interior mutability
		if this.continousRedrawRequests < 1 {
			panic!("logic error - more continuous redraw requests dropped than were pushed");
		}
		unsafe {
			std::ptr::write_volatile(&mut this.continousRedrawRequests as *mut u32, this.continousRedrawRequests-1)
		}
		if this.continousRedrawRequests < 1
		{
			unsafe {
				std::ptr::write_volatile(
					&mut this.prevFrameDuration as *mut time::Duration, time::Duration::from_secs(0)
				);
			}
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

	pub fn activeCamera (&self) -> &dyn Camera {
		self.camera.as_ref()
	}

	pub fn activeCamera_mut (&self) -> &mut dyn Camera {
		util::mutify(self.camera.as_ref()) // we use interior mutability
	}

	pub fn getDepthAtSurfacePixelAsync<Closure: FnOnce(Option<f32>) + wgpu::WasmNotSend + 'static> (
		&self, pixelCoords: glm::UVec2, callback: Closure
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
		&self, pixelCoords: glm::UVec2, callback: Closure
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
		&self, pixelCoords: glm::UVec2, callback: Closure
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
						let menuIcon = if ui.ctx().style().visuals.dark_mode {DARK_ICON} else {LIGHT_ICON};
						ui.menu_button(menuIcon, |ui| {
							ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
							if ui.add(egui::Button::new(format!("{LIGHT_ICON} Light"))).clicked() {
								ui.ctx().set_theme(egui::ThemePreference::Light);
								ui.close_menu();
							}
							else if ui.add(egui::Button::new(format!("{DARK_ICON} Dark"))).clicked() {
								ui.ctx().set_theme(egui::ThemePreference::Dark);
								ui.close_menu();
							}
							else if ui.add(egui::Button::new(format!("{SYSTEM_ICON} System"))).clicked() {
								ui.ctx().set_theme(egui::ThemePreference::System);
								ui.close_menu();
							}
						});
					}
					ui.separator();

					// App switcher
					// - player settings
					if ui.selectable_label(self.activeSidePanel==0, "Player").clicked() {
						self.activeSidePanel = 0;
					}
					// - view settings
					if ui.selectable_label(self.activeSidePanel==1, "View").clicked() {
						self.activeSidePanel = 1;
					}
					// - applications
					/*for (idx, app) in self.applications.iter().enumerate()
					{
						if ui.selectable_label(self.activeSidePanel==0, "Camera").clicked() {
							self.activeSidePanel = 0;
						}
					}*/
					if ui.selectable_label(
						self.activeSidePanel==2, self.activeApplication.as_ref().unwrap().title()
					).clicked() {
						self.activeSidePanel = 2;
					}
				});
			}));


		////
		// Side panel

		egui::SidePanel::right("CGV__sidePanel")
			.resizable(true)
			.default_width(320.)
			.show(ctx, |ui|
			{
				egui::ScrollArea::both().show(ui, |ui|
				{
					ui.horizontal(|ui|
					{
						ui.vertical(|ui|
						{
							match self.activeSidePanel
							{
								0 => {
									// Player UI
									ui.centered_and_justified(|ui| ui.heading("▶ Player"));
									ui.separator();
									ui.label("<nothing here yet>");
								},

								1 => {
									// Camera UI
									ui.centered_and_justified(|ui| ui.heading("📷 View"));
									ui.separator();

									// Active camera and interactor selection
									gui::layout::ControlTableLayouter::new(ui).layout(
										ui, "CGV__CameraUi",
										|cameraUi|
										{
											// activeCameraInteractor
											cameraUi.add("Interactor", |ui, idealSize|
												egui::ComboBox::from_id_salt("CGV_view_inter")
													.selected_text(
														self.cameraInteractors[self.activeCameraInteractor].title()
													)
													.width(idealSize)
													.show_ui(ui, |ui| {
														for (i, ci) in
															self.cameraInteractors.iter().enumerate()
														{
															ui.selectable_value(
																&mut self.activeCameraInteractor, i, ci.title()
															);
														}
													})
											);

											// activeCamera
											let mut sel: usize = 0; // dummy, remove once camera management is done
											cameraUi.add("Active Camera", |ui, idealSize|
												egui::ComboBox::from_id_salt("CGV_view_act")
													.selected_text(self.camera.name())
													.width(idealSize)
													.show_ui(ui, |ui| {
														ui.selectable_value(
															&mut sel, 0, self.camera.name()
														);
													})
											);
										}
									);

									// Settings from active camera and interactor
									ui.add_space(6.);
									egui::CollapsingHeader::new("Interactor settings")
										.id_salt("CGV_view_inter_s")
										.show(ui, |ui| {
											self.cameraInteractors[self.activeCameraInteractor]
											.ui(self.camera.as_mut(), ui);
										});
									egui::CollapsingHeader::new("Active camera settings")
										.id_salt("CGV_view_act_s")
										.show(ui, |ui| {
											CameraParameters::ui(self.camera.as_mut(), ui);
										});
								},

								2 => {
									// Application UI
									ui.centered_and_justified(|ui| ui.heading(
										self.activeApplication.as_ref().unwrap().title()
									));
									ui.separator();
									let this = util::statify(self);
									self.activeApplication.as_mut().unwrap().ui(ui, this);
								},

								_ => unreachable!("INTERNAL LOGIC ERROR: UI state corrupted!")
							}
						});
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
			// Keep track of reasons to do a scene redraw
			let mut redrawScene = self.continousRedrawRequests > 0;

			// Update framebuffer size
			let availableSpace_egui = ui.available_size();
			let pxlsPerPoint = ctx.pixels_per_point();
			let fbResolution = {
				let pixelsEgui = (availableSpace_egui*pxlsPerPoint).ceil();
				glm::vec2(pixelsEgui.x as u32, pixelsEgui.y as u32)
			};
			if fbResolution != self.prevFramebufferResolution && fbResolution.x > 0 && fbResolution.y > 0 {
				self.camera.resize(&self.context, fbResolution);
				self.viewportCompositor.updateSource(&self.context, self.camera.framebuffer().color0());
				self.prevFramebufferResolution = fbResolution;
				tracing::info!("Main framebuffer resized to {:?}", fbResolution);
				redrawScene = true; // we'll need to redraw the scene in addition to the UI
			}
			let (rect, response) =
				ui.allocate_exact_size(availableSpace_egui, egui::Sense::click_and_drag());

			// Gather input
			let inputState = ui.input(|state| util::statify(state));
			let complexEvents = self.prepareEvents(inputState, &response, pxlsPerPoint);

			// Dispatch all gathered events
			redrawScene |= self.dispatchEvents(&inputState.events, &complexEvents);

			// Update camera interactor
			let this = util::statify(self);
			self.cameraInteractors[self.activeCameraInteractor].update(self.camera.as_mut(), this);
			if self.camera.update() {
				redrawScene = true;
			}

			// Hand off remaining logic to render manager
			self.pendingRedraw |= redrawScene;
			ui.painter().add(egui_wgpu::Callback::new_paint_callback(
				rect, RenderManager {
					/*redrawScene, */player: &this, // ToDo: investigate, see below
					viewportCompositor: util::statify(&self.viewportCompositor)
				}
			));
		});
	}
}

/// Helper object for interfacing the [`Player`] with egui_wgpu's draw callbacks.
struct RenderManager<'player> {
	//redrawScene: bool, ToDo: investigate why we can't use this depending on the initial value of Playr.activeSidePanel
	player: &'player Player,
	viewportCompositor: &'player ViewportCompositor
}
impl egui_wgpu::CallbackTrait for RenderManager<'static>
{
	fn prepare (
		&self, device: &wgpu::Device, queue: &wgpu::Queue, _: &egui_wgpu::ScreenDescriptor,
		eguiEncoder: &mut wgpu::CommandEncoder, _: &mut egui_wgpu::CallbackResources
	) -> Vec<wgpu::CommandBuffer>
	{
		// Only prepare the scene if requested
		if self.player.pendingRedraw /* || self.redrawScene*/ { // ToDo: investigate, see above
			tracing::debug!("Redrawing");
			self.player.prepare(device, queue, eguiEncoder)
		} else {
			Vec::new()
		}
	}

	fn finish_prepare (
		&self, device: &wgpu::Device, queue: &wgpu::Queue, eguiEncoder: &mut wgpu::CommandEncoder,
		_: &mut egui_wgpu::CallbackResources
	) -> Vec<wgpu::CommandBuffer>
	{
		// Only redraw the scene if requested
		if self.player.pendingRedraw {
			let cmdBuffers = self.player.redraw(device, queue, eguiEncoder);
			unsafe { #[allow(invalid_reference_casting)] std::ptr::write_volatile(
				&self.player.pendingRedraw as *const bool as *mut bool, false
			)}
			cmdBuffers
		} else {
			Vec::new()
		}
	}

	fn paint (
		&self, _: epaint::PaintCallbackInfo, eguiRenderPass: &mut wgpu::RenderPass<'static>,
		_: &egui_wgpu::CallbackResources
	){
		// Composit current view of the scene onto egui viewport
		self.viewportCompositor.composit(eguiRenderPass);

		// Update frame stats
		if self.player.continousRedrawRequests > 0 {
			let player = util::mutify(self.player); // we use interior mutability
			let elapsed = player.startInstant.elapsed();
			player.prevFrameDuration = elapsed - player.prevFrameElapsed;
			player.prevFrameElapsed = elapsed;
			player.egui.request_repaint();
		}
	}
}
