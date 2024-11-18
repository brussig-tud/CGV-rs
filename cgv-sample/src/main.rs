
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
use std::default::Default;

// CGV re-imports
use cgv::{wgpu, glm};

// WGPU API
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
	fn create (&self, context: &cgv::Context, _: &cgv::RenderSetup) -> cgv::Result<Box<dyn cgv::Application>>
	{
		////
		// Prepare buffers

		let vertexBuffer = context.device().create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Example__HermiteNodes"),
				contents: util::slicify(NODES),
				usage: wgpu::BufferUsages::VERTEX,
			}
		);
		let indexBuffer = context.device().create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Example__HermiteIndices"),
				contents: util::slicify(INDICES),
				usage: wgpu::BufferUsages::INDEX,
			}
		);


		////
		// Load resources

		// The example shader
		let shader = context.device().create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Example__ShaderModule"),
			source: wgpu::ShaderSource::Wgsl(util::sourceFile!("/shader/traj/shader.wgsl").into()),
		});

		// The example texture
		let tex = cgv::hal::Texture::fromBlob(
			context, util::sourceBytes!("/res/tex/cgvCube.png"), None, Some("Example__TestTexture")
		)?;
		#[allow(non_upper_case_globals)]
		static texBindGroupLayoutEntries: [wgpu::BindGroupLayoutEntry; 2] = [
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
				ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
				count: None,
			},
		];
		let texBindGroupLayout = context.device().create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				entries: texBindGroupLayoutEntries.as_slice(),
				label: Some("Example__TestBindGroupLayout"),
			}
		);
		let texBindGroup = context.device().create_bind_group(
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
				label: Some("Example__TestBindGroup"),
			}
		);


		////
		// Done!

		// Construct the instance and put it in a box
		Ok(Box::new(SampleApplication {
			shader,
			texBindGroupLayout,
			texBindGroup,
			pipelines: Vec::new(),
			vertexBuffer,
			indexBuffer
		}))
	}
}


////
// SampleApplicaton

#[derive(Debug)]
pub struct SampleApplication {
	shader: wgpu::ShaderModule,
	texBindGroupLayout: wgpu::BindGroupLayout,
	texBindGroup: wgpu::BindGroup,
	pipelines: Vec<wgpu::RenderPipeline>,
	vertexBuffer: wgpu::Buffer,
	indexBuffer: wgpu::Buffer
}
impl SampleApplication
{
	/// Helper function: create the interfacing pipeline for the given render state.
	fn createPipeline (
		&self, context: &cgv::Context, renderState: &cgv::RenderState, renderSetup: &cgv::RenderSetup
	) -> wgpu::RenderPipeline
	{
		let pipelineLayout =
			context.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Example__RenderPipelineLayout"),
				bind_group_layouts: &[&renderSetup.bindGroupLayouts().viewing, &self.texBindGroupLayout],
				push_constant_ranges: &[],
			});
		context.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Example__RenderPipeline"),
			layout: Some(&pipelineLayout),
			vertex: wgpu::VertexState {
				module: &self.shader,
				entry_point: None, // our shader traj/shader.wgsl declares only one @vertex function ("vs_main")
				buffers: &[HermiteNode::layoutDesc()],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			},
			fragment: Some(wgpu::FragmentState {
				module: &self.shader,
				entry_point: None, // our shader traj/shader.wgsl declares only one @vertex function ("fs_main")
				targets: &[Some(renderState.colorTargetState().clone())],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleStrip,
				strip_index_format: Some(wgpu::IndexFormat::Uint32),
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Back),
				..Default::default()
			},
			depth_stencil: Some(renderState.depthStencilState().clone()),
			multisample: wgpu::MultisampleState::default(),
			multiview: None,
			cache: None,
		})
	}
}

impl cgv::Application for SampleApplication
{
	fn title (&self) -> &str {
		"Example App"
	}

	fn recreatePipelines (
		&mut self, context: &cgv::Context, renderSetup: &cgv::RenderSetup, globalPasses: &[&cgv::GlobalPassInfo],
		_: &cgv::Player
	){
		// Make space
		self.pipelines.clear();
		self.pipelines.reserve(globalPasses.len());

		// Recreate pipelines
		for (_, pass) in globalPasses.iter().enumerate() {
			self.pipelines.push(self.createPipeline(context, pass.renderState, renderSetup));
		}
	}

	fn input (&mut self, _: &cgv::InputEvent, _: &cgv::Player) -> cgv::EventOutcome {
		// We're not reacting to any input
		cgv::EventOutcome::NotHandled
	}

	fn resize (&mut self, _: &cgv::Context, _: glm::UVec2, _: &cgv::Player) {
		/* We don't have anything to adapt to a new main framebuffer size */
	}

	fn update (&mut self, _: &cgv::Context, _: &cgv::Player) -> bool {
		// We're not updating anything, so no need to redraw from us
		false
	}

	fn prepareFrame (&mut self, _: &cgv::Context, _: &cgv::RenderState, _: &cgv::GlobalPass)
		-> Option<Vec<wgpu::CommandBuffer>>
	{
		// We don't need any additional preparation.
		None
	}

	fn render (
		&mut self, _: &cgv::Context, renderState: &cgv::RenderState, renderPass: &mut wgpu::RenderPass,
		_: &cgv::GlobalPass
	) -> Option<Vec<wgpu::CommandBuffer>>
	{
		renderPass.set_pipeline(&self.pipelines[0]);
		renderPass.set_bind_group(0, &renderState.viewingUniforms.bindGroup, &[]);
		renderPass.set_bind_group(1, &self.texBindGroup, &[]);
		renderPass.set_vertex_buffer(0, self.vertexBuffer.slice(..));
		renderPass.set_index_buffer(self.indexBuffer.slice(..), wgpu::IndexFormat::Uint32);
		renderPass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);

		// We don't need the Player to submit any custom command buffers for us
		None
	}
}

/// The application entry point.
pub fn main() -> cgv::Result<()> {
	// Immediately hand off control flow, passing in a factory for our SampleApplication
	cgv::Player::run(SampleApplicationFactory{})
}
