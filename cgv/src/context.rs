
//////
//
// Imports
//

// Standard library
use std::hash::{Hash, Hasher};

// Dashmap library
use dashmap::DashMap;

// Egui library and framework
use eframe::egui_wgpu;

// WGPU API
use wgpu;

// Local imports
use crate::*;



//////
//
// Structs
//

/// A newtype'd [`wgpu::SamplerDescriptor`] which we can implement traits for, and which also hides the lifetime'd
/// `label` property since we can't include it for keying samplers in our cache anyway for semantic reasons.
#[derive(Clone,PartialEq)]
struct SamplerDescriptor {
	/// The wrapped *WGPU* sampler descriptor.
	pub inner: wgpu::SamplerDescriptor<'static>
}
impl SamplerDescriptor {
	pub fn fromWgpuSamplerDescriptor(descriptor: &wgpu::SamplerDescriptor) -> Self {
		Self { inner: wgpu::SamplerDescriptor {
			label: Some("CGV__contextManagedSampler"),
			..*descriptor
		}}
	}
}
impl Eq for SamplerDescriptor {}
impl Hash for SamplerDescriptor {
	fn hash<H: Hasher>(&self, state: &mut H) {
		let bytes: &[usize] = util::slicifyInto(self);
		bytes.hash(state);
	}
}



//////
//
// Classes
//

/// The CGV-rs graphics context providing access to all global *WGPU* HAL objects.
pub struct Context {
	adapter: wgpu::Adapter,
	device: wgpu::Device,
	queue: wgpu::Queue,

	samplerCache: DashMap<SamplerDescriptor, wgpu::Sampler>,
	pub(crate) mipmapPipelineCache: gpu::mipmap::PipelineCache,
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
		// Hardware objects
		adapter: eguiRS.adapter.clone(), // WGPU HAL objects are internally
		device: eguiRS.device.clone(),   // reference counted, so cloning just
		queue: eguiRS.queue.clone(),     // creates a new owned reference
		// Caches
		samplerCache: DashMap::with_capacity(8),
		mipmapPipelineCache: DashMap::with_capacity(8),
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

	/// Obtain a reference to a sampler with the given configuration. Note that the *WGPU* `label` property of the
	/// provided `config` is ignored, as that would run counter the principle of having at most one sampler per
	/// functionally distinct configuration.
	pub fn refSampler (&self, config: &wgpu::SamplerDescriptor<'_>) -> wgpu::Sampler
	{
		// Obtain query descriptor
		let queryDesc = SamplerDescriptor::fromWgpuSamplerDescriptor(config);

		// Query cache
		use dashmap::Entry;
		match self.samplerCache.entry(queryDesc) {
			Entry::Occupied(entry) => entry.get().clone(),
			Entry::Vacant(entry) => {
				// We need to create (and cache) a new sampler
				let sampler = self.device.create_sampler(&entry.key().inner);
				entry.insert(sampler).clone()
			}
		}
	}
}
