
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

/// Submodule implementing various auxiliary functionality.
mod helpers;



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

// Local imports
use helpers::*;



//////
//
// Structs
//

////
// Point

/// A "data point" suitable for interleaved storage, providing all attributes we're testing out here. 
#[repr(C)]
#[derive(Clone, renderer::data::InterleavedElem)]
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
	// Defaults

	let guiState = GuiState {
		numDataPoints: 64
	};


	////
	// Prepare data

	// Generate initial test data
	let mut testData = TestData::default();
	let renderData = testData.regenerateData(context, guiState.numDataPoints);


	////
	// Initialize renderers

	let mut sphereRenderer = renderer::Managed::new(renderer::Spheres::new(context, renderSetup));
	sphereRenderer.setData(renderer::spheres::DataReceiver::new(renderData.clone()));
	sphereRenderer.setDefaults(context, |d| {
		d.radius = 7./64.; // these defaults will actually get overridden by our data since we have all attributes
		d.color = cgv::RGBA::from_srgba_premultiplied(127, 127, 127, 255);
	});


	////
	// Done!

	// Construct the instance and put it in a box
	Ok(Box::new(RenderersDemo { testData, renderData, sphereRenderer, guiState }))
}

/// Test data holder. We make this its own struct so it can be used even before the `RenderersDemo` is fully
/// constructed.
#[derive(Default)]
struct TestData {
	samples: Vec<DataPoint>
}
impl TestData
{
	/// (Re-)generate the test data and upload to the given `renderer::GpuData`.
	fn regenerateData (&mut self, context: &cgv::Context, num: usize) -> Arc<renderer::data::InterleavedBuffer> {
		regenerateData(&mut self.samples, num);
		renderer::data::InterleavedBuffer::fromHost(
			context, &self.samples, /* options: */Default::default(), Some("RenderersDemo_spheresData")
		)
	}
}
impl std::ops::Deref for TestData {
	type Target = Vec<DataPoint>;

	fn deref (&self) -> &Self::Target {
		&self.samples
	}
}

/// Backing state for the GUI controls
#[derive(Default,Debug)]
struct GuiState {
	numDataPoints: usize
}

struct RenderersDemo
{
	// The renderable test data.
	testData: TestData,

	/// GPU buffer containing the test data.
	renderData: Arc<renderer::data::InterleavedBuffer>,

	// Test sphere renderer.
	sphereRenderer: renderer::Managed<renderer::Spheres>,

	// GUI-controllable state.
	guiState: GuiState
}
impl RenderersDemo
{
	fn regenerateData (&mut self, player: &cgv::Player) {
		self.renderData = self.testData.regenerateData(&player.context, self.guiState.numDataPoints);
		self.sphereRenderer.setDataWithPlayer(
			&player.context, player, renderer::spheres::DataReceiver::new(self.renderData.clone())
		);
	}
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
		let mut redraw = false;

		// Test data configuration
		egui::CollapsingHeader::new("Data").default_open(true).show(ui, |ui|
			cgv::gui::layout::ControlTableLayouter::new(ui)
				.layout(ui, "Cgv.Ex.Renderers-Data", |controlTable| {
					if controlTable.add("Num. Data Points", |ui, _|
						ui.add(
							egui::Slider::new(&mut self.guiState.numDataPoints, 8..=2482176).logarithmic(true),
						).changed()
					){
						self.regenerateData(player);
						redraw = true;
					}
				})
		);

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
