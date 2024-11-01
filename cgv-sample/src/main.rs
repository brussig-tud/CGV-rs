
//////
//
// Language config
//

// Eff this convention. Probably the worst aspect of Rust after the lack of a standardized ABI
#![allow(non_snake_case)]



//////
//
// Imports
//

// Standard library
/* Nothing here yet */

// CGV re-imports
use cgv::{wgpu, event, glm, Result};

// WGPU
use wgpu::util::DeviceExt;

// CGV Framework
use cgv;
use cgv::util;



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

	HermiteNode {
		pos: glm::Vec4::new(-1., -1., 0., 1.),
		color: glm::Vec4::new(1., 1., 1., 1.),
		texcoord: glm::Vec2::new(1., 1.)
	},
	HermiteNode {
		pos: glm::Vec4::new(1., -1., 0., 1.),
		color: glm::Vec4::new(1., 1., 1., 1.),
		texcoord: glm::Vec2::new(0., 1.)
	},
	HermiteNode {
		pos: glm::Vec4::new(-1., 1., 0., 1.),
		color: glm::Vec4::new(1., 1., 1., 1.),
		texcoord: glm::Vec2::new(1., 0.)
	},
	HermiteNode {
		pos: glm::Vec4::new(1., 1., 0., 1.),
		color: glm::Vec4::new(1., 1., 1., 1.),
		texcoord: glm::Vec2::new(0., 0.)
	}
];

const INDICES: &[u32] = &[/*quad 1*/0, 1, 2, 3,  /*degen*/3, 5,  /*quad 2*/5, 4, 7, 6];



//////
//
// Data structures
//

////
// HermiteNode

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

	fn layoutDesc () -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: size_of::<Self>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &Self::GPU_ATTRIBS,
		}
	}
}



//////
//
// Classes
//

////
// SampleApplicationFactory

struct SampleApplicationFactory {}

impl cgv::ApplicationFactory for SampleApplicationFactory
{
	fn create (&self, context: &cgv::Context, renderSetup: &cgv::RenderSetup) -> Result<Box<dyn cgv::Application>>
	{
		////
		// Load example shader

		let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Shader"),
			source: wgpu::ShaderSource::Wgsl(util::sourceFile!("/shader/traj/shader.wgsl").into()),
		});

		let vertexBuffer = context.device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("HermiteNodes"),
				contents: util::slicify(NODES),
				usage: wgpu::BufferUsages::VERTEX,
			}
		);
		let indexBuffer = context.device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("HermiteIndices"),
				contents: util::slicify(INDICES),
				usage: wgpu::BufferUsages::INDEX,
			}
		);


		////
		// Load resources

		let tex = cgv::hal::Texture::fromBlob(
			context, util::sourceBytes!("/res/tex/cgvCube.png"), None, Some("TestTexture")
		)?;
		let texBindGroupLayout = context.device.create_bind_group_layout(
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
				label: Some("TestBindGroupLayout"),
			}
		);
		let texBindGroup = context.device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				layout: &texBindGroupLayout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&tex.view),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&tex.sampler),
					}
				],
				label: Some("TestBindGroup"),
			}
		);


		////
		// Create pipeline

		let pipelineLayout =
			context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[&renderSetup.bindGroupLayouts().viewing, &texBindGroupLayout],
				push_constant_ranges: &[],
			});

		let pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&pipelineLayout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: None, // our shader traj/shader.wgsl declares only one @vertex function ("vs_main")
				buffers: &[HermiteNode::layoutDesc()], // 2.
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			},
			fragment: Some(wgpu::FragmentState { // 3.
				module: &shader,
				entry_point: None, // our shader traj/shader.wgsl declares only one @vertex function ("fs_main")
				targets: &[Some(wgpu::ColorTargetState { // 4.
					format: context.config.format,
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
			depth_stencil: Some(wgpu::DepthStencilState {
				format: renderSetup.depthStencilFormat(),
				depth_write_enabled: true,
				depth_compare: wgpu::CompareFunction::LessEqual,
				stencil: Default::default(),
				bias: Default::default(),
			}),
			multisample: wgpu::MultisampleState {
				count: 1, // 2.
				mask: !0, // 3.
				alpha_to_coverage_enabled: false, // 4.
			},
			multiview: None, // 5.
			cache: None, // 6.
		});

		Ok(Box::new(SampleApplication {
			pipeline,
			vertexBuffer,
			indexBuffer,
			texBindGroup
		}))
	}
}


////
// SampleApplicaton

#[derive(Debug)]
pub struct SampleApplication {
	pipeline: wgpu::RenderPipeline,
	vertexBuffer: wgpu::Buffer,
	indexBuffer: wgpu::Buffer,
	texBindGroup: wgpu::BindGroup
}

impl cgv::Application for SampleApplication
{
	fn onInput(&mut self, _: &event::WindowEvent) -> cgv::EventOutcome { cgv::EventOutcome::NotHandled }

	fn onResize(&mut self, _: &glm::UVec2) {}

	fn update(&mut self) {}

	fn render(&mut self, context: &cgv::Context, renderState: &cgv::RenderState, _: &cgv::GlobalPass)
		-> Result<()>
	{
		// Get a command encoder
		let mut encoder = context.device.create_command_encoder(
			&wgpu::CommandEncoderDescriptor{label: Some("SampleCommandEncoder")}
		);

		/* create render pass */ {
			let mut renderPass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("SampleRenderPass"),
				color_attachments: &[renderState.getMainSurfaceColorAttachment()],
				depth_stencil_attachment: renderState.getMainSurfaceDepthStencilAttachment(),
				occlusion_query_set: None,
				timestamp_writes: None,
			});
			renderPass.set_pipeline(&self.pipeline);
			renderPass.set_bind_group(0, Some(&renderState.viewingUniforms.bindGroup), &[]);
			renderPass.set_bind_group(1, Some(&self.texBindGroup), &[]);
			renderPass.set_vertex_buffer(0, self.vertexBuffer.slice(..));
			renderPass.set_index_buffer(self.indexBuffer.slice(..), wgpu::IndexFormat::Uint32);
			renderPass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
		}

		// Submit
		context.queue.submit([encoder.finish()]);

		// Done!
		Ok(())
	}
}

// Application entry point
pub fn main() -> Result<()>
{
	// Create a player
	let player = cgv::Player::new()?;

	// Hand off control flow, passing in a factory for our SampleApplication
	player.run(SampleApplicationFactory{})
}
