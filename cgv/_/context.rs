
//////
//
// Imports
//

// Standard library
use std::sync::Arc;

// Anyhow library
use anyhow::Result;

// Winit library
use winit::{window::Window, dpi};

// Egui library
use ::egui;
use egui_wgpu;

// Local imports
use crate::*;



//////
//
// Classes
//

pub struct Context
{
	pub surface: wgpu::Surface<'static>,
	pub instance: Arc<wgpu::Instance>,
	pub adapter: Arc<wgpu::Adapter>,
	pub device: Arc<wgpu::Device>,
	pub queue: Arc<wgpu::Queue>,

	pub config: wgpu::SurfaceConfiguration,
	pub surfaceConfigured: bool,

	pub(crate) surfaceTexture: Option<wgpu::SurfaceTexture>,
	pub(crate) surfaceView: Option<wgpu::TextureView>,

	pub size: dpi::PhysicalSize<u32>,
	pub window: Arc<Window>,

	pub eguiScreenDesc: egui_wgpu::ScreenDescriptor,
	pub eguiPlatform: egui_integration::Platform,
	pub eguiRenderer: egui_wgpu::Renderer
}

impl Context
{
	// Creating some of the wgpu types requires async code
	pub async fn new (window: Window) -> Result<Context>
	{
		let size = window.inner_size();

		// The instance is a handle to our GPU
		// Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
		let instance = Arc::new(wgpu::Instance::new(wgpu::InstanceDescriptor {
			#[cfg(not(target_arch="wasm32"))]
				backends: wgpu::Backends::PRIMARY,
			#[cfg(target_arch="wasm32")]
				backends: wgpu::Backends::BROWSER_WEBGPU,
			..Default::default()
		}));

		let window = Arc::new(window);
		let surface = instance.create_surface(window.clone()).unwrap();

		let adapter = Arc::new(instance.request_adapter(
			&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			},
		).await.unwrap());

		let (device, queue) = {
			let (device, queue) = match adapter.request_device(
				&wgpu::DeviceDescriptor {
					required_features: wgpu::Features::empty(),
					required_limits: if cfg!(target_arch="wasm32") {
						wgpu::Limits::default()
					} else {
						wgpu::Limits::default()
					},
					label: None,
					memory_hints: Default::default(),
				},
				None, // trace path
			).await {
				Ok((device, queue)) => (device, queue),
				Err(error) => {
					tracing::error!("Could not create WGPU device: {:?}", error);
					panic!("graphics context creation failure");
				}
			};
			(Arc::new(device), Arc::new(queue))
		};

		let surfaceCaps = surface.get_capabilities(&adapter);
		let surface_format = surfaceCaps.formats.iter()
			.find(|f| !f.is_srgb())
			.copied()
			.unwrap_or(surfaceCaps.formats[0]);
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: size.width,
			height: size.height,
			present_mode: surfaceCaps.present_modes[0],
			alpha_mode: surfaceCaps.alpha_modes[0],
			view_formats: vec![],
			desired_maximum_frame_latency: 1,
		};

		// Attach egui
		let eguiScreenDesc = egui_wgpu::ScreenDescriptor {
			size_in_pixels: [size.width, size.height], pixels_per_point: window.scale_factor() as f32
		};
		let eguiPlatform = egui_integration::Platform::new(
			egui_integration::PlatformDescriptor {
				physical_width: size.width as u32,
				physical_height: size.height as u32,
				scale_factor: window.scale_factor(),
				font_definitions: egui::FontDefinitions::default(),
				style: Default::default(),
			}
		);
		/*let egui = eguiPlatform.context();
		let eguiConfig = egui_wgpu::WgpuConfiguration {
			present_mode: config.present_mode,
			desired_maximum_frame_latency: Some(config.desired_maximum_frame_latency),
			wgpu_setup: egui_wgpu::WgpuSetup::Existing {
				instance: instance.clone(), adapter: adapter.clone(),
				device: device.clone(), queue: queue.clone(),
			},
			..Default::default()
		};
		let mut eguiPainter = egui_wgpu::winit::Painter::new(
			eguiConfig, 1, Some(hal::DepthStencilFormat::D32.into()),
			true, false
		);
		eguiPainter.set_window(egui::ViewportId::ROOT, Some(window.clone())).await?;*/

		let eguiRenderer = egui_wgpu::Renderer::new(
			&device, config.format, /* depth/stencil */None, 1, false
		);

		let surfaceConfigured;
		#[cfg(not(target_arch = "wasm32"))] {
			surface.configure(&device, &config);
			surfaceConfigured = true;
		}
		#[cfg(target_arch = "wasm32")] {
			surfaceConfigured = false;
		}

		// Done!
		Ok(Self {
			surface,
			instance,
			adapter,
			device,
			queue,
			config,
			surfaceConfigured,
			surfaceTexture: None,
			surfaceView: None,
			size,
			window,
			eguiScreenDesc,
			eguiPlatform,
			eguiRenderer
		})
	}

	pub fn window (&self) -> &Window {
		&self.window
	}

	pub fn resize (&mut self, newSize: winit::dpi::PhysicalSize<u32>)
	{
		tracing::info!("Resizing to {:?}", newSize);
		if newSize.width > 0 && newSize.height > 0
		{
			self.size = newSize;
			self.config.width = newSize.width;
			self.config.height = newSize.height;
			self.surface.configure(&self.device, &self.config);
			self.surfaceConfigured = true;
		}
	}
}


////
// ContextPrivateInterface

pub(crate) trait ContextPrivateInterface {
	fn newFrame (&mut self) -> Result<(), wgpu::SurfaceError>;

	fn endFrame (&mut self) -> wgpu::SurfaceTexture;
}

impl ContextPrivateInterface for Context
{
	fn newFrame (&mut self) -> Result<(), wgpu::SurfaceError>
	{
		// Obtain the current surface texture
		self.surfaceTexture = Some(self.surface.get_current_texture()?);
		self.surfaceView = Some(
			self.surfaceTexture.as_ref().unwrap().texture.create_view(&wgpu::TextureViewDescriptor::default())
		);

		// Done!
		Ok(())
	}

	fn endFrame (&mut self) -> wgpu::SurfaceTexture {
		self.surfaceView.take();
		self.surfaceTexture.take().unwrap()
	}
}
