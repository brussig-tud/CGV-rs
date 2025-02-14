
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

	pub(crate) framebuffer: hal::Framebuffer,
	colorTargetState: wgpu::ColorTargetState,
	depthStencilState: wgpu::DepthStencilState
}
impl RenderState
{
	/// Create a render state with the given framebuffer.
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
			colorTargetState: defaultColorTargetState(&framebuffer.color0()),
			depthStencilState: defaultDepthStencilState(&framebuffer.depthStencil().unwrap()),
			viewingUniforms, framebuffer
		}
	}

	/// Query the depth/stencil format used in the current managed [render pass](GlobalPassInfo) the render state
	/// belongs to.
	pub fn colorTargetFormat (&self) -> wgpu::TextureFormat {
		self.colorTargetState.format
	}

	/// Reference a depth/stencil state that can be used for rendering to the main framebuffer of the current managed
	/// [render pass](GlobalPassInfo).
	pub fn colorTargetState (&self) -> &wgpu::ColorTargetState {
		&self.colorTargetState
	}

	/// Query the depth/stencil format used in the current managed [render pass](GlobalPassInfo) the render state
	/// belongs to.
	pub fn depthStencilFormat (&self) -> wgpu::TextureFormat {
		self.depthStencilState.format
	}

	/// Query the depth compare function to be used for visibility testing when interfacing with the current managed
	/// [render pass](GlobalPassInfo) the render state belongs to.
	pub fn depthCompareFunction (&self) -> wgpu::CompareFunction {
		self.depthStencilState.depth_compare
	}

	/// Reference a depth/stencil state that can be used for rendering to the main framebuffer of the current managed
	/// [render pass](GlobalPassInfo).
	pub fn depthStencilState (&self) -> &wgpu::DepthStencilState {
		&self.depthStencilState
	}

	pub fn setFramebuffer (&mut self, newFramebuffer: hal::Framebuffer) {
		self.depthStencilState = defaultDepthStencilState(newFramebuffer.depthStencil().unwrap());
		self.framebuffer = newFramebuffer;
	}

	pub fn resizeFramebuffer (&mut self, context: &Context, dims: glm::UVec2) {
		self.framebuffer.resize(context, dims)
	}

	pub fn getMainColorAttachment (&self, clear: Option<&wgpu::Color>) -> Option<wgpu::RenderPassColorAttachment>
	{
		Some(wgpu::RenderPassColorAttachment {
			view: &self.framebuffer.color0().view(),
			resolve_target: None,
			ops: wgpu::Operations {
				load: if let Some(color) = clear {
					wgpu::LoadOp::Clear(*color)
				} else {
					wgpu::LoadOp::Load
				},
				store: wgpu::StoreOp::Store,
			},
		})
	}

	pub fn getMainDepthStencilAttachment (&self, clear: Option<f32>) -> Option<wgpu::RenderPassDepthStencilAttachment>
	{
		self.framebuffer.depthStencil().map(|depthStencilTex| wgpu::RenderPassDepthStencilAttachment {
			view: &depthStencilTex.view(),
			depth_ops: Some(wgpu::Operations {
				load: if let Some(value) = clear {
					wgpu::LoadOp::Clear(value)
				} else {
					wgpu::LoadOp::Load
				},
				store: wgpu::StoreOp::Store,
			}),
			stencil_ops: None,
		})
	}
}



//////
//
// Functions
//

/// Convenience function for creating an opinionated default color target state for a given color texture.
///
/// # Arguments
///
/// * `colorTex` – The color texture to base the state on.
///
/// # Returns
///
/// A color target state with opinionated defaults for the given reference texture.
pub fn defaultColorTargetState (colorTex: &hal::Texture) -> wgpu::ColorTargetState
{
	wgpu::ColorTargetState {
		format: colorTex.descriptor.format,
		blend: if hal::hasAlpha(colorTex.descriptor.format) {
			match colorTex.alphaUsage {
				hal::AlphaUsage::DontCare => None,
				hal::AlphaUsage::Straight => Some(wgpu::BlendState {
					color: wgpu::BlendComponent {
						src_factor: wgpu::BlendFactor::SrcAlpha,
						dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
						operation: wgpu::BlendOperation::Add,
					},
					alpha: wgpu::BlendComponent {
						src_factor: wgpu::BlendFactor::One,
						dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
						operation: wgpu::BlendOperation::Add,
					}
				}),
				hal::AlphaUsage::PreMultiplied => Some(wgpu::BlendState {
					color: wgpu::BlendComponent {
						src_factor: wgpu::BlendFactor::One,
						dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
						operation: wgpu::BlendOperation::Add,
					},
					alpha: wgpu::BlendComponent {
						src_factor: wgpu::BlendFactor::One,
						dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
						operation: wgpu::BlendOperation::Add,
					}
				})
			}
		}
		else { None },
		write_mask: Default::default(),
	}
}

/// Convenience function for creating an opinionated default depth/stencil state for a given depth/stencil texture.
///
/// # Arguments
///
/// * `depthStencilTex` – The depth/stencil texture to base the state on.
///
/// # Returns
///
/// A depth/stencil state with opinionated defaults for the given reference texture.
pub fn defaultDepthStencilState (depthStencilTex: &hal::Texture) -> wgpu::DepthStencilState
{
	wgpu::DepthStencilState {
		format: depthStencilTex.descriptor.format,
		depth_write_enabled: true,
		depth_compare: wgpu::CompareFunction::Less,
		stencil: Default::default(),
		bias: Default::default(),
	}
}
