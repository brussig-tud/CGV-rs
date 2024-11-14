
//////
//
// Imports
//

// Standard library
use std::sync::Arc;

// WGPU API
use crate::wgpu;



//////
//
// Structs and enums
//

// Used to pass the *egui_wgpu* setup into the [`Context`] constructor.
pub(crate) struct WgpuSetup<'a> {
	pub adapter: &'a Arc<wgpu::Adapter>,
	pub device: &'a Arc<wgpu::Device>,
	pub queue: &'a Arc<wgpu::Queue>
}



//////
//
// Classes
//

/// The CGV-rs rendering context storing all global graphics state.
pub struct Context {
	adapter: Arc<wgpu::Adapter>,
	device: Arc<wgpu::Device>,
	queue: Arc<wgpu::Queue>
}
impl Context
{
	// Creating some of the wgpu types requires async code
	pub(crate) fn new (wgpuSetup: &WgpuSetup) -> Self { Self {
		adapter: wgpuSetup.adapter.clone(),
		device: wgpuSetup.device.clone(),
		queue: wgpuSetup.queue.clone()
	}}

	/// Reference the *WGPU* instance.
	#[inline(always)]
	pub fn adapter (&self) -> &wgpu::Adapter { &self.adapter }

	/// Reference the *WGPU* device.
	#[inline(always)]
	pub fn device (&self) -> &wgpu::Device { &self.device }

	/// Reference the *WGPU* queue.
	#[inline(always)]
	pub fn queue (&self) -> &wgpu::Queue { &self.queue }
}
