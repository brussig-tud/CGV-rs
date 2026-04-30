
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
use std::default::Default;

// CGV re-imports
use cgv::{wgpu, glm, egui, tracing};

// CGV-rs Framework
use cgv::{self, util};



//////
//
// Statics
//

const DATA_POINTS: &[PointRTNC; 8] = &[
	// Front side:
	PointRTNC {
		pos_rad: glm::Vec4::new(-1., -1., -1., 1.),
		tangent: glm::Vec4::new(1., 0., 0., 0.),
		color: cgv::RGBA::from_rgba_premultiplied(1., 1., 1., 1.),
		normal: glm::Vec3::new(0., 0., -1.),
	},
	PointRTNC {
		pos_rad: glm::Vec4::new(1., -1., -1., 1.),
		tangent: glm::Vec4::new(-1., 1., 0., 0.),
		color: cgv::RGBA::from_rgba_premultiplied(1., 0., 0., 1.),
		normal: glm::Vec3::new(0., 0., -1.,)
	},
	PointRTNC {
		pos_rad: glm::Vec4::new(-1., 1., -1., 1.),
		tangent: glm::Vec4::new(1., 0., 0., 0.),
		color: cgv::RGBA::from_rgba_premultiplied(0., 1., 0., 1.),
		normal: glm::Vec3::new(0., 0., -1.,)
	},
	PointRTNC {
		pos_rad: glm::Vec4::new(1., 1., -1., 1.),
		tangent: glm::Vec4::new(0., 0., 1., 0.),
		color: cgv::RGBA::from_rgba_premultiplied(0., 0., 1., 1.),
		normal: glm::Vec3::new(1., 0., 0.,)
	},

	// Back side:
	PointRTNC {
		pos_rad: glm::Vec4::new(1., 1., 1., 1.),
		tangent: glm::Vec4::new(-1., 0., 0., 0.),
		color: cgv::RGBA::from_rgba_premultiplied(1., 1., 1., 1.),
		normal: glm::Vec3::new(0., 0., 1.,)
	},
	PointRTNC {
		pos_rad: glm::Vec4::new(-1., 1., 1., 1.),
		tangent: glm::Vec4::new(1., -1., 0., 0.),
		color: cgv::RGBA::from_rgba_premultiplied(1., 0., 0., 1.),
		normal: glm::Vec3::new(0., 0., 1.,)
	},
	PointRTNC {
		pos_rad: glm::Vec4::new(1., -1., 1., 1.),
		tangent: glm::Vec4::new(-1., 0., 0., 0.),
		color: cgv::RGBA::from_rgba_premultiplied(0., 1., 0., 1.),
		normal: glm::Vec3::new(0., 0., 1.,)
	},
	PointRTNC {
		pos_rad: glm::Vec4::new(-1., -1., 1., 1.),
		tangent: glm::Vec4::new(-1., 0., 0., 0.),
		color: cgv::RGBA::from_rgba_premultiplied(0., 0., 1., 1.),
		normal: glm::Vec3::new(0., 0., 1.,)
	}
];

const TOPOLOGY: &[u32; 10] = &[/*front*/0, 1, 2, 3,  /*degen*/3, 5,  /*back*/5, 4, 7, 6];



//////
//
// Structs
//

////
// Point

/// **TODO: move into to-be-created `media` module.**
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PointRTNC {
	pub pos_rad: glm::Vec4,
	pub tangent: glm::Vec4, // contains radius derivative also
	pub color: cgv::RGBA,
	pub normal: glm::Vec3,
}
// TODO: implement related traits


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


	////
	// Initialize renderers

	let sphereRenderer = cgv::renderer::Spheres::new(context, renderSetup);


	////
	// Initialize GUI state

	let guiState = GuiState {};


	////
	// Done!

	// Construct the instance and put it in a box
	Ok(Box::new(RenderersDemo { sphereRenderer: cgv::renderer::Managed::new(sphereRenderer), guiState }))
}

#[derive(Default,Debug)]
struct GuiState {}

struct RenderersDemo
{
	// The renderable test data
	//spheresData: cgv::renderer::spheres::GpuData,

	// Test sphere renderer
	sphereRenderer: cgv::renderer::Managed<cgv::renderer::Spheres>,

	// GUI-controllable state
	guiState: GuiState
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
		&mut self, context: &cgv::Context, renderSetup: &cgv::RenderSetup, globalPasses: &cgv::GlobalPasses,
	){
		// Let our renderers now of the new render states
		self.sphereRenderer.rebuildForGlobalPasses(context, globalPasses);
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
		None // we don't need the Player to submit any custom command buffers for us
	}

	fn ui (&mut self, ui: &mut egui::Ui, player: &mut cgv::Player)
	{
		// Keep track of whether we need to redraw our scene contents
		let mut redraw = false;

		// Renderer configuration
		egui::CollapsingHeader::new("Renderer").default_open(true).show(ui, |ui|
		{
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
