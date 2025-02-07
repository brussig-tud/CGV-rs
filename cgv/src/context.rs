
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

/// The CGV-rs graphics context providing access to all global *WGPU* HAL objects.
pub struct Context {
	adapter: wgpu::Adapter,
	device: wgpu::Device,
	queue: wgpu::Queue
}
impl Context
{
	/// Create a context for the given *egui_wgpu* `RenderState`.
	///
	/// # Arguments
	///
	/// * `eguiRS` – An *egui_wgpu* `RenderState` providing the [`wgpu::Adapter`], [`wgpu::Device`] and
	///              [`wgpu::Queue`] to be used in the context.
	///
	/// # Returns
	///
	/// A new graphics context owning references to the *WGPU* HAL objects the *egui_wgpu* `RenderState` provided.
	pub(crate) fn new (eguiRS: &egui_wgpu::RenderState) -> Self { Self {
		adapter: eguiRS.adapter.clone(), // WGPU HAL objects are internally
		device: eguiRS.device.clone(),   // reference counted, so cloning just
		queue: eguiRS.queue.clone()      // creates a new owned reference
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
