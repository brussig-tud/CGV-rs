
//////
//
// Imports
//

// WGPU API
use crate::wgpu;

// Local imports
use crate::player::*;



//////
//
// Classes
//

/// Collects all rendering setup provided by the *CGV-rs* [`Player`] for applications to use, including everything they
/// need in order to interface with the managed [render passes](GlobalPassInfo) over the scene.
pub struct RenderSetup
{
	// Relevant for pipeline construction
	surfaceFormat: wgpu::TextureFormat,
	defaultColorFormat: wgpu::TextureFormat,
	defaultDepthStencilFormat: wgpu::TextureFormat,
	defaultDepthCompare: wgpu::CompareFunction,
	bindGroupLayouts: ManagedBindGroupLayouts,

	// Relevant for renderpasses 
	pub(crate) defaultClearColor: wgpu::Color,
	pub(crate) defaultDepthClearValue: f32
}
impl RenderSetup
{
	pub(crate) fn new (
		context: &Context, surfaceFormat: wgpu::TextureFormat, defaultColorFormat: wgpu::TextureFormat,
		defaultDepthStencilFormat: hal::DepthStencilFormat, defaultClearColor: wgpu::Color,
		defaultDepthClearValue: f32, defaultDepthCompare: wgpu::CompareFunction
	) -> Self
	{
		Self {
			surfaceFormat, defaultColorFormat, defaultClearColor, defaultDepthCompare, defaultDepthClearValue,
			defaultDepthStencilFormat: defaultDepthStencilFormat.into(),
			bindGroupLayouts: ManagedBindGroupLayouts {
				viewing: renderstate::ViewingUniformGroup::createBindGroupLayout(
					context, wgpu::ShaderStages::VERTEX_FRAGMENT, Some("CGV__ViewingBindGroupLayout")
				)
			},
		}
	}

	/// Query the texture format of the main window surface.
	#[inline(always)]
	pub fn surfaceFormat (&self) -> wgpu::TextureFormat { self.surfaceFormat }

	/// Query the texture format used by default for the color attachment of render targets in managed
	/// [global passes](GlobalPass) if not overridden by a [`Camera`]. The effective value will be contained in the
	/// global passes [`RenderState`].
	#[inline(always)]
	pub fn defaultColorFormat (&self) -> wgpu::TextureFormat { self.defaultColorFormat }

	/// Query the depth/stencil format used by default for the render targets of managed [global passes](GlobalPass) if
	/// not overridden by a [`Camera`]. The effective value will be contained in the global passes [`RenderState`].
	#[inline(always)]
	pub fn defaultDepthStencilFormat (&self) -> wgpu::TextureFormat { self.defaultDepthStencilFormat }

	/// Reference the clear color used by default on the main framebuffer of managed [global passes](GlobalPass) if not
	/// overridden by a [`Camera`]. The effective value will be contained in the global passes [`RenderState`].
	#[inline(always)]
	pub fn defaultClearColor (&self) -> &wgpu::Color { &self.defaultClearColor }

	/// Reference the depth clear value used by default in managed [global passes](GlobalPass) if not overridden by
	/// a [`Camera`]. The effective value will be contained in the global passes [`RenderState`].
	///
	/// [Applications](Application) can expect that the reported [default compare function](Self::defaultDepthCompare)
	/// will be appropriate for the clear value reported here.
	#[inline(always)]
	pub fn defaultDepthClearValue (&self) -> f32 { self.defaultDepthClearValue }

	/// Reference the depth compare function used by default in managed [global passes](GlobalPass) if not overridden by
	/// a [`Camera`]. The effective value will be contained in the global passes [`RenderState`].
	///
	/// [Applications](Application) should take care that the *z* values they emit when drawing to the framebuffer of
	/// a managed `GlobalPass` are ordered accordingly if they want occlusion with geometry drawn by other applications
	/// to work properly. Applications can further expect that the reported
	/// [default clear value](Self::defaultDepthClearValue) will contain a value appropriate for the compare function reported
	/// here.
	#[inline(always)]
	pub fn defaultDepthCompare (&self) -> &wgpu::CompareFunction { &self.defaultDepthCompare }

	/// Reference the bind group layouts provided for interfacing with centrally managed uniforms.
	#[inline(always)]
	pub fn bindGroupLayouts (&self) -> &ManagedBindGroupLayouts { &self.bindGroupLayouts }
}
