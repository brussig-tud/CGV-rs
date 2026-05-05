
//////
//
// Imports
//

// Standard library
use std::ops::Deref;

// Local imports
#[expect(unused_imports)] // we only use these for documentation links
use crate::{self as cgv, *};



//////
//
// Traits
//

/// Trait of GPU-side renderable data, ready for drawing by a [`Renderer`].
pub trait Data: Sync+Send
{
	/// Return the number of elements in the underlying data series.
	fn num (&self) -> u32;

	/// Return the [buffer layout](wgpu::api::render_pipeline::VertexState.buffer) of the data inside the GPU.
	fn layout (&self) -> BufferLayout<'_>;

	/// Reference the underlying GPU buffer(s) region(s) containing the renderable data.
	fn geometry (&self) -> Vec<wgpu::BufferSlice<'_>>;

	/// Return the preferred [topology](wgpu::PrimitiveTopology) of the data. Some renderers, like
	/// [`renderer::Spheres`], will completely ignore this, while others like [`renderer::Mesh`] will require specific
	/// topologies.
	fn topology (&self) -> wgpu::PrimitiveTopology;
}

/// Trait indicating that the render data contains index information; that is, indices into the data that connect
/// individual data points to form complex primitives, like lines/line strips, triangles/triangle strips, etc.
pub trait Indexed: Data
{
	/// Return the number of indices over the underlying data series.
	fn numIndices (&self) -> u32;

	/// Reference the underlying GPU buffer region containing the indices.
	fn indices (&self) -> wgpu::BufferSlice<'_>;

	/// Return the [index format](wgpu::IndexFormat) the indices are stored in.
	fn indexFormat (&self) -> wgpu::IndexFormat;
}

/// Marker trait for [`renderer::GpuData`] indicating that the data attributes are stored in an interleaved fashion (aka.
/// "array of structs").
pub trait Interleaved: Data {}

/// Marker trait for [`renderer::GpuData`] indicating that the data attributes are stored in a non-interleaved fashion
/// (aka. "struct of arrays").
pub trait NonInterleaved: Data {}

///
pub trait CanHaveNormals: Data {
	/// Indicate whether normals are available in the data.
	fn hasNormals (&self) -> bool;
}

///
pub trait HasNormals: CanHaveNormals {}

///
pub trait CanHaveTangents: Data {
	/// Indicate whether tangents are available in the data.
	fn hasTangents (&self) -> bool;
}

///
pub trait HasTangents: CanHaveTangents {}

///
pub trait CanHaveRadii: Data {
	/// Indicate whether radii are available in the data.
	fn hasRadii (&self) -> bool;
}

///
pub trait HasRadii: CanHaveRadii {}

///
pub trait CanHaveRadiusDerivs: Data {
	/// Indicate whether radius derivatives are available in the data.
	fn hasRadiusDerivs (&self) -> bool;
}

///
pub trait HasRadiusDerivs: CanHaveRadiusDerivs+HasRadii {}

///
pub trait CanHaveOrientations: Data {
	/// Indicate whether orientations are available in the data.
	fn hasOrientations (&self) -> bool;
}

///
pub trait HasOrientations: CanHaveOrientations {}

///
pub trait CanHaveScalings: Data {
	/// Indicate whether scaling vectors are available in the data.
	fn hasScalings (&self) -> bool;
}

///
pub trait HasScalings: CanHaveScalings {}

///
pub trait CanHaveColors: Data {
	/// Indicate whether colors are available in the data.
	fn hasColors (&self) -> bool;
}

///
pub trait HasColors: CanHaveColors {}



//////
//
// Structs
//

#[derive(Clone)]
pub struct BufferLayout<'this> {
	layout: &'this [wgpu::VertexBufferLayout<'static>]
}
impl BufferLayout<'_>
{
	/// Return a slice of [`wgpu::VertexBufferLayout`]s ready for use in the [vertex state](wgpu::VertexState) of a
	/// [`wgpu::RenderPipelineDescriptor`].
	#[inline(always)]
	pub fn layout (&self) -> &[wgpu::VertexBufferLayout<'static>] {
		&self.layout
	}

	/// Check if another buffer layout is compatible to be used in the same pipeline as this one.
	///
	/// **NOTE: This check does as of yet *not take attribute semantics into account*!**
	///
	/// **TODO: Implement layout attribute semantics and base the check on that.**
	pub fn isCompatible (&self, other: &Self) -> bool {
		   self.layout.len() == other.layout.len()
		&& self.layout.iter().zip(other.layout.iter()).all(|(a,b)| a == b)
	}
}
impl<'this> From<&'this [wgpu::VertexBufferLayout<'static>]> for BufferLayout<'this> {
	#[inline(always)]
	fn from (other: &'this [wgpu::VertexBufferLayout<'static>]) -> Self {
		Self { layout: other }
	}
}
impl PartialEq<Self> for BufferLayout<'_>
{
	fn eq (&self, other: &Self) -> bool
	{
		if self.layout.len() != other.layout.len() {
			return false;
		}
		for (a,b) in self.layout.iter().zip(other.layout.iter())
		{
			if a.array_stride != b.array_stride || a.step_mode != b.step_mode || a.attributes.len() != b.attributes.len() {
				return false;
			}
			for (a,b) in a.attributes.iter().zip(b.attributes.iter()) {
				if a.format != b.format || a.offset != b.offset || a.shader_location != b.shader_location {
					return false;
				}
			}
		}
		true
	}
}
impl Eq for BufferLayout<'_> {}
impl Deref for BufferLayout<'_>
{
	type Target = [wgpu::VertexBufferLayout<'static>];

	#[inline(always)]
	fn deref (&self) -> &Self::Target {
		self.layout
	}
}
