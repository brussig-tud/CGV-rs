
//////
//
// Imports
//

// Standard library
use std::default::Default;

// WGPU API
use wgpu;

// GLM library
use glm;

// Local imports
use crate::*;



//////
//
// Enums
//

pub enum ColorAttachment<'a> {
	Surface(&'a wgpu::TextureView),
	Texture(&'a hal::Texture)
}



//////
//
// Classes
//

////
// ViewingUniforms

// The CPU-side representation of the UniformBuffer used for storing the viewing information.
#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct ViewingStruct
{
	/// The modelview transformation matrix.
	pub view: glm::Mat4,

	/// The projection matrix.
	pub projection: glm::Mat4
}


////
// DepthStencilAttachment

/// Encapsulates inter-referencing state for depth stencil attachments.
pub struct DepthStencilAttachment {
	pub(crate) texture: hal::Texture,
	defaultState: wgpu::DepthStencilState
}


////
// RenderState

pub struct RenderState
{
	pub viewingUniforms: hal::UniformGroup<ViewingStruct>,

	clearColor: wgpu::Color,
	colorAttachment: ColorAttachment<'static>,

	depthStencilFormat: Option<hal::DepthStencilFormat>,
	pub depthStencilAttachment: Option<DepthStencilAttachment>
}

impl RenderState
{
	pub fn new(
		context: &Context, colorAttachment: ColorAttachment<'static>,
		depthStencilFormat: Option<hal::DepthStencilFormat>
	) -> Self
	{
		////
		// Prepare non-inter-referencing fields

		// Uniforms and associated buffers and bind groups
		let viewingUniforms = hal::UniformGroup::create(
			context, wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT, Some("CGV__ViewingUniforms")
		);


		////
		// Construct result object

		// Henceforth, we mutate this result object for the remaining initialization
		let mut result = Self {
			viewingUniforms,

			clearColor: wgpu::Color{r: 0.3, g: 0.5, b: 0.7, a: 1.},
			colorAttachment,

			depthStencilFormat,
			depthStencilAttachment: None
		};


		////
		// Initialize inter-referencing fields

		result.recreateMainSurfaceDepthStencilAttachment(context);
		result
	}

	fn recreateMainSurfaceDepthStencilAttachment (&mut self, context: &Context)
	{
		// Early-out: no depth/stencil attachment
		if let Some(format) = self.depthStencilFormat
		{
			// Recreate according to selected main depth/stencil mode
			// - initialize fields that are reference targets
			let dims: glm::UVec2 = glm::vec2(context.config.width.max(1), context.config.height.max(1));
			let texture = hal::Texture::createDepthStencilTexture(
				context, &dims, format, Some(wgpu::TextureUsages::COPY_SRC), Some("MainSurfaceDepthStencilTex")
			);
			// - create the attachment struct with trivially initializable fields constructed in-place
			self.depthStencilAttachment = Some(DepthStencilAttachment {
				defaultState: wgpu::DepthStencilState {
					format: texture.texture.format(),
					depth_write_enabled: true,
					depth_compare: wgpu::CompareFunction::Less, // 1.
					stencil: wgpu::StencilState::default(), // 2.
					bias: wgpu::DepthBiasState::default(),
				}, texture
			});
		}
		else {
			self.depthStencilAttachment = None;
		}
	}

	pub fn getMainSurfaceColorAttachment (&self) -> Option<wgpu::RenderPassColorAttachment>
	{
		Some(wgpu::RenderPassColorAttachment {
			view: match self.colorAttachment {
				ColorAttachment::Surface(view) => view,
				ColorAttachment::Texture(tex) => &tex.view
			},
			resolve_target: None,
			ops: wgpu::Operations {
				load: /*match self.clearColor {
					Some(color) => wgpu::LoadOp::Clear(color),
					None => wgpu::LoadOp::Load
				}*/wgpu::LoadOp::Clear(self.clearColor),
				store: wgpu::StoreOp::Store,
			},
		})
	}

	pub fn getMainSurfaceDepthStencilAttachment (&self) -> Option<wgpu::RenderPassDepthStencilAttachment>
	{
		if let Some(dsa) = &self.depthStencilAttachment {
			Some(wgpu::RenderPassDepthStencilAttachment {
				view: &dsa.texture.view,
				depth_ops: Some(wgpu::Operations {
					load: wgpu::LoadOp::Clear(1.)/*wgpu::LoadOp::Load*/,
					store: wgpu::StoreOp::Store,
				}),
				stencil_ops: None,
			})
		}
		else {
			None
		}
	}

	pub fn getMainSurfaceDepthStencilState (&self) -> Option<wgpu::DepthStencilState>
	{
		if let Some(dsa) = &self.depthStencilAttachment {
			Some(dsa.defaultState.clone())
		}
		else {
			None
		}
	}

	pub(crate) fn updateSize (&mut self, context: &Context)
	{
		if context.size == self.depthStencilAttachment.as_ref().unwrap().texture.physicalSizeWH() {
			return;
		}
		self.recreateMainSurfaceDepthStencilAttachment(context);
	}
}


////
// RenderStatePrivateInterface

pub(crate) trait RenderStatePrivateInterface {
	fn updateMainSurfaceColorAttachment (&mut self, context: &Context);
}

impl RenderStatePrivateInterface for RenderState {
	fn updateMainSurfaceColorAttachment (&mut self, context: &Context)
	{
		// Update view and attachment
		match self.colorAttachment
		{
			ColorAttachment::Surface(_) =>
				self.colorAttachment = ColorAttachment::Surface(util::statify(context).surfaceView.as_ref().unwrap()),
			ColorAttachment::Texture(_) => {/* nothing to do */}
		}
	}
}
