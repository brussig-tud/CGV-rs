
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

// CGV-rs Framework
use cgv::{self, util};



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
// Structs
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


////
// ExampleApplication

/// Factory function for our `ExampleApplication` defined right after. Regular functions with this signature implement
/// the [`cgv::ApplicationFactory`] trait. If more flexibility is required to control how exactly your app is created,
/// you can also write a factory class, i.e. a custom struct that implements `cgv::ApplicationFactory`.
fn createBasicExampleApp (context: &cgv::Context, _: &cgv::RenderSetup, environment: cgv::run::Environment)
	-> cgv::Result<Box<dyn cgv::Application>>
{
	// Tracing
	tracing::info!("Creating \"Basic\" example application");
	tracing::info!("{:?}", environment);


	////
	// Prepare buffers

	// Vertex buffer
	let vertexBuffer = context.device().create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("ExBasic__HermiteNodes"),
			contents: util::slicify(NODES),
			usage: wgpu::BufferUsages::VERTEX
		}
	);

	// Index buffer
	let indexBuffer = context.device().create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("ExBasic__HermiteIndices"),
			contents: util::slicify(INDICES),
			usage: wgpu::BufferUsages::INDEX
		}
	);


	////
	// Load resources

	// The example shader
	// - load the shader package we pre-built while the crate was compiled. We could load it from the filesystem
	//   during runtime using `Package::fromFile`, but for easy portability of the executable we bake it into the
	//   binary image and deserialize it from memory.
	let shaderPackage = cgv::shader::Package::deserialize(
		// Bake – when using `sourceGeneratedBytes`, the path is rooted at our crate's *Cargo* build script output
		// directory. `cgv_build::prepareShaders` in our build script will have mirrored our source folder structure.
		util::sourceGeneratedBytes!("/shader/example.spk")
	)?;
	// - obtain the *WGPU* shader module
	let shader = shaderPackage.createShaderModuleFromBestInstance(
		context.device(), None, Some("ExBasic__ShaderModule")
	).ok_or(
		cgv::anyhow!("Could not create example shader module")
	)?;

	// The example texture
	let tex = cgv::hal::Texture::fromBlob(
		context, util::sourceBytes!("/res/tex/cgvCube.png"), cgv::hal::AlphaUsage::DontCare, None,
		cgv::hal::defaultMipmapping(), Some("ExBasic__TestTexture")
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
			label: Some("ExBasic__TexBindGroupLayout"),
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
							mag_filter: wgpu::FilterMode::Linear,
							min_filter: wgpu::FilterMode::Linear,
							mipmap_filter: wgpu::FilterMode::Linear,
							anisotropy_clamp: 16,
							..Default::default()
						}
					)),
				}
			],
			label: Some("ExBasic__TexBindGroup"),
		}
	);


	////
	// Done!

	// Construct the instance and put it in a box
	Ok(Box::new(ExampleApplication {
		shader, texBindGroupLayout, texBindGroup, vertexBuffer, indexBuffer, guiState: Default::default(),
		pipelines: Vec::new(), // <- delayed, *CGV-rs* has a dedicated cycle for this as typically we don't have all
	}))                        //    required information at this point, like viewport dimensions
}

#[derive(Default,Debug)]
struct GuiState {
	pub dummy_bool: bool,
	pub dummy_float: f32
}

#[derive(Debug)]
struct ExampleApplication
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
impl ExampleApplication
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
				label: Some("ExBasic__RenderPipelineLayout"),
				bind_group_layouts: &[&renderSetup.bindGroupLayouts().viewing, &self.texBindGroupLayout],
				push_constant_ranges: &[],
			});
		context.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("ExBasic__RenderPipeline"),
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
impl cgv::Application for ExampleApplication
{
	fn title (&self) -> &str {
		"Basic Example App"
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
			ui, "Cgv.Ex.Basic",
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
	// Immediately hand off control flow, passing in a factory for our ExampleApplication
	cgv::Player::run(createBasicExampleApp)
}
