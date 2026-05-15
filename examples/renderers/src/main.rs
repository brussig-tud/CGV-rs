
//////
//
// Language config
//

// Eff this convention.
#![allow(non_snake_case)]



//////
//
// Imports
//

// Standard library
use std::{sync::Arc, default::Default};

// CGV re-imports
use cgv::{wgpu, glm, egui, tracing};

// CGV-rs Framework
use cgv::{self, renderer};



//////
//
// Statics
//

const DATA_POINTS: [DataPoint; 8] = [
	// Front side:
	DataPoint {
		pos: glm::Vec3::new(-0.5, -0.5, -0.5), radius: 4./64.,
		tangent: glm::Vec3::new(1., 0., 0.), radDeriv: -1./64.,
		color: cgv::RGBA::from_rgba_premultiplied(1., 1., 1., 1.),
		normal: glm::Vec3::new(0., 0., -1.),
	},
	DataPoint {
		pos: glm::Vec3::new(0.5, -0.5, -0.5), radius: 2./64.,
		tangent: glm::Vec3::new(-1., 1., 0.), radDeriv: 1./64.,
		color: cgv::RGBA::from_rgba_premultiplied(1., 0., 0., 1.),
		normal: glm::Vec3::new(0., 0., -1.,)
	},
	DataPoint {
		pos: glm::Vec3::new(-0.5, 0.5, -0.5), radius: 3./64.,
		tangent: glm::Vec3::new(1., 0., 0.), radDeriv: 1./64.,
		color: cgv::RGBA::from_rgba_premultiplied(0., 1., 0., 0.),
		normal: glm::Vec3::new(0., 0., -1.,)
	},
	DataPoint {
		pos: glm::Vec3::new(0.5, 0.5, -0.5), radius: 3./64.,
		tangent: glm::Vec3::new(0., 0., 1.), radDeriv: -1./64.,
		color: cgv::RGBA::from_rgba_premultiplied(0., 0., 1., 1.),
		normal: glm::Vec3::new(1., 0., 0.,)
	},

	// Back side:
	DataPoint {
		pos: glm::Vec3::new(0.5, 0.5, 0.5), radius: 2./64.,
		tangent: glm::Vec3::new(-1., 0., 0.), radDeriv: -1./128.,
		color: cgv::RGBA::from_rgba_premultiplied(1., 1., 1., 1.),
		normal: glm::Vec3::new(0., 0., 1.,)
	},
	DataPoint {
		pos: glm::Vec3::new(-0.5, 0.5, 0.5), radius: 1./64.,
		tangent: glm::Vec3::new(1., -1., 0.), radDeriv: 2./64.,
		color: cgv::RGBA::from_rgba_premultiplied(1., 0., 0., 1.),
		normal: glm::Vec3::new(0., 0., 1.,)
	},
	DataPoint {
		pos: glm::Vec3::new(0.5, -0.5, 0.5), radius: 4./64.,
		tangent: glm::Vec3::new(-1., 0., 0.), radDeriv: 1./64.,
		color: cgv::RGBA::from_rgba_premultiplied(0., 1., 0., 1.),
		normal: glm::Vec3::new(0., 0., 1.,)
	},
	DataPoint {
		pos: glm::Vec3::new(-0.5, -0.5, 0.5), radius: 4./64.,
		tangent: glm::Vec3::new(-1., 0., 0.), radDeriv: 0.,
		color: cgv::RGBA::from_rgba_premultiplied(0., 0., 1., 1.),
		normal: glm::Vec3::new(0., 0., 1.,)
	}
];

const _TOPOLOGY: &[u32; 10] = &[/*front*/0, 1, 2, 3,  /*degen*/3, 5,  /*back*/5, 4, 7, 6];



//////
//
// Structs
//

////
// Point

/// A "data point" suitable for interleaved storage, providing all attributes we're testing out here. 
#[repr(C)]
#[derive(
	// Standard traits
	Clone,

	// Attributes we have
	renderer::data::InterleavedElem,renderer::data::ElemWithRadius,renderer::data::ElemWithTangent,
	renderer::data::ElemWithRadiusDeriv, renderer::data::ElemWithNormal,renderer::data::ElemWithColor,

	// Attributes we don't have
	renderer::data::NoOrientation,renderer::data::NoScaling
)]
pub struct DataPoint
{
	#[cgv_renderAttr(pos)]         pub pos: glm::Vec3,
	#[cgv_renderAttr(radius)]      pub radius: f32,
	#[cgv_renderAttr(tangent)]     pub tangent: glm::Vec3,
	#[cgv_renderAttr(radiusDeriv)] pub radDeriv: f32,
	#[cgv_renderAttr(normal)]      pub normal: glm::Vec3,
	#[cgv_renderAttr(color)]       pub color: cgv::RGBA,
}


////
// RenderersDemo

