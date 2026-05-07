
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
		renderer::data::gpu::HasNormals as GpuDataHasNormals,
		renderer::data::gpu::HasTangents as GpuDataHasTangents,
		renderer::data::gpu::HasRadii as GpuDataHasRadii,
		renderer::data::gpu::HasRadiusDerivs as GpuDataHasRadiusDerivs,
		renderer::data::gpu::HasOrientations as GpuDataHasOrientations,
		renderer::data::gpu::HasScalings as GpuDataHasScalings,
		renderer::data::gpu::HasColors as GpuDataHasColors
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
pub trait GpuDataReceiver {
	///
	fn gpuData (&self) -> &dyn GpuData;

	/// Check if the wrapped GPU data is [compatible](data::gpu::BufferLayout::isCompatible) with another.
	#[inline]
	fn isCompatible (&self, otherData: &dyn GpuData) -> bool {
		self.gpuData().layout().isCompatible(&otherData.layout())
	}
}

///
pub trait GpuState {}
impl GpuState for wgpu::RenderPipeline {}

///
pub trait Renderer
{
	///
	type GpuState: GpuState;

	///
	type GpuDataReceiver: GpuDataReceiver;

	/// Returns `true` if this renderer's [`GpuState`] does not depend on the data being rendered. Most implementations
	/// will not require `&self` to answer this query, but we want this to be dynamically dispatchable.
	///
	/// **NOTE:** Implementations that can statically answer this query with `true` are encouraged to implement the
	/// [`renderer::DataIndependent`] trait.
	fn gpuStateIsIndependentFromData(&self) -> bool;

	/// **TODO: this is a placeholder, subject to extensive change as things develop**
	fn createGpuState (&self, context: &Context, renderState: &RenderState, data: &Self::GpuDataReceiver)
		-> Self::GpuState;

	/// **TODO: this is a placeholder, subject to extensive change as things develop**
	fn render (&self, context: &Context, gpuState: &Self::GpuState, data: &Self::GpuDataReceiver);
}

/// Marker trait to enable the [`Managed::setData`] method that statically omits re-creating [`GpuState`].
pub trait DataIndependent {}



//////
//
// Structs
//

///
pub struct Managed<R: Renderer> {
	renderer: R,
	gpuStates: Vec<R::GpuState>,
	data: Option<R::GpuDataReceiver>
}
impl<R: Renderer> Managed<R>
{
	/// Create a new renderer with the given renderer implementation and render state.
	pub fn new (renderer: R) -> Self { Self {
		renderer, gpuStates: Default::default(), data: None
	}}

	/// Helper function to decide whether the `setData...` family of functions need to rebuild [`GpuState`].
	#[inline]
	fn needsRebuild (&self, newData: &R::GpuDataReceiver) -> bool
	{!(
		self.renderer.gpuStateIsIndependentFromData() ||
		if let Some(data) = &self.data {
			data.isCompatible(newData.gpuData())
		} else {
			false
		}
	)}

	/// Rebuild the wrapped renderer's [render state](crate::RenderState)-dependent [`GpuState`] for the given single
	/// render state.
	pub fn rebuildForSingleRenderState (&mut self, context: &Context, renderState: &RenderState) {
		if let Some(data) = &self.data {
			self.gpuStates = vec![self.renderer.createGpuState(context, renderState, data)]
		}
	}

	/// Rebuild the wrapped renderer's [render state](crate::RenderState)-dependent [`GpuState`] as required for the
	/// [`Player`] to perform a full scene pass.
	#[inline(always)]
	pub fn rebuildForPlayer (&mut self, context: &Context, player: &Player) {
		self.rebuildForGlobalPasses(context, player.activeGlobalPasses())
	}

	/// Rebuild the wrapped renderer's [render state](crate::RenderState)-dependent [`GpuState`] for the given list of
	/// [global render passes](GlobalPassInfo).
	pub fn rebuildForGlobalPasses (&mut self, context: &Context, globalPasses: GlobalPasses)
	{
		if let Some(data) = &self.data {
			self.gpuStates.clear();
			self.gpuStates.reserve(globalPasses.renderStates.len());
			for renderState in globalPasses.renderStates {
				self.gpuStates.push(self.renderer.createGpuState(context, renderState, data));
			}
		}
	}

