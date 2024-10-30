
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

/// Indicates the color attachment for a [`RenderState`].
pub enum ColorAttachment<'a>
{
	/// The color attachment is the currently active swapchain image associated with a window surface.
	Surface,

	/// The color attachment is a renderable texture.
	Texture(&'a hal::Texture),

	/// The color attachment is a specific image in the swapchain associated with a surface. This should not be used
	/// directly, rather the *CGV-rs* [`Player`] will substitute any instances of [`ColorAttachment::Surface`] with this
	/// as soon as a swapchain image becomes available automatically. Using this when initializing a [`RenderState`] is
	/// illegal and will panic the render loop at the beginning of a [`GlobalPass`].
	SurfaceView(&'a wgpu::TextureView)
}



//////
//
// Classes
//

////
// ViewingStruct

/// The CPU-side representation of the UniformBuffer used for storing the viewing information.
#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct ViewingStruct
{
	/// The modelview transformation matrix.
	pub view: glm::Mat4,

	/// The projection matrix.
	pub projection: glm::Mat4
}
pub type ViewingUniformGroup = hal::UniformGroup<ViewingStruct>;


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
	pub globalPass: GlobalPass,
	pub viewingUniforms: ViewingUniformGroup,

	pub(crate) colorAttachment: ColorAttachment<'static>,

	depthStencilFormat: Option<hal::DepthStencilFormat>,
	pub(crate) depthStencilAttachment: Option<DepthStencilAttachment>
}

impl RenderState
{
	pub fn new(
		context: &Context, globalPass: GlobalPass, colorAttachment: ColorAttachment<'static>,
		depthStencilFormat: Option<hal::DepthStencilFormat>, name: Option<&str>
	) -> Self
	{
		////
		// Prepare non-inter-referencing fields

		// Uniforms and associated buffers and bind groups
		let viewingUniforms = hal::UniformGroup::create(
			context, wgpu::ShaderStages::VERTEX_FRAGMENT,
			util::concatIfSome(&name, "_viewingUniforms").as_deref()
		);


		////
		// Construct result object

		// Henceforth, we mutate this result object for the remaining initialization
		let mut result = Self {
			globalPass,
			viewingUniforms,

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
		if let Some(format) = self.depthStencilFormat
		{
			// Recreate according to selected main depth/stencil mode
			// - initialize fields that are used by other initializations
			let dims: glm::UVec2 = match self.colorAttachment
			{
				ColorAttachment::Surface/* | ColorAttachment::SurfaceView(_)*/
				=> glm::vec2(context.config.width.max(1), context.config.height.max(1)),

				ColorAttachment::Texture(texture) => texture.dims2WH(),

				_ => unreachable!("Invalid color attachment enum: {:?}", colorAttachmentEnumStr(&self.colorAttachment))
			};
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
				ColorAttachment::SurfaceView(view) => view,
				ColorAttachment::Texture(tex) => &tex.view,
				_ => unreachable!("Invalid color attachment enum: {:?}", colorAttachmentEnumStr(&self.colorAttachment))
			},
			resolve_target: None,
			ops: wgpu::Operations {
				load: wgpu::LoadOp::Load,
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
					load: wgpu::LoadOp::Load,
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

	pub(crate) fn updateSize (&mut self, context: &Context, viewportDims: &glm::UVec2)
	{
		if viewportDims == &self.depthStencilAttachment.as_ref().unwrap().texture.dims2WH() {
			return;
		}
		self.recreateMainSurfaceDepthStencilAttachment(context);
	}
}


////
// RenderStatePrivateInterface

pub(crate) trait RenderStatePrivateInterface {
	fn beginGlobalPass (&mut self, context: &Context);
	fn endGlobalPass (&mut self);
}

impl RenderStatePrivateInterface for RenderState {
	fn beginGlobalPass (&mut self, context: &Context)
	{
		// Validate color attachment state and update for current swapchain image if attached to surface
		match self.colorAttachment {
			ColorAttachment::Surface => self.colorAttachment = ColorAttachment::SurfaceView(
				util::statify(context).surfaceView.as_ref().unwrap()
			),
			ColorAttachment::Texture(_) => {},
			ColorAttachment::SurfaceView(_) => unreachable!(
				"Invalid color attachment kind for starting a global Pass: {:?}", colorAttachmentEnumStr(&self.colorAttachment)
			)
		}
	}

	fn endGlobalPass (&mut self)
	{
		// Validate color attachment state
		match self.colorAttachment {
			ColorAttachment::SurfaceView(_) => self.colorAttachment = ColorAttachment::Surface,
			ColorAttachment::Texture(_) => {/* no action needed */},
			ColorAttachment::Surface => unreachable!(
				"Invalid color attachment kind for starting a global Pass: {:?}", colorAttachmentEnumStr(&self.colorAttachment)
			)
		}
	}
}



//////
//
// Functions
//

fn colorAttachmentEnumStr (colorAttachment: &ColorAttachment) -> &'static str
{
	match colorAttachment {
		ColorAttachment::Surface => "Surface",
		ColorAttachment::Texture(_) => "Texture",
		ColorAttachment::SurfaceView(_) => "SurfaceView"
	}
}
