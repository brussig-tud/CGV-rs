
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

fn init (player: &mut cgv::Player)
{
	// Tracing
	tracing::info!("Creating \"Renderers\" example application");
	tracing::info!("{:?}", player.runenv());

	let context = &player.state.context;

	////
	// Defaults

	let mut guiState = GuiState {
		numDataPoints: 128, radiusScale: 1., defaultRadius: 7./64.,
		defaultColor: Default::default(), // <- we'll set this to the renderer's default later
		radiiFromData: true, colorsFromData: true
	};


	////
	// Prepare data

	// Generate initial test data
	let mut testData = TestData::default();
	let renderData = testData.regenerateData(context, guiState.numDataPoints);


	////
	// Initialize renderers

	let mut sphereRenderer = renderer::Managed::new(renderer::Spheres::new(context, &player.renderSetup));
	sphereRenderer.setData(renderer::spheres::DataReceiver::new(renderData.clone()));
	sphereRenderer.setStyleUniforms(context, |u| {
		u.radiusScale = guiState.radiusScale;
		u.defaultRadius = guiState.defaultRadius;
		guiState.defaultColor = u.defaultColor.into();
	});


	////
	// Done!

	player.addApp(Box::new(cgv::AppObject::from(RenderersDemo {
		testData, renderData, sphereRenderer, guiState
	})), true).unwrap();
}

/// Test data holder. We make this its own struct so it can be used even before the `RenderersDemo` is fully
/// constructed.
#[derive(Default)]
struct TestData {
	samples: Vec<DataPoint>
}
impl TestData
{
	/// (Re-)generate the test data and upload to a new [`InterleavedBuffer`](renderer::data::InterleavedBuffer).
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
struct GuiState
{
	numDataPoints: usize,
	radiusScale: f32,
	defaultRadius: f32,
	defaultColor: egui::Color32,
	radiiFromData: bool,
	colorsFromData: bool
}

struct RenderersDemo
{
	/// The renderable test data.
	testData: TestData,

	/// GPU buffer containing the test data.
	renderData: Arc<renderer::data::InterleavedBuffer>,

	/// Test sphere renderer.
	sphereRenderer: renderer::Managed<renderer::Spheres>,

	/// GUI-controllable state.
	guiState: GuiState
}
impl RenderersDemo
{
	/// Assign the current GPU-side [render data](Self::renderData) to the currently selected renderer.
	fn reassignData (&mut self, player: &cgv::Player)
	{
		// Decide which attributes to use from the data
		use renderer::data::GAF;
		let mut dataAttribs = renderer::data::GeometryAttributeFlags::empty();
		if self.guiState.radiiFromData { dataAttribs |= GAF::RADII; }
		if self.guiState.colorsFromData { dataAttribs |= GAF::COLORS; }

		// (Re-)assign with the selected attributes
		self.sphereRenderer.setDataWithPlayer(&player.context, player, renderer::spheres::DataReceiver::withAttributes(
			self.renderData.clone(), dataAttribs
		));
	}

	/// Re-generate the test data and upload to the GPU.
	fn regenerateData (&mut self, player: &cgv::Player) {
		self.renderData = self.testData.regenerateData(&player.context, self.guiState.numDataPoints);
		self.reassignData(player);
	}
}
impl cgv::Application<RenderersDemo> for cgv::AppObject<RenderersDemo>
{
	fn title (self: &Self) -> &str {
		"Renderers Demo"
	}

	fn preInit (self: &mut Self, _: &mut cgv::Player) -> cgv::Result<()> {
		// We don't need to perform any pre-initialization
		Ok(())
	}

	fn recreatePipelines (
		self: &mut Self, context: &cgv::Context, _: &cgv::RenderSetup, globalPasses: &cgv::GlobalPasses,
	){
		// Let our renderers know of the new render states
		self.sphereRenderer.rebuildForGlobalPasses(context, *globalPasses);
	}

	fn postInit (self: &mut Self, player: &mut cgv::Player) -> cgv::Result<()>
	{
		// Tracing
		tracing::info!("Positioning initial camera");

		// Make sure the camera is where we want it to be (assuming we're the only application that cares about that)
		let cam = player.camera.parameters_mut();
		cam.intrinsics.f = 2.;
		cam.extrinsics.eye = glm::vec3(0., 0., 2.);
		Ok(())
	}

