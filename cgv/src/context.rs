
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

// WGPU AP
use wgpu;

// Local imports
use crate::*;



//////
//
// Classes
//

#[derive(Debug)]
pub struct Context
{
	pub surface: wgpu::Surface<'static>,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,

	config: wgpu::SurfaceConfiguration,
	pub surfaceConfigured: bool,

	pub size: dpi::PhysicalSize<u32>,
	pub window: Arc<Window>
}

impl Context {
	// Creating some of the wgpu types requires async code
	pub async fn new (window: Window) -> Result<Context>
	{
		let size = window.inner_size();

		// The instance is a handle to our GPU
		// Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			#[cfg(not(target_arch="wasm32"))]
				backends: wgpu::Backends::PRIMARY,
			#[cfg(target_arch="wasm32")]
				backends: wgpu::Backends::BROWSER_WEBGPU,
			..Default::default()
		});

		let window = Arc::new(window);
		let surface = instance.create_surface(window.clone()).unwrap();

		let adapter = instance.request_adapter(
			&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::HighPerformance,
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			},
		).await.unwrap();

		let (device, queue) = adapter.request_device(
			&wgpu::DeviceDescriptor {
				required_features: wgpu::Features::empty(),
				// WebGL doesn't support all of wgpu's features, so if
				// we're building for the web, we'll have to disable some.
				required_limits: if cfg!(target_arch = "wasm32") {
					wgpu::Limits::default()
				} else {
					wgpu::Limits::default()
				},
				label: None,
				memory_hints: Default::default(),
			},
			None, // Trace path
		).await.unwrap();

		let surface_caps = surface.get_capabilities(&adapter);
		// Shader code in this tutorial assumes an sRGB surface texture. Using a different
		// one will result in all the colors coming out darker. If you want to support non
		// sRGB surfaces, you'll need to account for that when drawing to the frame.
		let surface_format = surface_caps.formats.iter()
			.find(|f| !f.is_srgb())
			.copied()
			.unwrap_or(surface_caps.formats[0]);
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: size.width,
			height: size.height,
			present_mode: surface_caps.present_modes[0],
			alpha_mode: surface_caps.alpha_modes[0],
			view_formats: vec![],
			desired_maximum_frame_latency: 1,
		};

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
			device,
			queue,
			config,
			surfaceConfigured,
			size,
			window
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

	pub fn render(&mut self) -> Result<(), wgpu::SurfaceError>
	{
		/*let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder"),
		});
		/* create render pass */ {
			let mut renderPass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.3,
							g: 0.5,
							b: 0.7,
							a: 1.,
						}),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				occlusion_query_set: None,
				timestamp_writes: None,
			});
			renderPass.set_pipeline(&self.pipeline);
			renderPass.set_bind_group(0, Some(&self.viewingUniformsBindGroup), &[]);
			renderPass.set_bind_group(1, Some(&self.texBindGroup), &[]);
			renderPass.set_vertex_buffer(0, self.vertexBuffer.slice(..));
			renderPass.set_index_buffer(self.indexBuffer.slice(..), wgpu::IndexFormat::Uint32);
			renderPass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
		}

		// submit will accept anything that implements IntoIter
		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();*/

		Ok(())
	}
}