
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
	// Prepare online shader compilation

	// Create our Slang compilation context
	tracing::info!("Preparing shader: context creation");
	#[cfg(not(target_arch="wasm32"))] let mut slangCtx = {
		// On native, it's a good idea to always consider the shader path we get from the runtime environment
		cgv::shader::slang::ContextBuilder::withSearchPaths(&environment.shaderPath).build()?
	};
	#[cfg(target_arch="wasm32")] let mut slangCtx = {
		// On WASM, we can't (yet) use a shader path to find modules residing on a filesystem
		cgv::shader::slang::ContextBuilder::default().build()?
	};

	// Load the *CGV-rs* environment containing the core shader lib
	tracing::info!("Preparing shader: loading compilation environment");
	let env = cgv::obtainShaderCompileEnvironment();
	slangCtx.replaceEnvironment(Some(env))?;

	let slangCtxRef = unsafe {
		&mut *(&mut slangCtx as *mut cgv::shader::slang::Context)
	};

	// The user-editable shader code
	let userShaderCode =
		"import \"lib/glyph.slang\";

export struct Glyph: ex::IGlyph = ex::glyphs::Circle;"
			.to_string();
	tracing::info!("Preparing shader: compiling main module '/shader/sdf_demo.slang'");

	// Compile and link our modules
	#[cfg(not(target_arch="wasm32"))] let mainModule = {
		// On native, we can load the shader source from the filesystem
		slangCtxRef.compile(util::pathInsideCrate!("/shader/sdf_demo.slang"))?
	};
	#[cfg(target_arch="wasm32")] let mainModule = {
		// On WASM, we currently have to resort to baking shader source files into the crate. For the same reason (the
		// context does not have runtime filesystem access), we also need to load our local shader "library" into the
		// context manually. This could be largely automated by creating a local *compile environment* and merging it
		// with the one we get from *CGV-rs*, but for our two files we can just do it here.
		use cgv::shader::slang::EnvironmentStorage;
		slangCtxRef.loadModuleFromSource(
			EnvironmentStorage::SourceCode, "lib/sdf.slang",
			util::sourceFile!("/shader/lib/sdf.slang")
		)?;
		slangCtxRef.loadModuleFromSource(
			EnvironmentStorage::SourceCode, "lib/glyph.slang",
			util::sourceFile!("/shader/lib/glyph.slang")
		)?;

		// Now load our actual main module
		slangCtxRef.compileFromNamedSource(
			"sdf_demo.slang", util::sourceFile!("/shader/sdf_demo.slang")
		)?
	};


	////
	// Done!

	let shaderState = ShaderState { slangCtx, mainModule };

	// Construct the instance and put it in a box
	Ok(Box::new(OnlineShadersDemo {
		statusText: "<STATUS UNKNOWN>".into(), shaderState, userShaderCode, pipelines: Vec::new(), vertexBuffer,
		indexBuffer, shader: None, guiState: Default::default()
	}))
}

struct ShaderState<'this> {
	slangCtx: cgv::shader::slang::Context<'this>,
	mainModule: cgv::shader::slang::Module<'this>
}

#[derive(Default)]
struct GuiState {
	showEditor: bool
}

struct OnlineShadersDemo<'this>
{
	// Online shader compilation
	statusText: String,
	shaderState: ShaderState<'this>,
	userShaderCode: String,

	// Rendering related
	shader: Option<wgpu::ShaderModule>,
	pipelines: Vec<wgpu::RenderPipeline>,
	vertexBuffer: wgpu::Buffer,
	indexBuffer: wgpu::Buffer,