	fn input (self: &mut Self, _: &cgv::InputEvent, _: &mut cgv::Player) -> cgv::EventOutcome {
		// We're not reacting to any input
		cgv::EventOutcome::NotHandled
	}

	fn resize (self: &mut Self, _: &cgv::Context, _: glm::UVec2) {
		/* We don't have anything to adapt to a new main framebuffer size */
	}

	fn update (self: &mut Self, _: &mut cgv::Player) -> bool {
		// We're not updating anything, so no need to redraw from us
		false
	}

	fn prepareFrame (self: &mut Self, _: &cgv::Context, _: &cgv::RenderState, _: &cgv::GlobalPassInfo)
	-> Option<Vec<wgpu::CommandBuffer>> {
		// We don't need any additional preparation.
		None
	}

	fn render (
		self: &mut Self, context: &cgv::Context, renderState: &cgv::RenderState, renderPass: &mut wgpu::RenderPass,
		globalPass: &cgv::GlobalPassInfo
	) -> Option<Vec<wgpu::CommandBuffer>>
	{
		// Render our test data
		self.sphereRenderer.renderForGlobalPass(context, renderState, renderPass, globalPass.index);
		None // <- we don't need the Player to submit any custom command buffers for us
	}

	fn ui (self: &mut Self, ui: &mut egui::Ui, player: &mut cgv::Player)
	{
		// Test data configuration
		egui::CollapsingHeader::new("Data").default_open(true).show(ui, |ui|
			cgv::gui::layout::ControlTableLayouter::new(ui)
				.layout(ui, "Cgv.Ex.Renderers-Data", |controlTable|
				{
					if controlTable.add("Count", |ui, _| {
						ui.spacing_mut().slider_width *= 0.9;
						ui.add(
							egui::Slider::new(&mut self.guiState.numDataPoints, 8..=2482176).logarithmic(true),
						).changed()
					}){
						self.regenerateData(player);
						player.requireSceneRedraw();
					}
				}
			)
		);

		// Renderer configuration
		egui::CollapsingHeader::new("Renderer").default_open(true).show(ui, |ui|
			cgv::gui::layout::ControlTableLayouter::new(ui)
				.layout(ui, "Cgv.Ex.Renderers-Settings", |controlTable|
				{
					if controlTable.add("Defaults", |ui, _| {
						ui.label(" color ");
						ui.color_edit_button_srgba(&mut self.guiState.defaultColor).changed()
					}){
						self.user.sphereRenderer.setStyleUniforms(&player.context, |u|
							u.defaultColor = self.user.guiState.defaultColor.into()
						);
						player.requireSceneRedraw();
					};
					if controlTable.add("", |ui, _| {
						ui.label("radius");
						ui.spacing_mut().slider_width *= 0.65;
						ui.add(
							egui::Slider::new(&mut self.guiState.defaultRadius, 0.0625f32..=8.)
								.logarithmic(true).max_decimals(3),
						).changed()
					}){
						self.user.sphereRenderer.setStyleUniforms(&player.context, |u|
							u.defaultRadius = self.user.guiState.defaultRadius
						);
						player.requireSceneRedraw();
					}
					controlTable.add("Use attribs", |ui, _| {
						if ui.checkbox(&mut self.guiState.radiiFromData, "radii").changed() {
							self.reassignData(player);
							player.requireSceneRedraw();
						}
						ui.add_space(0.5*ui.style().spacing.item_spacing.x);
						if ui.checkbox(&mut self.guiState.colorsFromData, "colors").changed() {
							self.reassignData(player);
							player.requireSceneRedraw();
						}
					});
					if controlTable.add("Radius scale", |ui, _|
						ui.add(
							egui::Slider::new(&mut self.guiState.radiusScale, 0.0625f32..=8.).logarithmic(true),
						).changed()
					){
						self.user.sphereRenderer.setStyleUniforms(&player.context, |u|
							u.radiusScale = self.user.guiState.radiusScale
						);
						player.requireSceneRedraw();
					}
				}
			)
		);

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
	}
}



//////
//
// Functions
//

/// The application entry point.
pub fn main () -> cgv::Result<()> {
	// Immediately hand off control flow, passing in a callback to create our ExampleApplication
	cgv::Player::run(Box::new(init))
}
