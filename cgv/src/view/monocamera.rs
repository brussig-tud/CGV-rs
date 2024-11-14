
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
	framebuffer: hal::DynamicFramebuffer<'own>,
	renderState: Box<RenderState<'own>>,
	globalPasses: Vec<GlobalPassDeclaration<'static>>
}

impl<'rs> MonoCamera<'rs>
{
	pub fn new (
		context: &Context, renderTarget: RenderTarget, renderSetup: &RenderSetup, name: Option<&str>
	) -> Box<Self>
	{
		// Determine name
		let name: String = if let Some(name) = name { name } else { "UnnamedMonoCamera" }.into();

		// Create owned framebuffer if requested
		let framebuffer = match renderTarget {
			RenderTarget::Internal(dims, colorFormat, depthStencilFormat)
			=> {
				let fb = hal::FramebufferBuilder::withDims(&dims)
					.withLabel(format!("{name}_framebuffer").as_str())
					.attachColor(colorFormat, None)
					.attachDepthStencil(depthStencilFormat, Some(wgpu::TextureUsages::COPY_SRC))
					.build(context);
				hal::DynamicFramebuffer::Owned(fb)
			},

			RenderTarget::Provided(framebuffer)
			=> hal::DynamicFramebuffer::Borrowed(framebuffer)
		};

		// Initialize the main (and only) render state
		let renderState = Box::new(RenderState::new(
			context, framebuffer.as_ref(), Some(format!("{name}_renderState").as_str())
		));

		// Construct
		Box::new(Self {
			name,
			framebuffer,
			renderState,
			globalPasses: vec![GlobalPassDeclaration {
				pass: GlobalPass::Simple,
				renderState: util::mutify(&renderState),
				completionCallback: None,
			}],
		})
	}
}

impl<'rs> Camera for MonoCamera<'rs>
{
	fn projection (&self, _: &GlobalPassDeclaration) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.data.projection
	}
	fn projectionAt (&self, _: &glm::UVec2) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.data.projection
	}

	fn view (&self, _: &GlobalPassDeclaration) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.data.view
	}
	fn viewAt (&self, _: &glm::UVec2) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.data.view
	}

	fn resize (&mut self, context: &Context, viewportDims: &glm::UVec2, interactor: &dyn CameraInteractor)
	{
		if let hal::DynamicFramebuffer::Owned(fb) = &mut self.framebuffer {
			fb.resize(context, viewportDims);
		}
		self.renderState.viewingUniforms.data.projection = interactor.projection(viewportDims);
	}

	fn update (&mut self, interactor: &dyn CameraInteractor) {
		self.renderState.viewingUniforms.data.view = *interactor.view();
	}

	fn declareGlobalPasses (&self) ->&[GlobalPassDeclaration] {
		self.globalPasses.as_slice()
	}

	fn name (&self) -> &str { &self.name }

	fn getDepthReadbackDispatcher (&self, pixelCoords: &glm::UVec2) -> Option<DepthReadbackDispatcher>
	{
		self.framebuffer.as_ref().depthStencil().map(|depthStencil| { DepthReadbackDispatcher::new(
			&pixelCoords, &Viewport {
				min: glm::vec2(0u32, 0u32), extend: depthStencil.dimsWH()
			},
			self.projectionAt(&pixelCoords), self.viewAt(&pixelCoords), &depthStencil
		)})
	}
}
