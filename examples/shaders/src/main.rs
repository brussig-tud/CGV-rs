
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
use cgv::{self, util, shader::compile::prelude::*};



//////
//
// Statics
//

const QUAD_VERTS: &[QuadVertex; 8] = &[
	QuadVertex {
		pos: glm::Vec4::new(-1., -1., 0., 1.),
		texcoord: glm::Vec2::new(-1., 1.)
	},
	QuadVertex {
		pos: glm::Vec4::new(1., -1., 0., 1.),
		texcoord: glm::Vec2::new(1., 1.)
	},
	QuadVertex {
		pos: glm::Vec4::new(-1., 1., 0., 1.),
		texcoord: glm::Vec2::new(-1., -1.)
	},
	QuadVertex {
		pos: glm::Vec4::new(1., 1., 0., 1.),
		texcoord: glm::Vec2::new(1., -1.)
	},

	QuadVertex {
		pos: glm::Vec4::new(-1., -1., 0., 1.),
		texcoord: glm::Vec2::new(1., 1.)
	},
	QuadVertex {
		pos: glm::Vec4::new(1., -1., 0., 1.),
		texcoord: glm::Vec2::new(-1., 1.)
	},
	QuadVertex {
		pos: glm::Vec4::new(-1., 1., 0., 1.),
		texcoord: glm::Vec2::new(1., -1.)
	},
	QuadVertex {
		pos: glm::Vec4::new(1., 1., 0., 1.),
		texcoord: glm::Vec2::new(-1., -1.)
	}
];

const INDICES: &[u32; 10] = &[/*quad 1*/0, 1, 2, 3,  /*degen*/3, 5,  /*quad 2*/5, 4, 7, 6];



//////
//
// Structs
//

////
// QuadVertex

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct QuadVertex {
	pos: glm::Vec4,
	texcoord: glm::Vec2
}
impl QuadVertex
{
	const GPU_ATTRIBS: [wgpu::VertexAttribute; 2] =
		wgpu::vertex_attr_array![0=>Float32x4, 1=>Float32x2];

	fn layoutDesc () -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: size_of::<Self>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &Self::GPU_ATTRIBS,
		}
	}
}


////
// OnlineShadersDemo

