
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Egui library and framework
use eframe::egui_wgpu;

// WGPU API
use crate::wgpu;



//////
//
// Classes
//

/// The CGV-rs rendering context storing all global graphics state.
pub struct Context {
	adapter: wgpu::Adapter,
	device: wgpu::Device,
	queue: wgpu::Queue
}
impl Context
{
	// Creating some of the wgpu types requires async code
	pub(crate) fn new (eguiRS: &egui_wgpu::RenderState) -> Self { Self {
		adapter: eguiRS.adapter.clone(), // WGPU HAL objects are internally
		device: eguiRS.device.clone(),   // reference counted, so cloning just
		queue: eguiRS.queue.clone()      // creates and owns a new reference
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
