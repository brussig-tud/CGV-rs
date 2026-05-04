
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
use crate::{self as cgv, *};



//////
//
// Traits
//

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the [`renderer::HostData`]
/// trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHostData<D: cgv::renderer::HostData>() -> bool { true }
/// #[derive(cgv::renderer::InterleavedElem)]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3
///     // ...
/// }
/// assertHostData::<&[MyVertex]>();
/// ```
pub trait InterleavedElem {
	fn pos (&self) -> &glm::Vec3;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasNormals`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasNormals<D: cgv::renderer::data::host::HasNormals>() -> bool { true }
/// #[derive(cgv::renderer::InterleavedElem,cgv::renderer::ElemWithNormal)]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3,
///     #[cgv_renderAttr(normal)]
///     normal: glm::Vec3
///     // ...
/// }
/// assertHasNormals::<&[MyVertex]>();
/// ```
pub trait ElemWithNormal: InterleavedElem {
	fn normal (&self) -> &glm::Vec3;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasTangents`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasTangents<D: cgv::renderer::data::host::HasTangents>() -> bool { true }
/// #[derive(cgv::renderer::InterleavedElem,cgv::renderer::ElemWithTangent)]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3,
///     #[cgv_renderAttr(tangent)]
///     tangent: glm::Vec3
///     // ...
/// }
/// assertHasTangents::<&[MyVertex]>();
/// ```
pub trait ElemWithTangent: InterleavedElem {
	fn tangent (&self) -> &glm::Vec3;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasRadii`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasRadii<D: cgv::renderer::data::host::HasRadii>() -> bool { true }
/// #[derive(cgv::renderer::InterleavedElem,cgv::renderer::ElemWithRadius)]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3,
///     #[cgv_renderAttr(radius)]
///     radius: f32
///     // ...
/// }
/// assertHasRadii::<&[MyVertex]>();
/// ```
pub trait ElemWithRadius: InterleavedElem {
	fn radius (&self) -> &f32;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasRadiusDerivs`] trait on its slices.
///
/// This trait can be derived (note `ElemWithRadiusDeriv`'s requirement to also provide a radius):
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasRadiusDerivs<D: cgv::renderer::data::host::HasRadiusDerivs>() -> bool { true }
/// #[derive(
///     cgv::renderer::InterleavedElem,cgv::renderer::ElemWithRadius,cgv::renderer::ElemWithRadiusDeriv
/// )]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3,
///     #[cgv_renderAttr(radius)]
///     radius: f32,
///     #[cgv_renderAttr(radiusDeriv)]
///     radiusDeriv: f32
///     // ...
/// }
/// assertHasRadiusDerivs::<&[MyVertex]>();
/// ```
pub trait ElemWithRadiusDeriv: InterleavedElem {
	fn radiusDeriv (&self) -> &f32;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasOrientations`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasOrientations<D: cgv::renderer::data::host::HasOrientations>() -> bool { true }
/// #[derive(cgv::renderer::InterleavedElem,cgv::renderer::ElemWithOrientation)]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3,
///     #[cgv_renderAttr(orientation)]
///     orientation: glm::Quat
///     // ...
/// }
/// assertHasOrientations::<&[MyVertex]>();
/// ```
pub trait ElemWithOrientation: InterleavedElem {
	fn orientation (&self) -> &glm::Quat;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasScalings`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasScalings<D: cgv::renderer::data::host::HasScalings>() -> bool { true }
/// #[derive(cgv::renderer::InterleavedElem,cgv::renderer::ElemWithScaling)]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3,
///     #[cgv_renderAttr(scaling)]
///     scaling: glm::Vec3
///     // ...
/// }
/// assertHasScalings::<&[MyVertex]>();
/// ```
pub trait ElemWithScaling: InterleavedElem {
	fn scaling (&self) -> &glm::Vec3;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasColors`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasColors<D: cgv::renderer::data::host::HasColors>() -> bool { true }
/// #[derive(cgv::renderer::InterleavedElem,cgv::renderer::ElemWithColor)]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3,
///     #[cgv_renderAttr(color)]
///     color: cgv::RGBA
///     // ...
/// }
/// assertHasColors::<&[MyVertex]>();
/// ```
pub trait ElemWithColor: InterleavedElem {
	fn color (&self) -> &cgv::RGBA;
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
