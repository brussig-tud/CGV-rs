
//////
//
// Imports
//

// Standard library
use std::sync::Arc;

// Anyhow
use anyhow::Result;
use tracing::info;

// Winit
use winit::{window::Window, event::WindowEvent, dpi};

// WGPU
use wgpu;
use wgpu::util::DeviceExt;

// GLM
use glm;

// Local imports
use crate::util;



//////
//
// Statics
//

const NODES: &[HermiteNode] = &[
	HermiteNode {
		pos: glm::Vec4::new(-1., -1., 0., 1.),
		color: glm::Vec4::new(1., 1., 1., 1.),
		texcoord: glm::Vec2::new(0., 1.)
	},
	HermiteNode {
		pos: glm::Vec4::new(1., -1., 0., 1.),
		color: glm::Vec4::new(1., 1., 1., 1.),
		texcoord: glm::Vec2::new(1., 1.)
	},
	HermiteNode {
		pos: glm::Vec4::new(-1., 1., 0., 1.),
		color: glm::Vec4::new(1., 1., 1., 1.),
		texcoord: glm::Vec2::new(0., 0.)
	},
	HermiteNode {
		pos: glm::Vec4::new(1., 1., 0., 1.),
		color: glm::Vec4::new(1., 1., 1., 1.),
		texcoord: glm::Vec2::new(1., 0.)
	},
];

const INDICES: &[u32] = &[/* tri 1 */0, 1, 3,  /* tri 2 */3, 0, 2];



//////
//
// Classes
//

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct HermiteNode
{
	pos: glm::Vec4,
	//tan: glm::Vec4,
	color: glm::Vec4,
	//radius: glm::Vec2,
	texcoord: glm::Vec2
}

impl HermiteNode
{
	const GPU_ATTRIBS: [wgpu::VertexAttribute; 3] =
		wgpu::vertex_attr_array![0=>Float32x4, 1=>Float32x4, 2=>Float32x2];

	fn layoutDesc () -> wgpu::VertexBufferLayout<'static>	{
		wgpu::VertexBufferLayout {
			array_stride: size_of::<Self>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &Self::GPU_ATTRIBS,
		}
	}
}


#[derive(Debug)]
pub struct State
{
	surface: wgpu::Surface<'static>,
	device: wgpu::Device,
	queue: wgpu::Queue,

	config: wgpu::SurfaceConfiguration,
	pub surfaceConfigured: bool,

	pub size: dpi::PhysicalSize<u32>,
	pub window: Arc<Window>,

	pipeline: wgpu::RenderPipeline,
	vertexBuffer: wgpu::Buffer,
	indexBuffer: wgpu::Buffer,

	texBindGroup: wgpu::BindGroup
}

