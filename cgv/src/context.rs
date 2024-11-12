
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
	instance: &'a Arc<wgpu::Instance>,
	adapter: &'a Arc<wgpu::Adapter>,
	device: &'a Arc<wgpu::Device>,
	queue: &'a Arc<wgpu::Queue>,
}



//////
//
// Classes
//

/// The CGV-rs rendering context storing all global graphics state.
pub struct Context<'a>
{
	instance: Arc<wgpu::Instance>,
	adapter: Arc<wgpu::Adapter>,
	device: Arc<wgpu::Device>,
	queue: Arc<wgpu::Queue>,

	size: glm::UVec2,

	mainFramebuffer: hal::Framebuffer<'a>
}

impl<'a> Context<'a>
{
	// Creating some of the wgpu types requires async code
	pub(crate) fn new (wgpuSetup: &WgpuSetup) -> Self
	{
		Self {
			instance: wgpuSetup.instance.clone(),
			adapter: wgpuSetup.adapter.clone(),
			device: wgpuSetup.device.clone(),
			queue: wgpuSetup.queue.clone(),
			size: glm::vec2(0, 0),
			mainFramebuffer: Default::default()
		}
	}

	/// Reference the *WGPU* instance.
	#[inline(always)]
	pub fn instance (&self) -> &wgpu::Instance { &self.instance }

	/// Reference the *WGPU* instance.
	#[inline(always)]
	pub fn adapter (&self) -> &wgpu::Adapter { &self.adapter }

	/// Reference the *WGPU* device.
	#[inline(always)]
	pub fn device (&self) -> &wgpu::Device { &self.device }

	/// Reference the *WGPU* queue.
	#[inline(always)]
	pub fn queue (&self) -> &wgpu::Queue { &self.queue }

	pub fn resize (&mut self, newSize: &glm::UVec2)
	{
		if newSize.x > 0 && newSize.y > 0 {
			tracing::info!("Resizing to {:?}", newSize);
			self.size = *newSize;
		}
	}
}
