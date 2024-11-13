
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
use crate::wgpu;

// Egui library and framework
use eframe::egui_wgpu;
use eframe::epaint;

// Local imports
use crate::*;
use crate::context::WgpuSetup;
/*use crate::{context::*, renderstate::*};
use clear::{ClearColor, ClearDepth};
use hal::DepthStencilFormat;
#[allow(unused_imports)] // prevent this warning in WASM. ToDo: investigate
use view::Camera;*/



///////
//
// Enums and structs
//

// The type used for our user-defined event.
/*enum UserEvent {
	ContextReady(Result<Context>)
}*/

/// Enumeration of possible event handling outcomes.
/*pub enum EventOutcome
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
}*/

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

/*pub struct GlobalPassDeclaration<'a>
{
	pub pass: GlobalPass,
	pub renderState: &'a mut RenderState,
	pub completionCallback: Option<Box<dyn FnMut(&'static Context, u32)>>
}*/

/// Collects all bind group layouts available for interfacing with the managed [render pipeline](wgpu::RenderPipeline)
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
/*impl RenderSetup
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
}*/



//////
//
// Classes
//

////
// ViewportCompositor

struct ViewportCompositor {
	name: Option<String>,
	texBindGroupName: Option<String>,
	sampler: wgpu::Sampler,
	texBindGroupLayout: wgpu::BindGroupLayout,
	texBindGroup: wgpu::BindGroup,
	pipeline: wgpu::RenderPipeline
}
impl ViewportCompositor {
	pub fn new (context: &Context, source: &hal::Texture, name: Option<&str>) -> Self
	{
		let name = name.map(String::from);

		let shader = context.device().create_shader_module(wgpu::ShaderModuleDescriptor {
			label: util::concatIfSome(&name, "_shaderModule").as_deref(),
			source: wgpu::ShaderSource::Wgsl(util::sourceFile!("/shader/common/compositing.wgsl").into()),
		});

		let sampler = context.device().create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Nearest,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		let texBindGroupLayout = context.device().create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				label: util::concatIfSome(&name, "_texBindGroupLayout").as_deref(),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
						count: None,
					},
				]
			}
		);
		let texBindGroupName = util::concatIfSome(&name, "_texBindGroup");
		let texBindGroup = context.device().create_bind_group(
			&wgpu::BindGroupDescriptor {
				label: texBindGroupName.as_deref(),
				layout: &texBindGroupLayout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&source.view),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&sampler),
					}
				]
			}
		);

		let pipelineLayout = context.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: util::concatIfSome(&name, "_pipelineLayout").as_deref(),
			bind_group_layouts: &[&texBindGroupLayout],
			push_constant_ranges: &[],
		});

		let pipeline = context.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: util::concatIfSome(&name, "_pipeline").as_deref(),
			layout: Some(&pipelineLayout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: None,
				buffers: &[],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: Some("fs_non_premultiplied"),
				targets: &[Some(wgpu::ColorTargetState {
					format: context.surfaceFormat(),
					blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
					write_mask: wgpu::ColorWrites::ALL,
				})],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleStrip,
				cull_mode: None,
				..Default::default()
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState::default(),
			multiview: None,
			cache: None,
		});

		Self {
			name, texBindGroupName, sampler, texBindGroupLayout, texBindGroup, pipeline
		}
	}

	pub fn updateSource (&mut self, context: &Context, source: &hal::Texture) {
		self.texBindGroup = context.device().create_bind_group(
			&wgpu::BindGroupDescriptor {
				label: util::concatIfSome(&self.name, "_texBindGroup").as_deref(),
				layout: &self.texBindGroupLayout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&source.view),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&self.sampler),
					}
				]
			}
		);
	}

	pub fn composit (&self, renderPass: &mut wgpu::RenderPass) {
		renderPass.set_pipeline(&self.pipeline);
		renderPass.set_bind_group(0, &self.texBindGroup, &[]);
		renderPass.draw(0..3, 0..1);
	}
}


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

	prevFramebufferDims: glm::UVec2,
	context: Context,
	mainFramebuffer: /*Rc<*/hal::Framebuffer<'static>/*>*/,
	viewportCompositor: ViewportCompositor,

	applicationFactory: Box<dyn ApplicationFactory>,
	application: Option<Box<dyn Application>>/*,

	renderSetup: Option<RenderSetup>,

	camera: Option<Box<dyn view::Camera>>,
	cameraInteractor: Box<dyn view::CameraInteractor>,
	clearers: Vec<clear::Clear>,

	continousRedrawRequests: u32,
	startInstant: time::Instant,
	prevFrameElapsed: time::Duration,
	prevFrameDuration: time::Duration,

	egui: Option<Egui<'static>>*/
}
unsafe impl Sync for Player {}
unsafe impl Send for Player {}

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
		let context = Context::new(WgpuSetup {
			adapter: &eguiRs.adapter,
			device: &eguiRs.device,
			queue: &eguiRs.queue,
			surfaceFormat: eguiRs.target_format,
		});

		let mainFramebuffer = hal::FramebufferBuilder::withDims(&glm::vec2(1, 1))
			.withLabel("CGV__MainFramebuffer")
			.attachColor(wgpu::TextureFormat::Bgra8Unorm, Some(wgpu::TextureUsages::TEXTURE_BINDING))
			.attachDepthStencil(hal::DepthStencilFormat::D32, Some(wgpu::TextureUsages::COPY_SRC))
			.build(&context);

		tracing::info!("Startup complete.");

		// Done, now construct
		Ok(Self {
			/*eventLoopProxy: eventLoop.create_proxy(),
			eventLoop: Some(eventLoop),

			#[cfg(target_arch="wasm32")]
			canvas: None,

			context: None,
			redrawOnceOnWait: false,*/

			themeSet: false,
			activeSidePanel: 0,

			prevFramebufferDims: Default::default(),
			viewportCompositor: ViewportCompositor::new(
				&context, mainFramebuffer.color(0), Some("CGV__MainViewportCompositor")
			),
			mainFramebuffer,
			context,

			applicationFactory,
			application: None,

			/*renderSetup: None,

			camera: None,
			cameraInteractor: Box::new(view::OrbitInteractor::new()),
			clearers: Vec::new(),

			continousRedrawRequests: 0,
			startInstant: time::Instant::now(),
			prevFrameElapsed: time::Duration::from_secs(0),
			prevFrameDuration: time::Duration::from_secs(0),

			egui: None,*/
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

	/*fn context (&mut self) -> &mut Context {
		self.context.as_mut().unwrap()
	}*/

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
	}

	fn renderEgui (&mut self)
	{
		////
		// Compose GUI for current frame

		let context = util::mutify(self.context.as_mut().unwrap());
		context.eguiPlatform.begin_frame();
		self.egui.as_mut().unwrap().demoApp.ui(&context.eguiPlatform.context());
		let fullOutput = context.eguiPlatform.end_frame(Some(&context.window));
		let paintJobs = context.eguiPlatform.context().tessellate(
			fullOutput.shapes, context.window.scale_factor() as f32
		);
		let texDelta = &fullOutput.textures_delta;


		////
		// Actually render

		let mut eguiCmdEncoder = context.device.create_command_encoder(
			&wgpu::CommandEncoderDescriptor { label: Some("CGV__EguiPassCommandEncoder") }
		);
		context.eguiRenderer.update_buffers(
			&context.device, &context.queue, &mut eguiCmdEncoder, &paintJobs, &context.eguiScreenDesc
		);
		for tex in &texDelta.set {
			context.eguiRenderer.update_texture(&context.device, &context.queue, tex.0, &tex.1);
		}
		/* Render pass private scope */ {
			let this = util::mutify(self);
			let context = util::statify(context);
			this.egui.as_mut().unwrap().updateSurface(context);
			let mut rp = util::mutify(&eguiCmdEncoder).begin_render_pass(
				&self.egui.as_ref().unwrap().renderPassDescriptor
			);
			context.eguiRenderer.render(&mut rp, &paintJobs, &context.eguiScreenDesc);
		}
		context.queue.submit([eguiCmdEncoder.finish()]);
		for texId in &texDelta.free {
			context.eguiRenderer.free_texture(&texId);
		}

		// Check if repaint needed
		if context.eguiPlatform.context().has_requested_repaint() {
			self.postRedraw();
		}
	}

	pub fn pushContinuousRedrawRequest (&self)
	{
		let this = util::mutify(self);
		if self.continousRedrawRequests < 1 {
			this.prevFrameElapsed = this.startInstant.elapsed();
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
	}*/

	pub fn exit (&self, eguiContext: &egui::Context) {
		tracing::info!("Exiting...");
		eguiContext.send_viewport_cmd(egui::ViewportCommand::Close);
	}

	/*pub fn getDepthAtSurfacePixelAsync<Closure: FnOnce(Option<f32>) + wgpu::WasmNotSend + 'static> (
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
	}*/
}

impl eframe::App for Player {
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
					ui.vertical(|ui| {
						match self.activeSidePanel {
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
				self.viewportCompositor.updateSource(&self.context, self.mainFramebuffer.color(0));
				self.prevFramebufferDims = availableSpace;
				tracing::info!("Main framebuffer resized to {:?}", availableSpace);
				// ToDo: inform camera, applications of resize
				forceRedrawScene = true; // We'll need to redraw the whole scene
			}

			// Dispatch remaining logic to render manager
			let (rect, response) =
				ui.allocate_exact_size(availableSpace_egui, egui::Sense::click_and_drag());
			ui.painter().add(egui_wgpu::Callback::new_paint_callback(
				rect, RenderManager {
					forceRedrawScene, framebuffer: util::statify(&self.mainFramebuffer),
					viewportCompositor: util::statify(&self.viewportCompositor),
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

struct RenderManager<'pass> {
	framebuffer: &'pass hal::Framebuffer<'pass>,
	viewportCompositor: &'pass ViewportCompositor,
	forceRedrawScene: bool
}
unsafe impl<'pass> Sync for RenderManager<'pass> {}
unsafe impl<'pass> Send for RenderManager<'pass> {}

impl<'pass> egui_wgpu::CallbackTrait for RenderManager<'pass>
{
	fn prepare (
		&self, _device: &Device, _queue: &Queue, _screenDesc: &egui_wgpu::ScreenDescriptor,
		_eguiEncoder: &mut CommandEncoder, _callbackResources: &mut egui_wgpu::CallbackResources
	) -> Vec<CommandBuffer>
	{
		/* doNothing() */
		Vec::new()
	}

	fn finish_prepare (
		&self, _device: &Device, _queue: &Queue, _eguiEncoder: &mut CommandEncoder,
		_callbackResources: &mut egui_wgpu::CallbackResources
	) -> Vec<CommandBuffer>
	{
		/* doNothing() */
		Vec::new()
	}

	fn paint(
		&self, _info: epaint::PaintCallbackInfo, renderPass: &mut RenderPass<'static>,
		_callbackResources: &egui_wgpu::CallbackResources
	){
		// Composit rendering result to egui viewport
		self.viewportCompositor.composit(renderPass);
	}
}