	/// Set new data to render. This requires the wrapped renderer to implement [`renderer::DataIndependent`].
	pub fn setData (&mut self, newData: R::GpuDataReceiver) where R: DataIndependent {
		debug_assert!(self.renderer.gpuStateIsIndependentFromData());
		self.data = Some(newData);
	}

	/// Set new data to render. This potentially triggers a rebuild of the wrapped renderer's [`GpuState`] targeting the
	/// given single [render state](RenderState) if the new data is
	/// [incompatible](data::gpu::BufferLayout::isCompatible).
	///
	/// **NOTE**: The provided render state is assumed to be the same as was provided to the most recent call to
	/// [`Self::rebuildForSingleRenderState`] to allow skipping GPU state recreation for compatible data. Providing a
	/// different render state will not be detected and constitutes a logic bug.
	#[inline]
	pub fn setDataWithSingleRenderState (
		&mut self, context: &Context, renderState: &RenderState, newData: R::GpuDataReceiver
	){
		let rebuild = self.needsRebuild(&newData);
		self.data = Some(newData);
		if rebuild {
			self.rebuildForSingleRenderState(context, renderState);
		}
	}

	/// Set new data to render. This potentially triggers a rebuild of the wrapped renderer's [`GpuState`] as required
	/// for the [`Player`] to perform a full scene pass if the new data is
	/// [incompatible](data::gpu::BufferLayout::isCompatible).
	///
	/// **NOTE**: The provided `Player` reference is assumed to be the same as was provided to the most recent call to
	/// [`Self::rebuildForPlayer`] to allow skipping GPU state recreation for compatible data. Providing a different
	/// `Player` reference will not be detected and constitutes a logic bug. Since there is just one (global) `Player`
	/// instance that applications will typically use, this contract will virtually always be fulfilled in practice.
	#[inline]
	pub fn setDataWithPlayer (&mut self, context: &Context, player: &Player, newData: R::GpuDataReceiver)
	{
		let rebuild = self.needsRebuild(&newData);
		self.data = Some(newData);
		if rebuild {
			self.rebuildForPlayer(context, player);
		}
	}

	/// Set new data to render. This potentially triggers a rebuild of the wrapped renderer's [`GpuState`] for the given
	/// list of [global render passes](GlobalPassInfo) if the new data is
	/// [incompatible](data::gpu::BufferLayout::isCompatible).
	///
	/// **NOTE**: The provided list of global passes is assumed to be the same as was provided to the most recent call
	/// to [`Self::rebuildForGlobalPasses`] to allow skipping GPU state recreation for compatible data. Providing a
	/// different global pass list will not be detected and constitutes a logic bug.
	#[inline]
	pub fn setDataWithGlobalPasses (
		&mut self, context: &Context, globalPasses: GlobalPasses, newData: R::GpuDataReceiver
	){
		let rebuild = self.needsRebuild(&newData);
		self.data = Some(newData);
		if rebuild {
			self.rebuildForGlobalPasses(context, globalPasses);
		}
	}

	/// Dispatch rendering for the very first set of [`GpuState`], typically for use when the managed renderer was
	/// [built for a single render state](Self::rebuildForSingleRenderState).
	#[inline(always)]
	pub fn render (&self, context: &Context) {
		self.renderForGlobalPass(context, 0);
	}

	/// Dispatch rendering for the set of [`GpuState`] associated with the specified [`GlobalPass`].
	pub fn renderForGlobalPass (&self, context: &Context, globalPassIdx: usize)
	{
		assert!(
			globalPassIdx < self.gpuStates.len(), "invalid GPU state index - was the renderer properly initialized?"
		);
		self.renderer.render(context, &self.gpuStates[globalPassIdx], self.data.as_ref().expect(
			"render data should be set before rendering"
		));
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
