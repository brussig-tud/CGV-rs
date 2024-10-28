
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
// Structs
//



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
	globalPasses: Vec<GlobalPassDeclaration<'static>>
}

impl MonoCamera
{
	pub fn new (context: &Context, allocateOwnedRenderTarget: Option<glm::UVec2>, name: Option<&str>) -> Self
	{
		// Determine name
		let name: String = if let Some(name) = name { name } else { "UnnamedMonoCamera" }.into();

		// Initialize render target if requested
		let renderTarget = if let Some(dims) = &allocateOwnedRenderTarget {
			Some(Box::new(hal::RenderTarget::new(
				context, dims, wgpu::TextureFormat::Bgra8Unorm, hal::DepthStencilFormat::D32, name.as_str()
			)))
		}
		else {
			None
		};

		Self {
			name,
			globalPasses: vec![GlobalPassDeclaration {
				pass: GlobalPass::Simple,
				renderTarget: if renderTarget.is_some() {
					Some(util::statify(renderTarget.as_ref().unwrap().as_ref()))
				} else {
					None
				},
				completionCallback: Box::new(|_, _| {}),
			}],
			renderTarget
		}
	}
}

impl Camera for MonoCamera
{
	fn resize (&mut self, context: &Context, viewportDims: &glm::UVec2) {
		if self.renderTarget.is_some() {
			self.renderTarget = Some(Box::new(hal::RenderTarget::new(
				context, viewportDims, wgpu::TextureFormat::Bgra8Unorm, hal::DepthStencilFormat::D32, self.name.as_str()
			)))
		}
	}

	fn declareGlobalPasses (&self) ->&[GlobalPassDeclaration] {
		self.globalPasses.as_slice()
	}
}