	// GUI-controllable state
	guiState: GuiState
}
impl<'this> OnlineShadersDemo<'this>
{
	/// Helper function: create the interfacing pipeline for the given render state.
	fn createPipeline (
		&self, context: &cgv::Context, renderState: &cgv::RenderState, renderSetup: &cgv::RenderSetup
	) -> wgpu::RenderPipeline
	{
		// Tracing
		tracing::info!("Creating pipelines");

		let shader = self.shader.as_ref().unwrap();
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
				module: &shader,
				entry_point: Some("vertexMain"), // Slang (for now) requires explicitly stating entry points
				buffers: &[QuadVertex::layoutDesc()],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
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

	fn relink (
		context: &cgv::Context, slangCtx: &cgv::shader::slang::Context<'this>,
		mainModule: &cgv::shader::slang::Module<'this>, glyphModule: cgv::shader::slang::Module<'this>
	) -> cgv::Result<wgpu::ShaderModule>
	{
		tracing::info!("Preparing shader: specializing for custom glyph");

		// Link into usable program
		tracing::info!("Preparing shader: linking final program");
		let linked = slangCtx.linkComposite(&slangCtx.createComposite(&[
			cgv::shader::compile::ComponentRef::Module(mainModule),
			cgv::shader::compile::ComponentRef::Module(&glyphModule),
		])?)?;
		let shaderPackage = cgv::shader::Package::fromLinkedComposite(
			cgv::shader::WgpuSourceType::mostSuitable(), slangCtx, &linked,
			None, // <- we don't require our purely on-line package to have any particular name
			None  // <- no cherry-picked entry point specializations, just include all possible variants
		)?;
		// - final: obtain the *WGPU* shader module
		tracing::info!("Preparing shader: converting to WGPU shader");
		let shader = shaderPackage.createShaderModuleFromBestInstance(
			context.device(), None, Some("ExShaders__ShaderModule")
		).ok_or(
			cgv::anyhow!("Could not create example shader module")
		)?;

		// Done!
		Ok(shader)
	}

	fn initialCompile (&'this mut self, context: &cgv::Context) -> cgv::Result<()>
	{
		// Load our concrete `IGlyph`-implementing type that our "sdf_demo.slang" module expects into the context
		let glyphModule = self.recompileGlyphModule()?;

		// Link everything together
		self.shader = Some(
			Self::relink(context, &self.shaderState.slangCtx, &self.shaderState.mainModule, glyphModule)?
		);

		// Done!
		Ok(())
	}

	fn recompileGlyphModule (&self)
	-> Result<cgv::shader::slang::Module<'this>, cgv::shader::compile::LoadModuleError> {
		util::extendLifetime(self).shaderState.slangCtx.compileFromSource(&self.userShaderCode)
	}

	fn ui (&'this mut self, ui: &mut egui::Ui, player: &'static cgv::Player) where Self: 'this
	{
		// Code editor
		ui.toggle_value(&mut self.guiState.showEditor, "Show Editor");
		if self.guiState.showEditor
		{
			egui::Window::new("Shader Code").show(ui, |ui|
			{
				// Enable closing with [ESC]
				let quit_shortcut = egui::KeyboardShortcut::new(
					egui::Modifiers::NONE, egui::Key::Escape
				);
				if ui.input_mut(|i| i.consume_shortcut(&quit_shortcut)) {
					self.guiState.showEditor = false;
				}

				// Calculate editor size
				// TODO: lots of empirically determined magic numbers, need to rigorously calculate this
				const CODE_EDITOR_LINES: usize = 5;
				const MESSAGE_PANEL_LINES: usize = 5;
				let availableSize = ui.available_size();
				let lineSize = ui.style().text_styles[&egui::TextStyle::Monospace].size;
				let messagePaneSize = egui::vec2(
					f32::max(64., availableSize.x), MESSAGE_PANEL_LINES as f32 * lineSize
				);
				let editorSize = egui::vec2(
					messagePaneSize.x,
					f32::max(
						CODE_EDITOR_LINES as f32*lineSize, availableSize.y-messagePaneSize.y - 8.
					)
				);

				// Actual editor
				ui.allocate_ui(
					editorSize, |ui| egui::Frame::canvas(ui.style())
					.corner_radius(3.).show(ui, |ui|
					{
						egui::ScrollArea::vertical().id_salt("editorPane").show(ui, |ui|
						{
							let editor = egui::TextEdit::multiline(&mut self.userShaderCode)
								.code_editor()
								.desired_rows(CODE_EDITOR_LINES)
								.lock_focus(true)
								.desired_width(f32::INFINITY)
								.frame(false);
							if ui.add(editor).changed()
							{
								use cgv::shader::compile::LoadModuleError;
								let result = self.recompileGlyphModule();
								self.statusText = match result {
									Ok(glyphModule) => {
										self.shader = Self::relink(
											player.context(), &self.shaderState.slangCtx, &self.shaderState.mainModule,
											glyphModule
										).ok();
										"Code OK.".into()
									},
									Err(err) => match err {
										LoadModuleError::CompilationError(err) => {
											tracing::error!("{err}");
											format!("{err}")
										},

										LoadModuleError::DuplicatePath(_) | LoadModuleError::InvalidModulePath(_)
										=> unreachable!("")
									}
								}
							}
						})
					})
				);

				// Messages panel
				ui.allocate_ui(messagePaneSize, |ui| egui::Frame::NONE.corner_radius(3.).show(ui, |ui|
				{
					egui::ScrollArea::vertical().id_salt("msgPane").show(ui, |ui| {
						let msgPanel = egui::widgets::TextEdit::multiline(&mut self.statusText)
							.frame(false).desired_rows(MESSAGE_PANEL_LINES).desired_width(f32::INFINITY);
						ui.add_enabled(false, msgPanel)
					})
				}));
			});
		}
	}
}
impl<'this> cgv::Application for OnlineShadersDemo<'this>
{
	fn title (&self) -> &str {
		"Online Shader Compilation"
	}

	fn preInit (&mut self, context: &cgv::Context, _: &cgv::Player) -> cgv::Result<()> {
		util::extendLifetime_mut(self).initialCompile(context)
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

	fn ui (&mut self, ui: &mut egui::Ui, player: &'static cgv::Player) {
		// Delegate to extended-lifetime method
		util::extendLifetime_mut(self).ui(ui, player);
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
