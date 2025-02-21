
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
			address_mode_u: descriptor.address_mode_w,
			address_mode_v: descriptor.address_mode_v,
			address_mode_w: descriptor.address_mode_w,
			mag_filter: descriptor.mag_filter,
			min_filter: descriptor.min_filter,
			mipmap_filter: descriptor.mipmap_filter,
			lod_min_clamp: descriptor.lod_min_clamp,
			lod_max_clamp: descriptor.lod_max_clamp,
			compare: descriptor.compare,
			anisotropy_clamp: descriptor.anisotropy_clamp,
			border_color: descriptor.border_color,
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

	samplerCache: DashMap<
		SamplerDescriptor,
		Box<wgpu::Sampler> // TODO: Try without boxing the `Sampler` once we have sufficiently many to test
	>
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
		samplerCache: DashMap::with_capacity(8)
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

	/// Obtain a reference to a sampler with the given configuration. Note that the `label` property of the provided
	/// *WGPU* sampler descriptor is ignored.
	pub fn obtainSampler<'context> (&self, config: &wgpu::SamplerDescriptor<'_>) -> &'context wgpu::Sampler
	{
		// Obtain query descriptor
		let queryDesc = SamplerDescriptor::fromWgpuSamplerDescriptor(config);

		// Query cache
		let sampler = self.samplerCache.get(&queryDesc);
		if let Some(sampler) = sampler {
			// We already have a sampler with this config
			let sampler = util::notsafe::UncheckedRef::new(sampler.value().as_ref());
			unsafe {
				// Safety: - SAMPLER_CACHE lives in the context which is considered static, so it may report 'static
				//           references
				//         - the values are boxed, so their addresses never change even when iterators are invalidated
				sampler.as_ref()
			}
		}
		else
		{
			// We need to create (and cache) a new sampler
			let sampler = Box::new(self.device.create_sampler(&queryDesc.inner));
			let sampler_unchecked = util::notsafe::UncheckedRef::new(sampler.as_ref());
			self.samplerCache.insert(queryDesc, sampler);
			unsafe {
				// Safety: - the values are boxed, so their addresses never change, even when we move the newly
				//           constructed items into the cache
				//         - SAMPLER_CACHE, which now ownes the box, lives in the context which is considered static, so
				//           we may keep 'static references to its content
				sampler_unchecked.as_ref()
			}
		}
	}
}
