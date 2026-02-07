
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

const QUAD_VERTS: &[QuadVertex; 8] = &[
	QuadVertex {
		pos: glm::Vec4::new(-1., -1., 0., 1.),
		color: glm::Vec4::new(1., 1., 1., 1.),
		texcoord: glm::Vec2::new(0., 1.)
	},
	QuadVertex {
		pos: glm::Vec4::new(1., -1., 0., 1.),
		color: glm::Vec4::new(1., 1., 1., 1.),
		texcoord: glm::Vec2::new(1., 1.)
	},
	QuadVertex {
		pos: glm::Vec4::new(-1., 1., 0., 1.),
		color: glm::Vec4::new(1., 1., 1., 1.),
		texcoord: glm::Vec2::new(0., 0.)
	},
	QuadVertex {
		pos: glm::Vec4::new(1., 1., 0., 1.),
		color: glm::Vec4::new(1., 1., 1., 1.),
		texcoord: glm::Vec2::new(1., 0.)
	},

	QuadVertex {
		pos: glm::Vec4::new(-1., -1., 0., 1.),
		color: glm::Vec4::new(1., 1., 1., 1.),
		texcoord: glm::Vec2::new(1., 1.)
	},
	QuadVertex {
		pos: glm::Vec4::new(1., -1., 0., 1.),
		color: glm::Vec4::new(1., 1., 1., 1.),
		texcoord: glm::Vec2::new(0., 1.)
	},
	QuadVertex {
		pos: glm::Vec4::new(-1., 1., 0., 1.),
		color: glm::Vec4::new(1., 1., 1., 1.),
		texcoord: glm::Vec2::new(1., 0.)
	},
	QuadVertex {
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
// QuadVertex

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct QuadVertex {
	pos: glm::Vec4,
	color: glm::Vec4,
	texcoord: glm::Vec2
}
impl QuadVertex
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
// Our color data we'll send to the shader as a uniform block

#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct UserColors
{
	/// The color of the CGV logo.
	pub logo: egui::Rgba,

	/// The color of the CGV logo.
	pub background: egui::Rgba,

	/// The color of light checkers (visible where the CGV logo texture is not fully opaque).
	pub oddCheckers: egui::Rgba,

	/// The color of dark checkers (visible where the CGV logo texture is not fully opaque).
	pub evenCheckers: egui::Rgba
}
pub type UserColorsUniformGroup = cgv::hal::UniformGroup<UserColors>;


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
			label: Some("ExBasic__QuadVertices"),
			contents: util::slicify(QUAD_VERTS),
			usage: wgpu::BufferUsages::VERTEX
		}
	);

	// Index buffer
	let indexBuffer = context.device().create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("ExBasic__QuadIndices"),
			contents: util::slicify(INDICES),
			usage: wgpu::BufferUsages::INDEX
		}
	);


	////
	// Load resources

	// The example shader
	// - load the shader package we pre-built while the crate was compiled. We could load it from the filesystem
	//   during runtime using `Package::fromFile`, but for easy portability of the executable, we bake it into the
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


	////
	// Bind groups

	// Colors uniform
	let colorUniforms = cgv::hal::UniformGroup::create(
		context, wgpu::ShaderStages::FRAGMENT, Some("ExBasic__colorUniforms").as_deref()
	);

	// Texture uniform
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
	// Initialize GUI state

	let guiState = GuiState {
		logoColor: egui::Color32::from_rgba_premultiplied(
			0, 48, 94, 255   // <- CGV blue in sRGB (linear  would be [0, 7.54, 28.55]
		),
		backgroundColor: egui::Color32::from_rgba_unmultiplied(255, 255, 255, 166),
		oddCheckersColor: egui::Color32::from_rgba_premultiplied(144, 144, 144, 255),
		evenCheckersColor: egui::Color32::from_rgba_premultiplied(255, 255, 255, 255),
		drawBackside: true
	};


	////
	// Done!

	// Construct the instance and put it in a box
	Ok(Box::new(ExampleApplication {
		shader, colorUniforms, texBindGroupLayout, texBindGroup, vertexBuffer, indexBuffer, guiState,
		pipelines: Vec::new(), // <- delayed, *CGV-rs* has a dedicated cycle for this as typically we don't have all
	}))                        //    required information at this point, like viewport dimensions
}

#[derive(Default,Debug)]
struct GuiState {
	/// Proxy for [`UserColors::logo`].
	pub logoColor: egui::ecolor::Color32,

	/// Proxy for [`UserColors::background`].
	pub backgroundColor: egui::ecolor::Color32,

