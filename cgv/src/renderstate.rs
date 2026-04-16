
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
// Constants
//

/// Preset for pre-multiplied alpha blending.
pub const BLEND_ALPHA_PREMULTIPLIED: wgpu::BlendState = wgpu::BlendState {
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
};

/// Preset for straight ("unmultiplied") alpha blending.
pub const BLEND_ALPHA_STRAIGHT: wgpu::BlendState = wgpu::BlendState {
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
};



//////
//
// Enums
//

/// An enum describing a specific alpha blending operation for use with the [`colorTargetStateWithBlending`] convenience
/// function.
pub enum BlendingOperation
{
	/// Alpha blending with pre-multiplied alpha.
	AlphaPreMultiplied,

	/// Alpha Blending with straight alpha ("unmultiplied").
	AlphaStraight,

	/// A user-defined blending operation.
	Custom(wgpu::BlendState)
}
impl BlendingOperation
{
	/// Construct a *WGPU* blend state that corresponds to the selected blenbding operation.
	///
	/// **NOTE**: Since this function cannot know about any additional context, [`BlendingOperation::Alpha`] will just
	/// resolve to [`BLEND_ALPHA_STRAIGHT`].
	pub fn getBlendState (&self) -> wgpu::BlendState {
		match self {
			BlendingOperation::AlphaPreMultiplied => BLEND_ALPHA_PREMULTIPLIED,
			BlendingOperation::AlphaStraight => BLEND_ALPHA_STRAIGHT,
			BlendingOperation::Custom(blend) => *blend
		}
	}
}



//////
//
// Structs
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
	pub fn depthCompareFunction (&self) -> Option<wgpu::CompareFunction> {
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

	pub fn getMainColorAttachment (&self, clear: Option<&wgpu::Color>) -> Option<wgpu::RenderPassColorAttachment<'_>>
	{
		Some(wgpu::RenderPassColorAttachment {
			view: &self.framebuffer.color0().view(),
			resolve_target: None,
			ops: wgpu::Operations {
				load: if let Some(color) = clear { wgpu::LoadOp::Clear(*color) }
				      else                       { wgpu::LoadOp::Load },
				store: wgpu::StoreOp::Store,
			},
			depth_slice: None
		})
	}

	pub fn getMainDepthStencilAttachment (&self, clear: Option<f32>)
		-> Option<wgpu::RenderPassDepthStencilAttachment<'_>>
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

/// Convenience function for creating an opinionated default color target state for a given color target texture.
///
/// # Arguments
///
/// * `colorTarget` – The reference color texture to base the target state on.
///
/// # Returns
///
/// A color target state with opinionated defaults for the reference target texture.
pub fn defaultColorTargetState (colorTarget: &hal::Texture) -> wgpu::ColorTargetState
{
	wgpu::ColorTargetState {
		format: colorTarget.descriptor.format,
		blend: if hal::hasAlpha(colorTarget.descriptor.format) {
			match colorTarget.alphaUsage {
				hal::AlphaUsage::DontCare => None,
				hal::AlphaUsage::Straight => Some(BLEND_ALPHA_STRAIGHT),
				hal::AlphaUsage::PreMultiplied => Some(BLEND_ALPHA_PREMULTIPLIED)
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
pub fn defaultDepthStencilState (depthStencilTex: &hal::Texture) -> wgpu::DepthStencilState {
	wgpu::DepthStencilState {
		format: depthStencilTex.descriptor.format,
		depth_write_enabled: Some(true),
		depth_compare: Some(wgpu::CompareFunction::Less),
		stencil: Default::default(),
		bias: Default::default(),
	}
}

/// Convenience function for creating a copy of the given color target state with the [`wgpu::BlendState`] switched out
/// according to the provided [`BlendingOperation`].
///
/// # Arguments
///
/// * `colorTargetState` – The base `wgpu::ColorTargetState` to manipulate.
/// * `blending` – Description of the blending operation to use. Note that some [`BlendingOperation`]s require that
///                the corresponding color target in a pipeline has an alpha channel. Specifying such a blending
///                operation when no alpha channel exists in the target is a logic bug that will likely only manifest in
///                wrong rendering results.
/// # Returns
///
/// A color target state with the specified blending operation onto the given reference texture.
pub fn changeColorTargetState_blending (colorTargetState: &wgpu::ColorTargetState, blending: BlendingOperation)
	-> wgpu::ColorTargetState
{
	wgpu::ColorTargetState {
		format: colorTargetState.format,
		blend: match blending {
			BlendingOperation::AlphaPreMultiplied                     => Some(BLEND_ALPHA_PREMULTIPLIED),
			BlendingOperation::AlphaStraight                          => Some(BLEND_ALPHA_STRAIGHT),
			BlendingOperation::Custom(customBlend)         => Some(customBlend)
		},
		write_mask: colorTargetState.write_mask
	}
}
