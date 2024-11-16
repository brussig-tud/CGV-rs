
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
// RenderState

pub struct RenderState
{
	pub viewingUniforms: ViewingUniformGroup,

	pub framebuffer: hal::Framebuffer,
	pub defaultDepthStencilState: Option<wgpu::DepthStencilState>
}
impl RenderState
{
	pub fn new (context: &Context, framebuffer: hal::Framebuffer, name: Option<&str>) -> Self
	{
		// Prepare managed uniforms
		// - group 0 - viewing transformations
		let viewingUniforms = hal::UniformGroup::create(
			context, wgpu::ShaderStages::VERTEX_FRAGMENT,
			util::concatIfSome(&name, "_viewingUniforms").as_deref()
		);

		// Done!
		Self {
			defaultDepthStencilState: Self::defaultDepthStencilState(&framebuffer),
			viewingUniforms, framebuffer
		}
	}

	// Helper for extracting default depth/stencil state from a framebuffer
	fn defaultDepthStencilState (framebuffer: &hal::Framebuffer) -> Option<wgpu::DepthStencilState>
	{
		framebuffer.depthStencil().map(|depthStencilTex| {wgpu::DepthStencilState {
			format: depthStencilTex.descriptor.format,
			depth_write_enabled: true,
			depth_compare: wgpu::CompareFunction::Less,
			stencil: Default::default(),
			bias: Default::default(),
		}})
	}

	fn setFramebuffer (&mut self, newFramebuffer: hal::Framebuffer)
	{
		self.defaultDepthStencilState = Self::defaultDepthStencilState(&newFramebuffer);
		self.framebuffer = newFramebuffer;
	}

	fn resizeFramebuffer (&mut self, context: &Context, dims: glm::UVec2) {
		self.framebuffer.resize(context, dims)
	}

	pub fn getMainColorAttachment (&self) -> Option<wgpu::RenderPassColorAttachment>
	{
		Some(wgpu::RenderPassColorAttachment {
			view: &self.framebuffer.color0().view,
			resolve_target: None,
			ops: wgpu::Operations {
				load: wgpu::LoadOp::Load,
				store: wgpu::StoreOp::Store,
			},
		})
	}

	pub fn getMainSurfaceDepthStencilAttachment (&self) -> Option<wgpu::RenderPassDepthStencilAttachment>
	{
		self.framebuffer.depthStencil().map(|depthStencilTex| wgpu::RenderPassDepthStencilAttachment {
			view: &depthStencilTex.view,
			depth_ops: Some(wgpu::Operations {
				load: wgpu::LoadOp::Load,
				store: wgpu::StoreOp::Store,
			}),
			stencil_ops: None,
		})
	}

	pub fn getMainSurfaceDepthStencilState (&self) -> Option<wgpu::DepthStencilState> {
		self.defaultDepthStencilState.clone()
	}
}


////
// RenderStatePrivateInterface

pub(crate) trait RenderStatePrivateInterface {
	fn beginGlobalPass (&mut self, context: &Context);
	fn endGlobalPass (&mut self);
}

impl<'fb> RenderStatePrivateInterface for RenderState {
	fn beginGlobalPass (&mut self, _: &Context) {
		/* nothing to do right now */
	}

	fn endGlobalPass (&mut self) {
		/* nothing to do right now */
	}
}
