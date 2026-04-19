
//////
//
// Module definitions
//

/// Module implementing a renderer for large amount of spheres.
pub mod spheres;
pub use spheres::Spheres; // re-export

/// Module defining the render data model.
pub mod data;
pub use data::Data; // re-export

/// The module prelude.
pub mod prelude {
	pub use super::{
		Data, renderer::data::Interleaved, renderer::data::NonInterleaved, renderer::data::Indexed,
		renderer::data::CanHaveNormals, renderer::data::HasNormals, renderer::data::CanHaveTangents,
		renderer::data::HasTangents, renderer::data::CanHaveRadii, renderer::data::HasRadii,
		renderer::data::CanHaveRadiusDerivs, renderer::data::HasRadiusDerivs, renderer::data::CanHaveOrientations,
		renderer::data::HasOrientations, renderer::data::CanHaveScalings, renderer::data::HasScalings,
		renderer::data::CanHaveColors, renderer::data::HasColors
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
pub trait GpuObjects {}

///
pub trait Renderer
{
	///
	type GpuObjects: GpuObjects;

	///
	fn createGpuObjects (&self, context: &Context, renderState: &RenderState)
		-> Self::GpuObjects;
}



//////
//
// Structs
//

///
pub struct Helper<R: Renderer> {
	renderer: R,
	gpuObjects: Vec<R::GpuObjects>
}
impl<R: Renderer> Helper<R>
{
	/// Create a new renderer with the given renderer implementation and render state.
	pub fn new (renderer: R) -> Self { Self {
		renderer, gpuObjects: Default::default()
	}}

	/// Rebuild the wrapped renderer's [`RenderState`](crate::RenderState)-dependent [`GpuObjects`](GpuObjects) for the
	/// given single renderState.
	pub fn rebuildForSingleRenderState (&mut self, context: &Context, renderState: &RenderState) {
		self.gpuObjects = vec![self.renderer.createGpuObjects(context, renderState)];
	}

	/// Rebuild the wrapped renderer's [`RenderState`](crate::RenderState)-dependent [`GpuObjects`](GpuObjects) for the
	/// list of [managed global render passes](GlobalPassInfo).
	pub fn rebuildForGlobalPasses (&mut self, context: &Context, globalPasses: &[&GlobalPassInfo]) {
		self.gpuObjects.clear();
		self.gpuObjects.reserve(globalPasses.len());
		for globalPass in globalPasses {
			self.gpuObjects.push(self.renderer.createGpuObjects(context, globalPass.renderState));
		}
	}
}
impl<R: Renderer> Deref for Helper<R>
{
	type Target = R;

	/// Deref to the underlying renderer.
	fn deref (&self) -> &Self::Target {
		&self.renderer
	}
}
impl<R: Renderer> DerefMut for Helper<R>
{
	/// Deref to the underlying renderer.
	fn deref_mut (&mut self) -> &mut Self::Target {
		&mut self.renderer
	}
}
