
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
	pub fn new (context: &Context, allocateOwnedRenderTarget: Option<glm::UVec2>, name: Option<&str>)
		-> Box<Self>
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

		let mut result = Box::new(Self {
			name,
			globalPasses: vec![],
			renderTarget
		});
		/* setup global passes */ {
			let selfRef = util::mutify(result.as_mut());
			result.globalPasses.push(GlobalPassDeclaration {
				pass: GlobalPass::Simple,
				renderTarget: if result.renderTarget.is_some() {
					Some(util::statify(result.renderTarget.as_ref().unwrap().as_ref()))
				} else {
					None
				},
				completionCallback: Some(Box::new(|context, pass| {
					selfRef.globalPassDone(context, pass);
				})),
			});
		}

		result
	}

	fn globalPassDone (&mut self, _: &Context, pass: &GlobalPass) {
		tracing::debug!("Camera[{:?}]: Global pass done: {:?}", self.name.as_str(), pass);
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
