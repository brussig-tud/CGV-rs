
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
pub struct MonoCamera {
	name: String,
	renderTarget: Option<Box<hal::RenderTarget>>,
	renderState: Box<RenderState>,
	globalPasses: Vec<GlobalPassDeclaration<'static>>
}

impl MonoCamera
{
	pub fn new (
		context: &Context, allocateOwnedRenderTarget: Option<glm::UVec2>, renderSetup: &RenderSetup, name: Option<&str>
	) -> Box<Self>
	{
		// Determine name
		let name: String = if let Some(name) = name { name } else { "UnnamedMonoCamera" }.into();

		// Initialize render target if requested
		let renderTarget = if let Some(dims) = &allocateOwnedRenderTarget {
			Some(Box::new(hal::RenderTarget::new(
				context, dims, renderSetup.colorFormat, renderSetup.depthStencilFormat().into(), name.as_str()
			)))
		}
		else {
			None
		};

		// Initialize the main (and only) render state
		let renderState = Box::new(RenderState::new(
			context, GlobalPass::Simple,
			if let Some(rt) = util::statify(&renderTarget) {
				ColorAttachment::Texture(&rt.color)
			} else {
				ColorAttachment::Surface
			},
			Some(hal::DepthStencilFormat::D32),
			Some((name.clone() + "_renderState").as_str())
		));

		// Create Self with fields initialized up to now
		let mut result = Box::new(Self {
			name,
			renderTarget,
			renderState,
			globalPasses: Vec::new(),
		});

		// setup global passes
		result.globalPasses.push(GlobalPassDeclaration {
			pass: GlobalPass::Simple,
			renderState: util::mutify(&result.renderState),
			completionCallback: None,
		});

		// Done!
		result
	}
}

impl Camera for MonoCamera
{
	fn resize (&mut self, context: &Context, viewportDims: &glm::UVec2)
	{
		if self.renderTarget.is_some() {
			self.renderTarget = Some(Box::new(hal::RenderTarget::new(
				context, viewportDims, wgpu::TextureFormat::Bgra8Unorm, hal::DepthStencilFormat::D32, self.name.as_str()
			)))
		}
		self.renderState.updateSize(context, viewportDims)
	}

	fn update (&mut self, _: &dyn CameraInteractor) {}

	fn declareGlobalPasses (&self) ->&[GlobalPassDeclaration] {
		self.globalPasses.as_slice()
	}

	fn name (&self) -> &str { &self.name }
}
