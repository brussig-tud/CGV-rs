
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
use cgv::{wgpu, glm, egui, tracing};

// WGPU API
use wgpu::util::DeviceExt;

// CGV Framework
use cgv;
use cgv::{util, shader::compile::prelude::*};



//////
//
// Statics
//

const NODES: &[HermiteNode; 8] = &[
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

const INDICES: &[u32; 10] = &[/*quad 1*/0, 1, 2, 3,  /*degen*/3, 5,  /*quad 2*/5, 4, 7, 6];



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
	fn create (&self, context: &cgv::Context, _: &cgv::RenderSetup, environment: cgv::run::Environment)
		-> cgv::Result<Box<dyn cgv::Application>>
	{
		// Tracing
		tracing::info!("Creating Example application");
		tracing::info!("{:?}", environment);


		////
		// Prepare buffers

		// Vertex buffer
		let vertexBuffer = context.device().create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Example__HermiteNodes"),
				contents: util::slicify(NODES),
				usage: wgpu::BufferUsages::VERTEX,
			}
		);

		// Index buffer
		let indexBuffer = context.device().create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Example__HermiteIndices"),
				contents: util::slicify(INDICES),
				usage: wgpu::BufferUsages::INDEX,
			}
		);


		////
		// Load resources

		// Test Slang runtime compilation
		#[cfg(target_arch="wasm32")] let shaderPackage = {
			let mut slangCtx = cgv::shader::slang::ContextBuilder::default().build()?;
			let env = cgv::obtainShaderCompileEnvironment();
			slangCtx.replaceEnvironment(Some(env))?;
			cgv::shader::Package::/*fromSourceCode(
				cgv::shader::WgpuSourceType::mostSuitable(), &slangCtx, "example.slang",
				util::sourceFile!("/shader/example.slang"), None/* all entry points */
			)*/deserialize(
				util::sourceGeneratedBytes!("/shader/example.spk")
			)?
		};
		#[cfg(not(target_arch="wasm32"))] let shaderPackage = {
			let mut slangCtx = cgv::shader::slang::ContextBuilder::withSearchPaths(
				&environment.shaderPath
			).build()?;
			let env = cgv::obtainShaderCompileEnvironment();
			slangCtx.replaceEnvironment(Some(env))?;
			cgv::shader::Package::fromSourceFile(
				cgv::shader::WgpuSourceType::mostSuitable(), &slangCtx,
				util::pathInsideCrate!("/shader/example.slang"), None/* all entry points */
			)?
		};

		// The example shader
		let shader = /*cgv::shader::Package::deserialize(
			util::sourceGeneratedBytes!("/shader/example.spk")
		)?*/
		shaderPackage.createShaderModuleFromBestInstance(context.device(), None, Some("Example__ShaderModule")).ok_or(
			cgv::anyhow!("Could not create example shader module")
		)?;

		// The example texture
		let tex = cgv::hal::Texture::fromBlob(
			context, util::sourceBytes!("/res/tex/cgvCube.png"), cgv::hal::AlphaUsage::DontCare, None,
			cgv::hal::defaultMipmapping(), Some("Example__TestTexture")
		)?;
		static TEX_BINDGROUP_LAYOUT_ENTRIES: [wgpu::BindGroupLayoutEntry; 2] = [
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
				entries: TEX_BINDGROUP_LAYOUT_ENTRIES.as_slice(),
				label: Some("Example__TestBindGroupLayout"),
			}
		);
		let texBindGroup = context.device().create_bind_group(
			&wgpu::BindGroupDescriptor {
				layout: &texBindGroupLayout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&tex.view()),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(context.refSampler(
							&wgpu::SamplerDescriptor {
								address_mode_u: wgpu::AddressMode::Repeat,
								address_mode_v: wgpu::AddressMode::Repeat,
								address_mode_w: wgpu::AddressMode::Repeat,
								mag_filter: wgpu::FilterMode::Linear,
								min_filter: wgpu::FilterMode::Linear,
								mipmap_filter: wgpu::FilterMode::Linear,
								anisotropy_clamp: 16,
								..Default::default()
							}
						)),
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
			indexBuffer,
			guiState: Default::default()
		}))
	}
}


////
// SampleApplicaton

#[derive(Default,Debug)]
struct GuiState {
	pub dummy_bool: bool,
	pub dummy_float: f32
}

#[derive(Debug)]
struct SampleApplication
{
	// Rendering related
	shader: wgpu::ShaderModule,
	texBindGroupLayout: wgpu::BindGroupLayout,
	texBindGroup: wgpu::BindGroup,
	pipelines: Vec<wgpu::RenderPipeline>,
	vertexBuffer: wgpu::Buffer,
	indexBuffer: wgpu::Buffer,

	// GUI-controllable state
	guiState: GuiState
}
impl SampleApplication
{
	/// Helper function: create the interfacing pipeline for the given render state.
	fn createPipeline (
		&self, context: &cgv::Context, renderState: &cgv::RenderState, renderSetup: &cgv::RenderSetup
	) -> wgpu::RenderPipeline
	{
		// Tracing
		tracing::info!("Creating pipelines");

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
				entry_point: Some("vertexMain"), // Slang (for now) requires explicitly stating entry points
				buffers: &[HermiteNode::layoutDesc()],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			},
			fragment: Some(wgpu::FragmentState {
				module: &self.shader,
				entry_point: Some("fragmentMain"), // Slang (for now) requires explicitly stating entry points
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

	fn preInit (&mut self, _: &cgv::Context, _: &cgv::Player) -> cgv::Result<()> {
		// We don't have any pre-initialization to do
		Ok(())
	}

	fn recreatePipelines (
		&mut self, context: &cgv::Context, renderSetup: &cgv::RenderSetup, globalPasses: &[&cgv::GlobalPassInfo],
		_: &cgv::Player
	){
		// Make space
		self.pipelines.clear();
		self.pipelines.reserve(globalPasses.len());

		// Recreate pipelines
		for pass in globalPasses {
			self.pipelines.push(self.createPipeline(context, pass.renderState, renderSetup));
		}
	}

	fn postInit (&mut self, _: &cgv::Context, player: &cgv::Player) -> cgv::Result<()>
	{
		// Tracing
		tracing::info!("Positioning initial camera");

		// Make sure the camera is where we want it to be (assuming we're the only application that cares about that)
		let cam = player.activeCamera_mut().parameters_mut();
		cam.intrinsics.f = 2.;
		cam.extrinsics.eye = glm::vec3(0., 0., 2.);
		Ok(())
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
	-> Option<Vec<wgpu::CommandBuffer>> {
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
		None // we don't need the Player to submit any custom command buffers for us
	}

	fn ui (&mut self, ui: &mut egui::Ui, _: &'static cgv::Player)
	{
		// Add the standard 2-column layout control grid
		cgv::gui::layout::ControlTableLayouter::new(ui).layout(
			ui, "CgvExample",
			|controlTable|
			{
				controlTable.add("check", |ui, _| ui.add(
					egui::Checkbox::new(&mut self.guiState.dummy_bool, "dummy bool")
				));
				controlTable.add("dummy f32", |ui, _| ui.add(
					egui::Slider::new(&mut self.guiState.dummy_float, 0.1..=100.)
						.logarithmic(true)
						.clamping(egui::SliderClamping::Always)
				));
			}
		);
	}
}



//////
//
// Functions
//

/// The application entry point.
pub fn main() -> cgv::Result<()> {
	// Immediately hand off control flow, passing in a factory for our SampleApplication
	cgv::Player::run(SampleApplicationFactory{})
}
