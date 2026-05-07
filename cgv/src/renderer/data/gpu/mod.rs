
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Local imports
#[expect(unused_imports)] // we only use these for documentation links
use crate::{self as cgv, *};



//////
//
// Traits
//

/// Trait of GPU-side renderable data, ready for drawing by a [`Renderer`].
pub trait Data: Send+Sync
{
	/// Return the number of elements in the underlying data series.
	fn num (&self) -> u32;

	/// Return the [buffer layout](wgpu::api::render_pipeline::VertexState.buffer) of the data inside the GPU.
	fn layout (&self) -> &BufferLayout;

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
pub trait HasNormals: Data {
	/// Directly reference the exact location of the normals in the overall buffer layout.
	fn normals (&self) -> &BufferAttributeSlot;
}

///
pub trait HasTangents: Data {
	/// Directly reference the exact location of the tangents in the overall buffer layout.
	fn tangents (&self) -> &BufferAttributeSlot;
}

///
pub trait HasRadii: Data {
	/// Directly reference the exact location of the radii in the overall buffer layout.
	fn radii (&self) -> &BufferAttributeSlot;
}

///
pub trait HasRadiusDerivs: Data {
	/// Directly reference the exact location of the radius derivatives in the overall buffer layout.
	fn radiusDerivs (&self) -> &BufferAttributeSlot;
}

///
pub trait HasOrientations: Data {
	/// Directly reference the exact location of the orientations in the overall buffer layout.
	fn orientations (&self) -> &BufferAttributeSlot;
}

///
pub trait HasScalings: Data {
	/// Directly reference the exact location of the scalings in the overall buffer layout.
	fn scalings (&self) -> &BufferAttributeSlot;
}

///
pub trait HasColors: Data {
	/// Directly reference the exact location of the colors in the overall buffer layout.
	fn colors (&self) -> &BufferAttributeSlot;
}



//////
//
// Structs
//

/// Intermediate representation of a [`wgpu::VertexBufferLayout`] missing the
/// [`step_mode`](wgpu::VertexBufferLayout.step_mode) field. The reason for this is that [`Renderer`]s will choose this
/// to be different depending on how they do their rendering.
#[derive(Clone,Copy,PartialEq,Eq)]
pub struct VertexBufferLayoutDesc<'this> {
	/// Proxy for [`wgpu::VertexBufferLayout.array_stride`](wgpu::VertexBufferLayout).
	pub array_stride: wgpu::BufferAddress,

	/// Proxy for [`wgpu::VertexBufferLayout.attributes`](wgpu::VertexBufferLayout).
	pub attributes: &'this [wgpu::VertexAttribute],
}

/// Helper union to maje [`BufferAttributeSlot`] fit into a single `u64`.
#[repr(C)]
#[derive(Copy,Eq)]
union BufferOffsetUnion {
	storage: u16,
	buffer_offset: (u8, u8)
}
impl BufferOffsetUnion
{
	/// Create for the given buffer index and offset.
	#[inline(always)]
	fn new (buffer: u8, offset: u8) -> Self { Self {
		buffer_offset: (buffer, offset)
	}}

	/// Access the buffer index.
	#[inline(always)]
	fn buffer (&self) -> u8 {
		unsafe {
			// SAFETY: Every contiguous 8-bit sequence in a `u16` always constitutes a valid `u8`.
			self.buffer_offset.0
		}
	}

	/// Access the offset.
	#[inline(always)]
	fn offset (&self) -> u8 {
		unsafe {
			// SAFETY: Every contiguous 8-bit sequence in a `u16` always constitutes a valid `u8`.
			self.buffer_offset.1
		}
	}
}
impl Clone for BufferOffsetUnion
{
	#[inline(always)]
	fn clone (&self) -> Self {
		unsafe {
			// SAFETY: There is no valid `u16` that contains an invalid (as `u8`) contiguous 8-bit sequence.
			Self { storage: self.storage }
		}
	}
}
impl PartialEq for BufferOffsetUnion
{
	#[inline(always)]
	fn eq (&self, other: &Self) -> bool {
		unsafe {
			// SAFETY: All possible bit 16-bit sequences form a valid `u16`.
			self.storage == other.storage
		}
	}
}

