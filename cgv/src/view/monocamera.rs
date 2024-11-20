
//////
//
// Imports
//

// Standard library
/* Nothing here yet */

// Local imports
use crate::*;
use view::*;



//////
//
// Classes
//

////
// MonoCamera

/// A camera producing a single, monoscopic image of the scene.
pub struct MonoCamera<'own> {
	name: String,
	framebuffer: &'own hal::Framebuffer,
	renderState: Box<RenderState>,
	globalPasses: Vec<GlobalPassDeclaration<'own>>,
	intrinsics: Intrinsics,
	extrinsics: Extrinsics
}

impl MonoCamera<'_>
{
	pub fn new (
		context: &Context, renderSetup: &RenderSetup, resolution: glm::UVec2, colorFormat: wgpu::TextureFormat,
		depthStencilFormat: hal::DepthStencilFormat, name: Option<&str>
	) -> Self
	{
		// Determine name
		let name: String = if let Some(name) = name { name } else { "UnnamedMonoCamera" }.into();

		// Create framebuffer
		let framebuffer = hal::FramebufferBuilder::withDims(&resolution)
			.withLabel(format!("{name}_framebuffer").as_str())
			.attachColor(colorFormat, Some(wgpu::TextureUsages::TEXTURE_BINDING))
			.attachDepthStencil(depthStencilFormat, Some(wgpu::TextureUsages::COPY_SRC))
			.build(context);

		// Initialize the main (and only) render state
		let renderState = Box::new(RenderState::new(
			context, framebuffer, Some(format!("{name}_renderState").as_str())
		));

		// Construct
		Self {
			name,
			framebuffer: util::statify(&renderState.framebuffer),
			globalPasses: vec![GlobalPassDeclaration {
				info: GlobalPassInfo {
					pass: GlobalPass::Simple,
					renderState: util::statify(&renderState),
					clearColor: *renderSetup.defaultClearColor(),
					depthClearValue: renderSetup.defaultDepthClearValue(),
				},
				completionCallback: None,
			}],
			renderState,
			intrinsics: Intrinsics::defaultWithAspect(resolution.x as f32 / resolution.y as f32),
			extrinsics: Default::default()
		}
	}
}

impl Camera for MonoCamera<'_>
{
	fn projection (&self, _: &GlobalPassDeclaration) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.borrowData().projection
	}
	fn projectionAt (&self, _: glm::UVec2) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.borrowData().projection
	}

	fn view (&self, _: &GlobalPassDeclaration) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.borrowData().view
	}
	fn viewAt (&self, _: glm::UVec2) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.borrowData().view
	}

	fn intrinsics (&self) -> &Intrinsics {
		&self.intrinsics
	}

	fn extrinsics (&self) -> &Extrinsics {
		&self.extrinsics
	}

	fn resize (&mut self, context: &Context, viewportDims: glm::UVec2, interactor: &dyn CameraInteractor)
	{
		self.renderState.framebuffer.resize(context, viewportDims);
		self.renderState.viewingUniforms.borrowData_mut().projection = interactor.projection(viewportDims);
	}

	fn update (&mut self, interactor: &dyn CameraInteractor) -> bool {
		let mats = self.renderState.viewingUniforms.borrowData_mut();
		mats.projection = interactor.projection(self.renderState.framebuffer.dims());
		mats.view = *interactor.view();
		true
	}

	fn declareGlobalPasses (&self) -> &[GlobalPassDeclaration] {
		self.globalPasses.as_slice()
	}
	fn framebuffer (&self) -> &hal::Framebuffer {
		&self.framebuffer
	}

	fn name (&self) -> &str {
		&self.name
	}

	fn getDepthReadbackDispatcher (&self, pixelCoords: glm::UVec2) -> Option<DepthReadbackDispatcher>
	{
		self.framebuffer.depthStencil().map(|depthStencil| { DepthReadbackDispatcher::new(
			&pixelCoords, &Viewport {
				min: glm::vec2(0u32, 0u32), extend: depthStencil.dimsWH()
			},
			self.projectionAt(pixelCoords), self.viewAt(pixelCoords), &depthStencil
		)})
	}
}