	/// Proxy for [`UserColors::oddCheckers`].
	pub oddCheckersColor: egui::ecolor::Color32,

	/// Proxy for [`UserColors::evenCheckers`].
	pub evenCheckersColor: egui::ecolor::Color32,

	/// Whether to draw the quad's backside (relative to the initial viewing direction)
	pub drawBackside: bool
}

#[derive(Debug)]
struct ExampleApplication
{
	// Rendering related
	shader: wgpu::ShaderModule,
	colorUniforms: UserColorsUniformGroup,
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
				bind_group_layouts: &[
					&renderSetup.bindGroupLayouts().viewing, &self.colorUniforms.bindGroupLayout,
					&self.texBindGroupLayout
				],
				push_constant_ranges: &[],
			});
		context.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("ExBasic__RenderPipeline"),
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
impl cgv::Application for ExampleApplication
{
	fn title (&self) -> &str {
		"Basic Example App"
	}

	fn preInit (&mut self, context: &cgv::Context, _: &cgv::Player) -> cgv::Result<()>
	{
		// Upload initial uniform values
		let colors = self.colorUniforms.borrowData_mut();
		colors.logo = self.guiState.logoColor.into();
		colors.background = self.guiState.backgroundColor.into();
		colors.evenCheckers = self.guiState.oddCheckersColor.into();
		colors.oddCheckers = self.guiState.evenCheckersColor.into();
		self.colorUniforms.upload(context);

		// Done!
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
		renderPass.set_bind_group(1, &self.colorUniforms.bindGroup, &[]);
		renderPass.set_bind_group(2, &self.texBindGroup, &[]);
		renderPass.set_vertex_buffer(0, self.vertexBuffer.slice(..));
		renderPass.set_index_buffer(self.indexBuffer.slice(..), wgpu::IndexFormat::Uint32);
		renderPass.draw_indexed(
			0..if self.guiState.drawBackside {10} else {4}, 0, 0..1
		);
		None // we don't need the Player to submit any custom command buffers for us
	}

	fn ui (&mut self, ui: &mut egui::Ui, player: &'static cgv::Player)
	{
		// Keep track of whether we need to redraw our scene contents
		let mut redraw = false;

		// Appearance section
		egui::CollapsingHeader::new("Appearance").default_open(true).show(ui, |ui|
		{
			// Mutable access to our color uniforms in case something changes
			let colorUniforms = self.colorUniforms.borrowData_mut();

			// Add the standard 2-column layout control grid
			cgv::gui::layout::ControlTableLayouter::new(ui)
			.layout(ui, "Cgv.Ex.Basic-color", |controlTable|
			{
				let mut uploadFlag = false;
				controlTable.add("Logo colors", |ui, _| {
					if ui.color_edit_button_srgba(&mut self.guiState.logoColor).changed() {
						colorUniforms.logo = self.guiState.logoColor.into();
						uploadFlag = true;
					}
					ui.label("logo (foreground)");
				});
				controlTable.add("", |ui, _| {
					if ui.color_edit_button_srgba(&mut self.guiState.backgroundColor).changed() {
						colorUniforms.background = self.guiState.backgroundColor.into();
						uploadFlag = true;
					};
					ui.label("background");
				});
				controlTable.add("Canvas colors", |ui, _| {
					if ui.color_edit_button_srgba(&mut self.guiState.oddCheckersColor).changed() {
						colorUniforms.evenCheckers = self.guiState.oddCheckersColor.into();
						uploadFlag = true;
					}
					ui.label("odd checkers");
				});
				controlTable.add("", |ui, _| {
					if ui.color_edit_button_srgba(&mut self.guiState.evenCheckersColor).changed() {
						colorUniforms.oddCheckers = self.guiState.evenCheckersColor.into();
						uploadFlag = true;
					};
					ui.label("even checkers");
				});

				// Upload new color values if something changed
				if uploadFlag {
					self.colorUniforms.upload(player.context());
					redraw = true;
				}
			});
		});

		// Rendering section
		egui::CollapsingHeader::new("Rendering").default_open(true).show(ui, |ui|
		{
			// Add the standard 2-column layout control grid
			cgv::gui::layout::ControlTableLayouter::new(ui)
			.layout(ui, "Cgv.Ex.Basic-render", |controlTable| {
				redraw |= controlTable.add("geometry", |ui, _| ui.add(
					egui::Checkbox::new(&mut self.guiState.drawBackside, "draw backside")
				)).changed();
			});
		});

		// Make sure the scene will get re-rendered in the current draw pass
		if redraw {
			player.requireSceneRedraw();
		}
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