/// Helper struct used to attach semantics to the opaque *WGPU* entries in a [`BufferLayout`].
#[repr(C)]
#[derive(Clone,Copy,PartialEq,Eq)]
pub struct BufferAttributeSlot
{
	/// Buffer index and offset as a single `u16`-based union. Access via [`Self::buffer()`] and [`Self::offset()`].
	buffer_offset: BufferOffsetUnion,

	/// The attribute slot in the buffer (see [`wgpu::VertexBufferLayout.attributes`](wgpu::VertexBufferLayout)) we're
	/// referring to.
	slot: u16
}
impl BufferAttributeSlot {
	#[inline(always)]
	pub fn new (buffer: u8, slot: u16, offset: u8) -> Self { Self {
		buffer_offset: BufferOffsetUnion::new(buffer, offset), slot
	}}

	/// Get index of the buffer in the layout description (see [`wgpu::BufferLayout.buffers`](BufferLayout) we're
	/// referring to.
	#[inline(always)]
	pub fn buffer (&self) -> usize {
		self.buffer_offset.buffer() as usize
	}

	/// Get the offset within the slot in the buffer (see [`wgpu::VertexAttribute.format`](wgpu::VertexAttribute)), in
	/// multiples of the format's base primitive – i.e. `3` on a slot of format [`Uint8x4`](wgpu::VertexFormat::Uint8x4)
	/// will refer to the 3rd `Uint8` component of the slot (3 bytes from the slot beginning), and on a slot of format
	/// [`Float32x4`](wgpu::VertexFormat::Float32x4) it would refer to the 3rd `Float32` component (12 bytes from the
	/// slot beginning).
	#[inline(always)]
	pub fn offset (&self) -> u8 {
		self.buffer_offset.offset()
	}

	///
	#[inline(always)]
	pub fn slot (&self) -> usize {
		self.slot as usize
	}

	/// Check whether this [`BufferAttributeSlot`] combination refers to the same "physical" slot as another one.
	#[inline]
	pub fn inSameBufferSlot (&self, other: &Self) -> bool {
		self.buffer_offset.buffer() == other.buffer_offset.buffer() && self.slot == other.slot
	}
}

#[derive(Clone)]
pub struct BufferLayout {
	/// A slice of [`wgpu::VertexBufferLayout`]s ready for use in the [vertex state](wgpu::VertexState) of a
	/// [`wgpu::RenderPipelineDescriptor`].
	pub buffers: Vec<VertexBufferLayoutDesc<'static>>,

	/// The exact place of the *position* attributes in the layout.
	pub positions: BufferAttributeSlot,

	/// The exact place, if any, of the *normal* attributes in the layout.
	pub normals: Option<BufferAttributeSlot>,

	/// The exact place, if any, of the *tangent* attributes in the layout.
	pub tangents: Option<BufferAttributeSlot>,

	/// The exact place, if any, of the *radius* attributes in the layout.
	pub radii: Option<BufferAttributeSlot>,

	/// The exact place, if any, of the *radius derivative* attributes in the layout.
	pub radiusDerivs: Option<BufferAttributeSlot>,

	/// The exact place, if any, of the *orientation* attributes in the layout.
	pub orientations: Option<BufferAttributeSlot>,

	/// The exact place, if any, of the *scaling* attributes in the layout.
	pub scalings: Option<BufferAttributeSlot>,

