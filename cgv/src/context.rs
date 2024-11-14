
//////
//
// Imports
//

// Standard library
use std::sync::Arc;

// WGPU API
use wgpu;

// Local imports
use crate::*;



//////
//
// Structs and enums
//

// Used to pass the *egui_wgpu* setup into the [`Context`] constructor.
pub(crate) struct WgpuSetup<'a> {
	pub adapter: &'a Arc<wgpu::Adapter>,
	pub device: &'a Arc<wgpu::Device>,
	pub queue: &'a Arc<wgpu::Queue>,
	pub surfaceFormat: wgpu::TextureFormat
}



//////
//
// Classes
//

/// Collects all bind group layouts available for interfacing with the managed [render pipeline](wgpu::RenderPipeline)
/// setup of the *CGV-rs* [`Player`].
pub struct ManagedBindGroupLayouts {
	/// The layout of the bind group for the [viewing](ViewingStruct) uniforms.
	pub viewing: wgpu::BindGroupLayout
}

/// The CGV-rs rendering context storing all global graphics state.
pub struct Context
{
	adapter: Arc<wgpu::Adapter>,
	device: Arc<wgpu::Device>,
	queue: Arc<wgpu::Queue>,
	surfaceFormat: wgpu::TextureFormat,
	depthStencilFormat: wgpu::TextureFormat,
	defaultClearColor: wgpu::Color,
	bindGroupLayouts: ManagedBindGroupLayouts
}
impl Context
{
	// Creating some of the wgpu types requires async code
	pub(crate) fn new (wgpuSetup: WgpuSetup) -> Self
	{
		// Create a provisional phony context that only contains the WGPU adapter, device and queue to be able to
		// initialize other members that already need a context for these
		let mut phonyContext = util::Phony::<Context>::new();
		util::forceAssign(&mut phonyContext.adapter, wgpuSetup.adapter);
		util::forceAssign(&mut phonyContext.device, wgpuSetup.device);
		util::forceAssign(&mut phonyContext.queue, wgpuSetup.queue);

		// Actually construct
		Self {
			adapter: wgpuSetup.adapter.clone(),
			device: wgpuSetup.device.clone(),
			queue: wgpuSetup.queue.clone(),
			surfaceFormat: wgpuSetup.surfaceFormat,
			depthStencilFormat: hal::DepthStencilFormat::D32.into(),
			defaultClearColor: wgpu::Color{r: 0.3, g: 0.5, b: 0.7, a: 1.},
			bindGroupLayouts: ManagedBindGroupLayouts {
				viewing: renderstate::ViewingUniformGroup::createBindGroupLayout(
					&phonyContext, wgpu::ShaderStages::VERTEX_FRAGMENT, Some("CGV__ViewingBindGroupLayout")
				)
			}
		}
	}

	/// Reference the *WGPU* instance.
	#[inline(always)]
	pub fn adapter (&self) -> &wgpu::Adapter { &self.adapter }

	/// Reference the *WGPU* device.
	#[inline(always)]
	pub fn device (&self) -> &wgpu::Device { &self.device }

	/// Reference the *WGPU* queue.
	#[inline(always)]
	pub fn queue (&self) -> &wgpu::Queue { &self.queue }

	/// Query the texture format of the main window surafce.
	#[inline(always)]
	pub fn surfaceFormat (&self) -> wgpu::TextureFormat { self.surfaceFormat }

	/// Query the depth/stencil format used for the render targets of managed [global passes](GlobalPass).
	#[inline(always)]
	pub fn depthStencilFormat (&self) -> wgpu::TextureFormat { self.depthStencilFormat }

	/// Reference the clear color that will be used on the main framebuffer in case no [`Application`] requests a
	/// specific one.
	#[inline(always)]
	pub fn defaultClearColor (&self) -> &wgpu::Color { &self.defaultClearColor }

	/// Reference the bind groups provided for interfacing with centrally managed uniforms.
	#[inline(always)]
	pub fn bindGroupLayouts (&self) -> &ManagedBindGroupLayouts { &self.bindGroupLayouts }
}
