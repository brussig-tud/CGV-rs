
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Local imports
use crate::{self as cgv, *};



//////
//
// Traits
//

/// Trait of GPU-side renderable data, ready for drawing by a [`Renderer`].
pub trait Data {
	/// Return the number of elements in the underlying data series.
	fn num (&self) -> u32;

	/// Return the [layout](wgpu::VertexBufferLayout) of the data inside the GPU.
	fn layout (&self) -> wgpu::VertexBufferLayout<'static>;

	/// Reference the underlying GPU buffer containing the renderable data.
	fn geometryBuffer (&self) -> &wgpu::Buffer;

	/// Return the preferred [topology](wgpu::PrimitiveTopology) of the data. Some renderers, like
	/// [`renderer::Spheres`], will completely ignore this, while others like [`renderer::Mesh`] will require specific
	/// topologies.
	fn topology (&self) -> wgpu::PrimitiveTopology;
}

/// Trait indicating that the render data contains index information; that is, indices into the data that connect
/// individual data points to form complex primitives, like lines/line strips, triangles/triangle strips, etc.
pub trait Indexed: Data {
	/// Return the number of indices over the underlying data series.
	fn numIndices (&self) -> u32;

	/// Reference the underlying GPU buffer containing the indices.
	fn indexBuffer (&self) -> &wgpu::Buffer;

	/// Return the [index format](wgpu::IndexFormat) the indices are stored in.
	fn indexFormat (&self) -> wgpu::IndexFormat;
}