/// Factory function.
fn createOnlineShadersDemo (context: &cgv::Context, _: &cgv::RenderSetup, environment: cgv::run::Environment)
	-> cgv::Result<Box<dyn cgv::Application>>
{
	// Tracing
	tracing::info!("Creating \"Shaders\" example application");
	tracing::info!("{:?}", environment);


	////
	// Prepare buffers

	// Vertex buffer
	let vertexBuffer = context.device().create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("ExShaders__HermiteNodes"), contents: util::slicify(QUAD_VERTS),
			usage: wgpu::BufferUsages::VERTEX
		}
	);

	// Index buffer
	let indexBuffer = context.device().create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("ExShaders__HermiteIndices"), contents: util::slicify(INDICES), usage: wgpu::BufferUsages::INDEX
		}
	);


	////
	// Load resources

	// Our SDF glyph shader
	// - step 1: a Slang compilation context
	tracing::info!("Preparing shader: context creation");
	#[cfg(not(target_arch="wasm32"))] let mut slangCtx = {
		// On native, it's a good idea to always consider the shader path we get from the runtime environment
		cgv::shader::slang::ContextBuilder::withSearchPaths(&environment.shaderPath).build()?
	};
	#[cfg(target_arch="wasm32")] let mut slangCtx = {
		// On WASM, we can't (yet) use a shader path to find modules residing on a filesystem
		cgv::shader::slang::ContextBuilder::default().build()?
	};
	// - step 2: load the *CGV-rs* core shader library into the context
	tracing::info!("Preparing shader: loading compilation environment");
	let env = cgv::obtainShaderCompileEnvironment();
	slangCtx.replaceEnvironment(Some(env))?;
	// - step 3: build shader package we can use to create *WGPU* shader modules that can be plugged into a
	//           pipeline. In cases where offline compilation is ok, ready-made shader packages can contain several
	//           variants (e.g. SPIR-V for desktop, WGSL for WASM) and be deserialized from a file or memory blob
	tracing::info!("Preparing shader: compiling main module '/shader/sdf_demo.slang'");
	#[cfg(not(target_arch="wasm32"))] let mainModule = {
		// On native, we can load the shader source from the filesystem
		slangCtx.compile(util::pathInsideCrate!("/shader/sdf_demo.slang"))?
	};
	#[cfg(target_arch="wasm32")] let mainModule = {
		// On WASM, we currently have to resort to baking shader source files into the crate. For the same reason (the
		// context does not have runtime filesystem access), we also need to load our local shader "library" into the
		// context manually. This could be largely automated by creating a local *compile environment* and merging it
		// with the one we get from *CGV-rs*, but for our 3 source files we can just do it here.
		use cgv::shader::slang::EnvironmentStorage;
		slangCtx.loadModuleFromSource(
			EnvironmentStorage::SourceCode, "lib/common.slang",
			util::sourceFile!("/shader/lib/common.slang")
		)?;
		slangCtx.loadModuleFromSource(
			EnvironmentStorage::SourceCode, "lib/sdf.slang",
			util::sourceFile!("/shader/lib/sdf.slang")
		)?;
		slangCtx.loadModuleFromSource(
			EnvironmentStorage::SourceCode, "lib/glyph.slang",
			util::sourceFile!("/shader/lib/glyph.slang")
		)?;
		slangCtx.compileFromNamedSource(
			"sdf_demo.slang", util::sourceFile!("/shader/sdf_demo.slang")
		)?
	};
	// - step 4: load a module that provides the `instantiateGlyph` function that our "sdf_demo.slang" module expects
	tracing::info!("Preparing shader: generating specialization module 'glyphProvider'");
	let instantiateCircleModule = slangCtx.compileFromNamedSource(
		"glyph_provider",
		"import \"lib/glyph.slang\"; export struct Glyph: ex::IGlyph = ex::glyphs::Circle;"
	)?;
	// - step 5: link into usable program
	tracing::info!("Preparing shader: linking final program");
	let linked = slangCtx.linkComposite(&slangCtx.createComposite(&[
		cgv::shader::compile::ComponentRef::Module(&mainModule),
		cgv::shader::compile::ComponentRef::Module(&instantiateCircleModule),
	])?)?;
	let shaderPackage = cgv::shader::Package::fromProgram(
		cgv::shader::Program::fromLinkedComposite(
			&slangCtx, cgv::shader::compile::mostSuitableTarget(), &linked
		)?, Some("sdfquad".into()), /* all entry points */None
	)?;
	// - final: obtain the *WGPU* shader module
	tracing::info!("Preparing shader: converting to WGPU shader");
	let shader = shaderPackage.createShaderModuleFromBestInstance(
		context.device(), None, Some("ExShaders__ShaderModule")
	).ok_or(
		cgv::anyhow!("Could not create example shader module")
	)?;
	drop(linked); // currently needed because of Slang-rs issue #26: https://github.com/FloatyMonkey/slang-rs/issues/26


	////
	// Done!

	// Construct the instance and put it in a box
	Ok(Box::new(OnlineShadersDemo {
		shader, pipelines: Vec::new(), vertexBuffer, indexBuffer, guiState: Default::default()
	}))
}

#[derive(Default,Debug)]
struct GuiState {
	pub dummy_bool: bool,
	pub dummy_float: f32
}

#[derive(Debug)]
struct OnlineShadersDemo
{
	// Rendering related
	shader: wgpu::ShaderModule,
	pipelines: Vec<wgpu::RenderPipeline>,
	vertexBuffer: wgpu::Buffer,
	indexBuffer: wgpu::Buffer,

	// GUI-controllable state
	guiState: GuiState
}
impl OnlineShadersDemo
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
				label: Some("ExShaders__RenderPipelineLayout"),
				bind_group_layouts: &[&renderSetup.bindGroupLayouts().viewing],
				push_constant_ranges: &[],
			});
		context.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("ExShaders__RenderPipeline"),
			layout: Some(&pipelineLayout),
			vertex: wgpu::VertexState {
				module: &self.shader,
				entry_point: Some("vertexMain"), // Slang (for now) requires explicitly stating entry points
				buffers: &[QuadVertex::layoutDesc()],
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
impl cgv::Application for OnlineShadersDemo
{
	fn title (&self) -> &str {
		"Online Shader Compilation"
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
		renderPass.set_vertex_buffer(0, self.vertexBuffer.slice(..));
		renderPass.set_index_buffer(self.indexBuffer.slice(..), wgpu::IndexFormat::Uint32);
		renderPass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
		None // we don't need the Player to submit any custom command buffers for us
	}

	fn ui (&mut self, ui: &mut egui::Ui, _: &'static cgv::Player)
	{
		// Add the standard 2-column layout control grid
		cgv::gui::layout::ControlTableLayouter::new(ui).layout(
			ui, "Cgv.Ex.Shaders",
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
	// Immediately hand off control flow, passing in a factory for our online shader compilation demo app
	cgv::Player::run(createOnlineShadersDemo)
}
