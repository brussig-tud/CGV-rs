
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
		HostData, renderer::data::host::Interleaved, renderer::data::host::NonInterleaved,
		renderer::data::host::Indexed, renderer::data::host::CanHaveNormals, renderer::data::host::HasNormals,
		renderer::data::host::CanHaveTangents, renderer::data::host::HasTangents, renderer::data::host::CanHaveRadii,
		renderer::data::host::HasRadii, renderer::data::host::CanHaveRadiusDerivs,
		renderer::data::host::HasRadiusDerivs, renderer::data::host::CanHaveOrientations,
		renderer::data::host::HasOrientations, renderer::data::host::CanHaveScalings, renderer::data::host::HasScalings,
		renderer::data::host::CanHaveColors, renderer::data::host::HasColors
	};
}



//////
//
// Imports
//

// Standard library
use std::ops::{Deref, DerefMut};

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
	fn setData<Data: HostData> (&mut self, data: &Data);

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
	pub fn rebuildForGlobalPasses (&mut self, context: &Context, globalPasses: &[&GlobalPassInfo]) {
		self.gpuStates.clear();
		self.gpuStates.reserve(globalPasses.len());
		for globalPass in globalPasses {
			self.gpuStates.push(self.renderer.createGpuState(context, globalPass.renderState));
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
impl<R: Renderer> Deref for Managed<R>
{
	type Target = R;

	/// Deref to the underlying renderer.
	fn deref (&self) -> &Self::Target {
		&self.renderer
	}
}
impl<R: Renderer> DerefMut for Managed<R>
{
	/// Deref to the underlying renderer.
	fn deref_mut (&mut self) -> &mut Self::Target {
		&mut self.renderer
	}
}
