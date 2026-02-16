
//////
//
// Language config
//

// Eff this convention.
#![allow(non_snake_case)]



//////
//
// Module definitions
//

/// Private submodule holding the code for setting up our custom fonts
mod slang_completer;



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

// Egui library
use cgv::egui_extras as egui_extras;

// CGV Framework
use cgv::{self, util, shader::compile::prelude::*};

// Local imports
use slang_completer::*;



//////
//
// Statics
//

/// The completer syntax definition for the *Slang* shading language.
const SLANG_COMPLETION: std::sync::LazyLock<egui_code_editor::Syntax> = std::sync::LazyLock::new(
	|| egui_code_editor::Syntax::slang()
);

/// The vertices of our example quad that will hold the shaded glyph.
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
	// Setup code editing

	// Syntax highlighting
	// - language definitions
	let syntaxSet = {
		let mut builder = syntect::parsing::SyntaxSet::load_defaults_newlines().into_builder();
		builder.add(syntect::parsing::SyntaxDefinition::load_from_str(
			util::sourceFile!("/res/syntax/HLSL.sublime-syntax"), true, Some("HLSL")
		)?);
		builder.build()
	};
	// - egui interfacing
	let highlighterSettings = egui_extras::syntax_highlighting::SyntectSettings {
		ps: syntaxSet,
		ts: syntect::highlighting::ThemeSet::load_defaults()
	};

	// Keyword-based text completer
	let syntaxCompleter = egui_code_editor::Completer::new_with_syntax(&SLANG_COMPLETION).with_user_words();


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

	// The user-editable shader code
	let userShaderCode = util::sourceFile!("/shader/user.slang").into();

	// Compile and link our modules
	tracing::info!("Preparing shader: compiling static main module '/shader/sdf_demo.slang'");
	#[cfg(not(target_arch="wasm32"))] let mainModule = {
		// On native, we can load the shader source from the filesystem
		slangCtx.compile(util::pathInsideCrate!("/shader/sdf_demo.slang"))?
	};
	#[cfg(target_arch="wasm32")] let mainModule = {
		// On WASM, we currently have to resort to baking shader source files into the crate. For the same reason (the
		// context does not have runtime filesystem access), we also need to load our local shader "library" into the
		// context manually. This could be largely automated by creating a local *compile environment* in the build
		// script and merging it with the one we get from *CGV-rs*, but for our two files, we can just do it here.
		use cgv::shader::slang::EnvironmentStorage;
		slangCtx.loadModuleFromSource(
			EnvironmentStorage::SourceCode, "lib/sdf.slang",
			util::sourceFile!("/shader/lib/sdf.slang")
		)?;
		slangCtx.loadModuleFromSource(
			EnvironmentStorage::SourceCode, "lib/glyph.slang",
			util::sourceFile!("/shader/lib/glyph.slang")
		)?;

		// Now load our actual main module
		slangCtx.compileFromNamedSource(
			"sdf_demo.slang", util::sourceFile!("/shader/sdf_demo.slang")
		)?
	};


	////
	// Done!

	// Construct the instance and put it in a box
	Ok(Box::new(OnlineShadersDemo {
		statusText: "<STATUS UNKNOWN>".into(), mainModule, slangCtx, userShaderCode, pipelines: Vec::new(),
		vertexBuffer, indexBuffer, shader: cgv::util::LaterInit::uninit(), highlighterSettings, syntaxCompleter,
		guiState: Default::default()
	}))
}

#[derive(Default)]
struct GuiState {
	showEditor: bool
}

struct OnlineShadersDemo<'this>
{
	// Online shader compilation
	statusText: String,
	slangCtx: cgv::shader::slang::Context<'this>,
	mainModule: cgv::shader::slang::Module<'this>,
	userShaderCode: String,

	// Rendering related
	shader: cgv::util::LaterInit<wgpu::ShaderModule>,
	pipelines: Vec<wgpu::RenderPipeline>,
	vertexBuffer: wgpu::Buffer,
	indexBuffer: wgpu::Buffer,

	// GUI-related
	highlighterSettings: egui_extras::syntax_highlighting::SyntectSettings,
	syntaxCompleter: egui_code_editor::Completer,
	guiState: GuiState
}
impl OnlineShadersDemo<'_>
{
	/// Helper function: create the interfacing pipeline for the given render state.
	fn createPipeline (
		&self, context: &cgv::Context, renderState: &cgv::RenderState, renderSetup: &cgv::RenderSetup
	) -> wgpu::RenderPipeline
	{
		// Tracing
		tracing::info!("Creating pipelines");

		// Create the pipeline
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

	fn rebuildShader (&mut self, context: &cgv::Context) -> Result<&str, cgv::shader::compile::CompileOrBuildError>
	{
		// Load our concrete `IGlyph`-implementing type that our "sdf_demo.slang" module expects into the context
		tracing::info!("(Re-)building shader: specializing for custom glyph");
		let glyphModule = self.slangCtx.compileFromSource(&self.userShaderCode)?;

		// Link into usable program
		tracing::info!("(Re-)building shader: linking final program");
		let linked = self.slangCtx.linkComposite(&self.slangCtx.createComposite(&[
			cgv::shader::compile::ComponentRef::Module(&self.mainModule),
			cgv::shader::compile::ComponentRef::Module(&glyphModule),
		])?)?;
		let shaderPackage = cgv::shader::Package::fromLinkedComposite(
			cgv::shader::WgpuSourceType::mostSuitable(), &self.slangCtx, &linked,
			None, // <- we don't require our purely on-line package to have any particular name
			None  // <- no cherry-picked entry point specializations, just include all possible variants
		).expect("creating a package from a linked composite should not fail as we set everything up correctly");

		// Obtain the *WGPU* shader module
		tracing::info!("(Re-)building shader: converting to WGPU shader");
		self.shader.set(shaderPackage.createShaderModuleFromBestInstance(
			context.device(), None, Some("ExShaders__ShaderModule")
		).ok_or(
			cgv::anyhow!("Could not create example shader module")
		).expect(
			"creating the WGPU shader from the package should not fail as we compiled to the correct target")
		);

		// Done!
		Ok("Code OK.")
	}
}
impl<'this> cgv::Application for OnlineShadersDemo<'this>
{
	fn title (&self) -> &str {
		"Online Shader Compilation"
	}