impl State {
	// Creating some of the wgpu types requires async code
	pub async fn new (window: Window) -> State
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
				power_preference: wgpu::PowerPreference::default(),
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

		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Shader"),
			source: wgpu::ShaderSource::Wgsl(util::sourceFile!("/shader/traj/shader.wgsl").into()),
		});

		let vertexBuffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("HermiteNodes"),
				contents: util::slicify(NODES),
				usage: wgpu::BufferUsages::VERTEX,
			}
		);
		let indexBuffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("HermiteIndices"),
				contents: util::slicify(INDICES),
				usage: wgpu::BufferUsages::INDEX,
			}
		);


		////
		// Load resources

		let diffuseBytes = util::sourceBytes!("/res/tex/cgvCube.png");
		let diffuseImage = image::load_from_memory(diffuseBytes).unwrap();
		let diffuseRgba = diffuseImage.to_rgba8();
		let texDims = {
			use image::GenericImageView;
			let dims = diffuseImage.dimensions();
			wgpu::Extent3d {width: dims.0, height: dims.1, depth_or_array_layers: 1}
		};
		let tex = device.create_texture(
			&wgpu::TextureDescriptor {
				// All textures are stored as 3D, we represent our 2D texture
				// by setting depth to 1.
				size: texDims,
				mip_level_count: 1, // We'll talk about this a little later
				sample_count: 1,
				dimension: wgpu::TextureDimension::D2,
				// Most images are stored using sRGB, so we need to reflect that here.
				format: wgpu::TextureFormat::Rgba8Unorm,
				// TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
				// COPY_DST means that we want to copy data to this texture
				usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
				label: Some("TestTexture"),
				// This is the same as with the SurfaceConfig. It
				// specifies what texture formats can be used to
				// create TextureViews for this texture. The base
				// texture format (Rgba8Unorm in this case) is
				// always supported. Note that using a different
				// texture format is not supported on the WebGL2
				// backend.
				view_formats: &[],
			}
		);
		queue.write_texture(
			// Tells wgpu where to copy the pixel data
			wgpu::ImageCopyTexture {
				texture: &tex,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			// The actual pixel data
			&diffuseRgba,
			// The layout of the texture
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4*texDims.width),
				rows_per_image: Some(texDims.height),
			},
			texDims,
		);
		queue.submit([]); // make sure the texture transfer starts immediately

		// We don't need to configure the texture view much, so let's
		// let wgpu define it.
		let texView = tex.create_view(&wgpu::TextureViewDescriptor::default());
		let texSampler = device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Linear,
			..Default::default()
		});

		let bindGroupLayout = device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
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
						// This should match the filterable field of the
						// corresponding Texture entry above.
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None,
					},
				],
				label: Some("texture_bind_group_layout"),
			}
		);
		let texBindGroup = device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				layout: &bindGroupLayout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&texView),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&texSampler),
					}
				],
				label: Some("diffuse_bind_group"),
			}
		);


		////
		// Create pipeline

		let pipelineLayout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[&bindGroupLayout],
				push_constant_ranges: &[],
			});

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&pipelineLayout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: None, // 1. -- our shader traj/shader.wgsl declares only one @vertex function ("vs_main")
				buffers: &[HermiteNode::layoutDesc()], // 2.
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			},
			fragment: Some(wgpu::FragmentState { // 3.
				module: &shader,
				entry_point: None, // 1. -- our shader traj/shader.wgsl declares only one @vertex function ("fs_main")
				targets: &[Some(wgpu::ColorTargetState { // 4.
					format: config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				})],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleStrip, // 1.
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw, // 2.
				cull_mode: Some(wgpu::Face::Back),
				// Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
				polygon_mode: wgpu::PolygonMode::Fill,
				// Requires Features::DEPTH_CLIP_CONTROL
				unclipped_depth: false,
				// Requires Features::CONSERVATIVE_RASTERIZATION
				conservative: false,
			},
			depth_stencil: None, // 1.
			multisample: wgpu::MultisampleState {
				count: 1, // 2.
				mask: !0, // 3.
				alpha_to_coverage_enabled: false, // 4.
			},
			multiview: None, // 5.
			cache: None, // 6.
		});

		Self {
			surface,
			device,
			queue,
			config,
			surfaceConfigured,
			size,
			window,
			pipeline,
			vertexBuffer,
			indexBuffer,
			texBindGroup
		}
	}

	pub fn window (&self) -> &Window {
		&self.window
	}

	pub fn resize (&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		info!("Resizing to {:?}", new_size);
		if new_size.width > 0 && new_size.height > 0 {
			self.size = new_size;
			self.config.width = new_size.width;
			self.config.height = new_size.height;
			self.surface.configure(&self.device, &self.config);
			self.surfaceConfigured = true;
		}
	}

	pub fn input (&mut self, event: &WindowEvent) -> bool {
		false
	}

	pub fn update (&mut self) {}

	pub fn render(&mut self) -> Result<(), wgpu::SurfaceError>
	{
		let output = self.surface.get_current_texture()?;
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
			renderPass.set_bind_group(0, Some(&self.texBindGroup), &[]); // NEW!
			renderPass.set_vertex_buffer(0, self.vertexBuffer.slice(..));
			renderPass.set_index_buffer(self.indexBuffer.slice(..), wgpu::IndexFormat::Uint32);
			renderPass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
		}

		// submit will accept anything that implements IntoIter
		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();

		Ok(())
	}
}
