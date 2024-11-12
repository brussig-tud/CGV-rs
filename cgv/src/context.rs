
//////
//
// Imports
//

// Standard library
use std::sync::Arc;

// WGPU API
use crate::wgpu;

// Local imports
use crate::hal;

//////
//
// Structs and enums
//

/// Used to pass the *egui_wgpu* setup into the [`Context`] constructor.
pub(crate) struct WgpuSetup<'a> {
	pub adapter: &'a Arc<wgpu::Adapter>,
	pub device: &'a Arc<wgpu::Device>,
	pub queue: &'a Arc<wgpu::Queue>,
}



//////
//
// Classes
//

/// The CGV-rs rendering context storing all global graphics state.
pub struct Context<'a>
{
	adapter: Arc<wgpu::Adapter>,
	device: Arc<wgpu::Device>,
	queue: Arc<wgpu::Queue>,

	mainFramebuffer: hal::Framebuffer<'a>
}

impl<'a> Context<'a>
{
	// Creating some of the wgpu types requires async code
	pub(crate) fn new (wgpuSetup: &WgpuSetup) -> Self
	{
		Self {
			adapter: wgpuSetup.adapter.clone(),
			device: wgpuSetup.device.clone(),
			queue: wgpuSetup.queue.clone(),
			mainFramebuffer: Default::default()
		}
	}

	pub(crate) fn onResize (&mut self, newSize: &glm::UVec2)
	{
		if newSize.x > 0 && newSize.y > 0 {
			//self.size = *newSize;
			tracing::info!("Main framebuffer resized to {:?}", newSize);
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

	/// Retrieve the current dimensions of the main framebuffer.
	pub fn framebufferDims (&self) -> glm::UVec2 {
		self.mainFramebuffer.dims()
	}
}