	/// The exact place, if any, of the *color* attributes in the layout.
	pub colors: Option<BufferAttributeSlot>
}
impl BufferLayout
{
	/// Internal helper function for checking if two [`BufferAttributeSlot`]s are compatible.
	#[inline]
	fn checkAttrib (
		buffers: &[VertexBufferLayoutDesc<'static>], attrib: &Option<BufferAttributeSlot>,
		otherBuffers: &[VertexBufferLayoutDesc<'static>], otherAttrib: &Option<BufferAttributeSlot>
	) -> bool {
		match (attrib, otherAttrib) {
			(Some(a), Some(b)) => {
				   buffers[a.buffer()].attributes[a.slot()] == otherBuffers[b.buffer()].attributes[b.slot()]
				&& a.offset() == b.offset()
			},
			(None, None) => true,
			_ => false
		}
	}

	/// Check if another buffer layout is compatible to be used in the same pipeline as this one.
	///
	/// **NOTE**: This is a thorough check that will also properly handle differences that don't actually break
	/// compatibility. While this check is not exactly cheap, it will be faster than having to build a new pipeline.
	pub fn isCompatible (&self, other: &Self) -> bool
	{
		   self.buffers.len() == other.buffers.len()
		&& self.buffers.iter().zip(other.buffers.iter()).all(|(a,b)|
		   	// As long as the stride is the same, we don't care about the actual attribute slots. Their compatibility is
		   	// implicitly captured by the semantic attribute checks below.
		   	a.array_stride == b.array_stride
		   )
		&& self.positions == other.positions
		&& Self::checkAttrib(&self.buffers, &self.normals, &other.buffers, &other.normals)
		&& Self::checkAttrib(&self.buffers, &self.tangents, &other.buffers, &other.tangents)
		&& Self::checkAttrib(&self.buffers, &self.radii, &other.buffers, &other.radii)
		&& Self::checkAttrib(&self.buffers, &self.radiusDerivs, &other.buffers, &other.radiusDerivs)
		&& Self::checkAttrib(&self.buffers, &self.orientations, &other.buffers, &other.orientations)
		&& Self::checkAttrib(&self.buffers, &self.scalings, &other.buffers, &other.scalings)
		&& Self::checkAttrib(&self.buffers, &self.colors, &other.buffers, &other.colors)
	}

	/// **TODO: Remove**
	fn filter (&self, filter: super::GeometryAttributes) -> Self
	{
		// Local helper
		use super::*;
		fn include (
			buffers: &mut std::collections::BTreeSet<usize>, filter: GeometryAttributes, attribBit: GeometryAttributes,
			attrib: &Option<BufferAttributeSlot>
		) -> Option<BufferAttributeSlot>
		{
			if filter.contains(attribBit) && let Some(attrib) = attrib {
				buffers.insert(attrib.buffer());
				let newBufferIdx = buffers.len() - 1;
				Some(BufferAttributeSlot::new(newBufferIdx as u8, attrib.slot() as u16, attrib.offset()))
			} else {
				None
			}
		}

		// Perform the filtering
		let mut buffers = std::collections::BTreeSet::new();
		buffers.insert(self.positions.buffer());
		let normals = include(&mut buffers, filter, GA::NORMALS, &self.normals);
		let tangents = include(&mut buffers, filter, GA::TANGENTS, &self.tangents);
		let radii = include(&mut buffers, filter, GA::TANGENTS, &self.tangents);
		let radiusDerivs = include(&mut buffers, filter, GA::RADIUS_DERIVS, &self.radiusDerivs);
		let orientations = include(&mut buffers, filter, GA::ORIENTATIONS, &self.orientations);
		let scalings = include(&mut buffers, filter, GA::SCALINGS, &self.scalings);
		let colors = include(&mut buffers, filter, GA::COLORS, &self.colors);

		// Done!
		unimplemented!()
	}

	/// Instantiate the [buffer layouts](Self.buffers) with the given [`wgpu::VertexStepMode`], turning them into the
	/// corresponding [`wgpu::VertexBufferLayout`]s for consumption by *WGPU*.
	pub fn withStepMode (&self, step_mode: wgpu::VertexStepMode) -> Vec<wgpu::VertexBufferLayout<'static>> {
		self.buffers.iter().map(|vbl| wgpu::VertexBufferLayout {
			array_stride: vbl.array_stride, step_mode, attributes: vbl.attributes
		}).collect()
	}

	/// Infer whether this layout is interleaved or not.
	pub fn isInterleaved (&self) -> bool {
		self.buffers.len() < 2 && {
			let stride = self.buffers[0].array_stride;
			self.buffers[0].attributes.iter().all(|a| a.offset < stride)
		}
	}
}
impl PartialEq<Self> for BufferLayout
{
	fn eq (&self, other: &Self) -> bool
	{
		// Perform comparison logic with several early termination points
		if self.buffers.len() != other.buffers.len() {
			return false;
		}
		for (a,b) in self.buffers.iter().zip(other.buffers.iter())
		{
			if a.array_stride != b.array_stride || a.attributes.len() != b.attributes.len() {
				return false;
			}
			for (a,b) in a.attributes.iter().zip(b.attributes.iter()) {
				if a.format != b.format || a.offset != b.offset || a.shader_location != b.shader_location {
					return false;
				}
			}
		}

		// If we didn't return yet, the layout is identical
		true
	}
}
impl Eq for BufferLayout {}