/// Factory function for our `RenderersDemo` defined right after. Regular functions with this signature implement the
/// [`cgv::ApplicationFactory`] trait. If more flexibility is required to control how exactly your app is created, you
/// can also write a factory class, i.e. a custom struct that implements `cgv::ApplicationFactory`.
fn createRenderersDemo (context: &cgv::Context, renderSetup: &cgv::RenderSetup, environment: cgv::run::Environment)
	-> cgv::Result<Box<dyn cgv::Application>>
{
	// Tracing
	tracing::info!("Creating \"Renderers\" example application");
	tracing::info!("{:?}", environment);


	////
	// Prepare data

	/* generate test data */
	let spheresData = renderer::data::InterleavedBuffer::fromHost(
		context, &DATA_POINTS, /* options: */Default::default(), Some("RenderersDemo_spheresData")
	);


	////
	// Initialize renderers

	let mut sphereRenderer = renderer::Managed::new(renderer::Spheres::new(context, renderSetup));
	sphereRenderer.setData(renderer::spheres::DataReceiver::new(spheresData.clone()));
	sphereRenderer.setDefaults(context, |d| {
		d.radius = 7./64.; // these defaults will actually get overridden by our data since we have all attributes
		d.color = cgv::RGBA::from_srgba_premultiplied(127, 127, 127, 127);
	});


	////
	// Initialize GUI state

	let guiState = GuiState {};


	////
	// Done!

	// Construct the instance and put it in a box
	Ok(Box::new(RenderersDemo { spheresData, sphereRenderer, _guiState: guiState }))
}

#[derive(Default,Debug)]
struct GuiState {}

struct RenderersDemo
{
	// The renderable test data
	#[expect(dead_code)]
	spheresData: Arc<renderer::data::InterleavedBuffer>,

	// Test sphere renderer
	sphereRenderer: renderer::Managed<renderer::Spheres>,

	// GUI-controllable state
	_guiState: GuiState
}
impl cgv::Application for RenderersDemo
{
	fn title (&self) -> &str {
		"Renderers Demo"
	}

	fn preInit (&mut self, _: &mut cgv::Player) -> cgv::Result<()> {
		// We don't need to perform any pre-initialization
		Ok(())
	}

	fn recreatePipelines (
		&mut self, context: &cgv::Context, _: &cgv::RenderSetup, globalPasses: &cgv::GlobalPasses,
	){
		// Let our renderers know of the new render states
		self.sphereRenderer.rebuildForGlobalPasses(context, *globalPasses);
	}

	fn postInit (&mut self, player: &mut cgv::Player) -> cgv::Result<()>
	{
		// Tracing
		tracing::info!("Positioning initial camera");

		// Make sure the camera is where we want it to be (assuming we're the only application that cares about that)
		let cam = player.camera.parameters_mut();
		cam.intrinsics.f = 2.;
		cam.extrinsics.eye = glm::vec3(0., 0., 2.);
		Ok(())
	}

	fn input (&mut self, _: &cgv::InputEvent, _: &mut cgv::Player, _: cgv::player::Handle) -> cgv::EventOutcome {
		// We're not reacting to any input
		cgv::EventOutcome::NotHandled
	}

	fn resize (&mut self, _: &cgv::Context, _: glm::UVec2) {
		/* We don't have anything to adapt to a new main framebuffer size */
	}

	fn update (&mut self, _: &mut cgv::Player, _: cgv::player::Handle) -> bool {
		// We're not updating anything, so no need to redraw from us
		false
	}

	fn prepareFrame (&mut self, _: &cgv::Context, _: &cgv::RenderState, _: &cgv::GlobalPassInfo)
	-> Option<Vec<wgpu::CommandBuffer>> {
		// We don't need any additional preparation.
		None
	}

	fn render (
		&mut self, context: &cgv::Context, renderState: &cgv::RenderState, renderPass: &mut wgpu::RenderPass,
		globalPass: &cgv::GlobalPassInfo
	) -> Option<Vec<wgpu::CommandBuffer>>
	{
		// Render our test data
		self.sphereRenderer.renderForGlobalPass(context, renderState, renderPass, globalPass.index);

		None // <- we don't need the Player to submit any custom command buffers for us
	}

	fn ui (&mut self, ui: &mut egui::Ui, player: &mut cgv::Player)
	{
		// Keep track of whether we need to redraw our scene contents
		#[expect(unused_mut)] let mut redraw = false;

		// Renderer configuration
		egui::CollapsingHeader::new("Renderer").default_open(true).show(ui, |ui| {
			ui.label("Nothing here yet.");
		});

		// Links section
		ui.add_space(ui.style().spacing.item_spacing.y * 3.);
		egui::CollapsingHeader::new("Links").default_open(true).show(ui, |ui|
			cgv::gui::layout::ControlTableLayouter::new(ui)
			.layout(ui, "Cgv.Ex.Renderers-links", |controlTable| {
				controlTable.add("Source code:", |ui, _|
					ui.hyperlink_to(format!("{} examples/renderers", egui::special_emojis::GITHUB),
					"https://github.com/brussig-tud/CGV-rs/tree/main/examples/renderers")
				)
			})
		);

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
pub fn main () -> cgv::Result<()> {
	// Immediately hand off control flow, passing in a factory for our ExampleApplication
	cgv::Player::run(Box::new(createRenderersDemo))
}