	fn preInit (&mut self, context: &cgv::Context, _: &cgv::Player) -> cgv::Result<()> {
		self.statusText = self.rebuildShader(context).map_err(|err| cgv::Error::from(err))?.into();
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
		// Editor toggle
		ui.toggle_value(&mut self.guiState.showEditor, "Show Editor");

		// Editor theme controls (should move into editor window once we get flex box layouting figured out)
		let mut theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(
			ui.ctx(), ui.style()
		).with_font_size(11.);
		ui.collapsing("Editor theme", |ui| theme.ui(ui) );
		theme.store_in_memory(ui.ctx());

		// Links section
		ui.add_space(ui.style().spacing.item_spacing.y * 3.);
		egui::CollapsingHeader::new("Links").default_open(true).show(ui, |ui|
			cgv::gui::layout::ControlTableLayouter::new(ui)
			.layout(ui, "Cgv.Ex.Basic-render", |controlTable| {
				controlTable.add("Source code:", |ui, _|
					ui.hyperlink_to(format!("{} examples/shaders", egui::special_emojis::GITHUB),
					"https://github.com/brussig-tud/CGV-rs/tree/main/examples/shaders")
				)
			})
		);
	}

	fn freeUi (&mut self, ui: &mut egui::Ui, player: &'static cgv::Player)
	{
		// Code editor
		let mut showEditor = self.guiState.showEditor;
		if showEditor
		{
			egui::Window::new("Shader Code").default_width(768.).open(&mut showEditor)
			.show(ui, |ui|
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
				ui.allocate_ui(editorSize,
					|ui| egui::Frame::canvas(ui.style()).corner_radius(3.)
					.show(ui, |ui|
					{
						egui::ScrollArea::vertical().id_salt("editorPane").show(ui, |ui|
						{
							// Retrieve highlighter theme
							let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(
								ui.ctx(), ui.style()
							).with_font_size(11.);

							// Setup layouter
							let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrapWidth: f32|
							{
								let mut layoutJob = egui_extras::syntax_highlighting::highlight_with(
									ui.ctx(), ui.style(), &theme, text.as_str(),
									"hlsl", // <- wait until there is a sublime-text theme for Slang
									&self.highlighterSettings
								);
								layoutJob.wrap.max_width = wrapWidth;
								ui.fonts_mut(|f| f.layout_job(layoutJob))
							};

							// Add editor pane
							// - completer gets first dips on input
							self.syntaxCompleter.handle_input(ui.ctx());
							// - define editor
							let mut editorOutput = egui::TextEdit::multiline(&mut self.userShaderCode)
								.code_editor().desired_rows(CODE_EDITOR_LINES).lock_focus(true)
								.desired_width(f32::INFINITY).frame(false).layouter(&mut layouter)
								.show(ui);
							// - define completer window
							self.syntaxCompleter.show(
								&SLANG_COMPLETION, &egui_code_editor::ColorTheme::AYU_MIRAGE, 11.,
								&mut editorOutput
							);

							// Handle code edits
							if editorOutput.response.changed()
							{
								use cgv::shader::compile::CompileOrBuildError;
								self.statusText = match self.rebuildShader(player.context())
								{
									Ok(statusText) => {
										player.postRecreatePipelines();
										player.requireSceneRedraw();
										statusText.into()
									},
	
									Err(  CompileOrBuildError::CompilationError(err)
									      | CompileOrBuildError::CreateCompositeError(err)
									      | CompileOrBuildError::LinkError(err))
									=> {
										tracing::error!("{err}");
										format!("{err}")
									},
	
									Err(  CompileOrBuildError::DuplicateModulePaths(_)
									      | CompileOrBuildError::InvalidModulePath(_))
									=> unreachable!("")
								}
							}
						})
					})
				);

				// Messages panel
				ui.allocate_ui(
					messagePaneSize, |ui| egui::Frame::NONE.corner_radius(3.).show(
					ui, |ui| {
						egui::ScrollArea::vertical().id_salt("msgPane").show(ui, |ui| {
							let msgPanel = egui::widgets::TextEdit::multiline(&mut self.statusText)
								.frame(false).desired_rows(MESSAGE_PANEL_LINES).desired_width(f32::INFINITY);
							ui.add_enabled(false, msgPanel)
						})
					})
				);
			});
		}
		self.guiState.showEditor &= showEditor;
	}
}



//////
//
// Functions
//

/// The application entry point.
pub fn main () -> cgv::Result<()> {
	// Immediately hand off control flow, passing in a factory for our online shader compilation demo app
	cgv::Player::run(createOnlineShadersDemo)
}
