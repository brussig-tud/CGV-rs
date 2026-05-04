
//////
//
// Module definitions
//

/// Module implementing a renderer for large amount of spheres.
pub mod spheres;
pub use spheres::Spheres; // re-export

/// Module defining the render data model.
pub mod data;
pub use data::{host::Data as HostData, gpu::Data as GpuData}; // re-export

/// The module prelude.
pub mod prelude {
	pub use super::{
		// "The" renderer trait
		Renderer,

		// Host-side data traits
		HostData, renderer::data::host::Interleaved as HostDataInterleaved,
		renderer::data::host::NonInterleaved as HostDataNonInterleaved,
		renderer::data::host::Indexed as HostDataIndexed,
		renderer::data::host::CanHaveNormals as HostDataCanHaveNormals,
		renderer::data::host::HasNormals as HostDataHasNormals,
		renderer::data::host::CanHaveTangents as HostDataCanHaveTangents,
		renderer::data::host::HasTangents as HostDataHasTangents,
		renderer::data::host::CanHaveRadii as HostDataCanHaveRadii,
		renderer::data::host::HasRadii as HostDataHasRadii,
		renderer::data::host::CanHaveRadiusDerivs as HostDataCanHaveRadiusDerivs,
		renderer::data::host::HasRadiusDerivs as HostDataHasRadiusDerivs,
		renderer::data::host::CanHaveOrientations as HostDataCanHaveOrientations,
		renderer::data::host::HasOrientations as HostDataHasOrientations,
		renderer::data::host::CanHaveScalings as HostDataCanHaveScalings,
		renderer::data::host::HasScalings as HostDataHasScalings,
		renderer::data::host::CanHaveColors as HostDataCanHaveColors,
		renderer::data::host::HasColors as HostDataHasColors,

		// GPU-side data traits
		GpuData, renderer::data::gpu::Interleaved as GpuDataInterleaved,
		renderer::data::gpu::NonInterleaved as GpuDataNonInterleaved,
		renderer::data::gpu::Indexed as GpuDataIndexed,
		renderer::data::gpu::CanHaveNormals as GpuDataCanHaveNormals,
		renderer::data::gpu::HasNormals as GpuDataHasNormals,
		renderer::data::gpu::CanHaveTangents as GpuDataCanHaveTangents,
		renderer::data::gpu::HasTangents as GpuDataHasTangents,
		renderer::data::gpu::CanHaveRadii as GpuDataCanHaveRadii,
		renderer::data::gpu::HasRadii as GpuDataHasRadii,
		renderer::data::gpu::CanHaveRadiusDerivs as GpuDataCanHaveRadiusDerivs,
		renderer::data::gpu::HasRadiusDerivs as GpuDataHasRadiusDerivs,
		renderer::data::gpu::CanHaveOrientations as GpuDataCanHaveOrientations,
		renderer::data::gpu::HasOrientations as GpuDataHasOrientations,
		renderer::data::gpu::CanHaveScalings as GpuDataCanHaveScalings,
		renderer::data::gpu::HasScalings as GpuDataHasScalings,
		renderer::data::gpu::CanHaveColors as GpuDataCanHaveColors,
		renderer::data::gpu::HasColors as GpuDataHasColors
	};
}



//////
//
// Imports
//

// Standard library
use std::{ops::{Deref, DerefMut}, sync::Arc};

// Local imports
use crate::*;



//////
//
// Traits
//

///
pub trait GpuState {}
impl GpuState for wgpu::RenderPipeline {}

///
pub trait Renderer
{
	///
	type GpuState: GpuState;

	///
	#[expect(unused_variables)] // <- we want `data` to show up in the documented signature
	fn setData (&mut self, data: Arc<dyn GpuData>) {
		unimplemented!("renderer implementations must specifically opt-in to polymorphic render data assignment")
	}

	///
	fn createGpuState (&self, context: &Context, renderState: &RenderState) -> Self::GpuState;

	/// **TODO: this is a placeholder, subject to extensive change as things develop**
	fn render (&self, context: &Context, gpuObjects: &Self::GpuState);
}



//////
//
// Structs
//

///
pub struct Managed<R: Renderer> {
	renderer: R,
	gpuStates: Vec<R::GpuState>
}
impl<R: Renderer> Managed<R>
{
	/// Create a new renderer with the given renderer implementation and render state.
	pub fn new (renderer: R) -> Self { Self {
		renderer, gpuStates: Default::default()
	}}

	/// Rebuild the wrapped renderer's [`RenderState`](crate::RenderState)-dependent [`GpuObjects`](GpuState) for the
	/// given single renderState.
	pub fn rebuildForSingleRenderState (&mut self, context: &Context, renderState: &RenderState) {
		self.gpuStates = vec![self.renderer.createGpuState(context, renderState)];
	}

	/// Rebuild the wrapped renderer's [`RenderState`](crate::RenderState)-dependent [`GpuObjects`](GpuState) for the
	/// list of [managed global render passes](GlobalPassInfo).
	pub fn rebuildForGlobalPasses (&mut self, context: &Context, globalPasses: &GlobalPasses) {
		self.gpuStates.clear();
		self.gpuStates.reserve(globalPasses.renderStates.len());
		for renderState in globalPasses.renderStates {
			self.gpuStates.push(self.renderer.createGpuState(context, renderState));
		}
	}

	/// Dispatch rendering for the very first set of [`GpuState`], typically for use when the managed renderer was
	/// [built for a single render state](Self::rebuildForSingleRenderState).
	pub fn render (&self, context: &Context) {
		self.renderer.render(context, &self.gpuStates[0]);
	}

	/// Dispatch rendering for the set of [`GpuState`] associated with the specified [`GlobalPass`].
	pub fn renderForGlobalPass (&self, context: &Context, globalPassIdx: usize) {
		self.renderer.render(context, &self.gpuStates[globalPassIdx]);
	}
}
impl<R: Renderer> Deref for Managed<R> {
	type Target = R;
	fn deref (&self) -> &Self::Target {
		&self.renderer
	}
}
impl<R: Renderer> DerefMut for Managed<R>{
	fn deref_mut (&mut self) -> &mut Self::Target {
		&mut self.renderer
	}
}
