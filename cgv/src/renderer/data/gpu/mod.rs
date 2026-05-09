
//////
//
// Module definitions
//

/// Module defining the interleaved reference implementations of [`gpu::Data`](Data).
mod interleaved_buffer;
pub use interleaved_buffer::InterleavedBuffer; // re-export



//////
//
// Imports
//

// Standard library
use std::ops::{Deref, DerefMut};

// Local imports
#[expect(unused_imports)] // we only use these for documentation links
use crate::{self as cgv, *};
use crate::renderer::data::GeometryAttribute;
use super as data;



//////
//
// Enums
//

/// Where to store scalar attributes in [GPU-side render data](Data). Right now, only storing them
/// [separately](ScalarAttributeStorage::Separate) or [in the *w*-component](ScalarAttributeStorage::InWComponent) of a
/// vector-valued attribute is supported to keep the burden of checking dozens of possible  configurations low on
/// [`Renderer`]s.
///
/// **TODO: Add more options once [`BufferLayout`] gains the functionality to auto-generate a Slang "geometry input"
/// implementation for use by renderers that make use of [runtime shader compilation](cgv_shader::compile).**
#[derive(Clone,Copy)]
pub enum ScalarAttributeStorage
{
	/// Store the attribute in the *w*-component of the *positions*.
	InPosWComponent,

	/// Store the attribute in the *w*-component of the provided [`GeometryAttribute`].
	InWComponent(GeometryAttribute),

	/// Store the attribute in its own, separate shader location.
	Separate
}

/// Convenience shorthand for [`ScalarAttributeStorage`].
pub type SAS = ScalarAttributeStorage;



//////
//
// Traits
//

/// Trait of GPU-side renderable data, ready to be [received](renderer::GpuDataReceiver) for drawing by a [`Renderer`].
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
#[derive(Clone,PartialEq,Eq)]
pub struct VertexBufferLayoutDesc {
	/// Proxy for [`wgpu::VertexBufferLayout.array_stride`](wgpu::VertexBufferLayout).
	pub array_stride: wgpu::BufferAddress,

	/// Proxy for [`wgpu::VertexBufferLayout.attributes`](wgpu::VertexBufferLayout).
	pub attributes: Vec<wgpu::VertexAttribute>,
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

	/// Retrieve the buffer index.
	#[inline(always)]
	fn buffer (&self) -> u8 {
		unsafe {
			// SAFETY: Every contiguous 8-bit sequence in a `u16` always constitutes a valid `u8`.
			self.buffer_offset.0
		}
	}

	/// Change the buffer index.
	#[inline(always)]
	fn changeBuffer (&mut self, newBufferIdx: u8) {
		self.buffer_offset.0 = newBufferIdx;
	}

	/// Retrieve the offset.
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

	///
	#[inline(always)]
	pub fn changeBuffer (mut self, newBufferIdx: u8) -> Self {
		self.buffer_offset.changeBuffer(newBufferIdx);
		self
	}
}

///
#[derive(Default,Clone)]
pub struct GeometryAttributeOccupancy([Option<BufferAttributeSlot>; data::GA::NUM_SLOTS as usize]);
impl GeometryAttributeOccupancy {
	///
	pub fn withAttribute (mut self, attribute: data::GeometryAttribute, loc: BufferAttributeSlot) -> Self {
		self.0[attribute as usize].replace(loc);
		self
	}
}
impl From<GeometryAttributeOccupancy> for [Option<BufferAttributeSlot>; data::GA::NUM_SLOTS as usize] {
	#[inline(always)]
	fn from (occupancy: GeometryAttributeOccupancy) -> Self {
		occupancy.0
	}
}
impl Deref for GeometryAttributeOccupancy {
	type Target = [Option<BufferAttributeSlot>; data::GA::NUM_SLOTS as usize];

	#[inline(always)]
	fn deref (&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for GeometryAttributeOccupancy {
	#[inline(always)]
	fn deref_mut (&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// **TODO: Add functionality to auto-generate implementations of the projected `IGeometryInput` interface from the
/// *CGV-rs* core *Slang* shader library for use by renderers that make use of
/// [runtime shader compilation](cgv_shader::compile).**
#[derive(Clone)]
pub struct BufferLayout
{
	/// NOT TRUE ANYMORE: A slice of [`wgpu::VertexBufferLayout`]s ready for use in the
	/// [vertex state](wgpu::VertexState) of a [`wgpu::RenderPipelineDescriptor`].
	pub buffers: Vec<VertexBufferLayoutDesc>,

	/// The exact place of the *position* attributes in the layout.
	pub positions: BufferAttributeSlot,

	/// The exact place, if any, of the *normal* attributes in the layout.
	pub attribs: GeometryAttributeOccupancy
}
impl BufferLayout
{
	/// Internal helper function for checking if two [`BufferAttributeSlot`]s are compatible.
	#[inline]
	fn checkAttrib (
		buffers: &[VertexBufferLayoutDesc], attrib: &Option<BufferAttributeSlot>,
		otherBuffers: &[VertexBufferLayoutDesc], otherAttrib: &Option<BufferAttributeSlot>
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

	/// Check if the given attribute is present
	#[inline(always)]
	pub fn hasAttribute (&self, attribute: data::GeometryAttribute) -> bool {
		self.attribs[attribute as usize].is_some()
	}

	/// Check if the given attribute is present
	#[inline(always)]
	pub fn attribute (&self, attribute: data::GeometryAttribute) -> Option<BufferAttributeSlot> {
		self.attribs[attribute as usize]
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
		&& {
			for attrib in 0..data::GA::NUM_SLOTS as usize
			{
				if !Self::checkAttrib(
					&self.buffers, &self.attribs[attrib], &other.buffers,
					&other.attribs[attrib]
				){
					return false
				}
			}
			true
		}
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

///
pub struct PipelineBufferLayout {
	#[expect(dead_code)] // <- not dead code, but the references are hidden from the compiler inside `wgpuLayouts`
	attribDecls: Vec<Vec<wgpu::VertexAttribute>>,
	wgpuLayouts: Vec<wgpu::VertexBufferLayout<'static>>,
	bufferIndices: Vec<usize>
}
impl PipelineBufferLayout
{
	/// **NOTE:** This currently has worst-case complexity *O*(*N*⋅*M*), *N* being the number of buffers and *M* the
	/// number of filtered attributes, which is probably still the most performant way if we want to **preserve the
	/// order in which the buffers are referenced** in the original, unfiltered layout.
	///
	/// **TODO: It might be possible to include buffers in the vertex state of a pipeline with zero attributes bound to
	/// shader locations. In this case this step could be greatly simplified.**
	pub fn create (
		dataLayout: &BufferLayout, shaderLoc_positions: u32, step_mode: wgpu::VertexStepMode,
		includeAttribs: &[(data::GeometryAttribute, u32)]
	) -> Self
	{
		// Local helper functions
		fn includeAttrib (
			attribLoc_dest: &mut Option<BufferAttributeSlot>, newBufIdx: usize, shaderLoc: u32,
			attribLoc_src: &BufferAttributeSlot, buffer_src: &VertexBufferLayoutDesc
		) -> wgpu::VertexAttribute {
			assert!(attribLoc_dest.is_none(), "must not have multiple instances of an attribute in the layout");
			attribLoc_dest.replace(attribLoc_src.changeBuffer(newBufIdx as u8));
			let mut vertexAttrib = buffer_src.attributes[attribLoc_src.slot()];
			vertexAttrib.shader_location = shaderLoc;
			vertexAttrib
		}
		fn includeShaderAttrib (
			shaderAttribs: &mut Vec<Vec<wgpu::VertexAttribute>>, bufferIdx: usize, attribute: wgpu::VertexAttribute
		){
			assert!(bufferIdx == shaderAttribs.len() || bufferIdx == shaderAttribs.len()-1, "INTERNAL LOGIC ERROR");
			shaderAttribs.resize(bufferIdx+1, Vec::with_capacity(4));
			shaderAttribs.last_mut().unwrap().push(attribute);
		}

		// Build new layout database
		let mut filteredOrigBufIndices: Vec<usize> = Vec::with_capacity(dataLayout.buffers.len());
		let mut filteredAttribDecls: Vec<Vec<wgpu::VertexAttribute>> = Vec::with_capacity(
			filteredOrigBufIndices.capacity()
		);
		let mut positions: Option<BufferAttributeSlot> = None;
		let mut visitedAttribs = GeometryAttributeOccupancy::default();
		for (bufIdx, buffer) in dataLayout.buffers.iter().enumerate()
		{
			// Infer the new index the buffer would get, if it is included later
			let newBufIdx = filteredOrigBufIndices.len();

			// Check if anything we need to include references this buffer
			let mut includeBuffer = false;
			// - the mandatory position attribue
			if dataLayout.positions.buffer() == bufIdx {
				let wgpuVertexAttrib = includeAttrib(
					&mut positions, newBufIdx, shaderLoc_positions, &dataLayout.positions, buffer
				);
				includeShaderAttrib(&mut filteredAttribDecls, newBufIdx, wgpuVertexAttrib);
				includeBuffer = true;
			}
			// - the optional geometry attributes
			for (attrib, shaderLoc) in includeAttribs
			{
				if let Some(attribLoc) = &dataLayout.attribs[attrib.slot()]
				{
					// We only actually include this attribute if it's located in its own slot (implied by an offset=0).
					// Any other offset implies co-location. The knowledge about this will be implicitely reflected in
					// the renderer's shader code.
					//    Co-location necessarily implies a slot in the same buffer, so unless there is a logic bug in
					// the CPU-side code of the renderer filtering it out, the attribute that owns the slot in this
					// buffer will get included one way or another, and we can savely skip the co-located attribute.
					let hasOwnSlot = attribLoc.offset() == 0;
					if attribLoc.buffer() == bufIdx && hasOwnSlot
					{
						let wgpuVertexAttrib = includeAttrib(
							&mut visitedAttribs[attrib.slot()], newBufIdx, *shaderLoc, attribLoc, buffer
						);
						includeShaderAttrib(&mut filteredAttribDecls, newBufIdx, wgpuVertexAttrib);
						includeBuffer = true;
						break;
					}
				}
			}
			// - include if still referenced after filter
			if includeBuffer {
				filteredOrigBufIndices.push(bufIdx);
			}
		}

		// Pre-create the vertex buffer layouts for WGPU consumption
		let wgpuLayouts: Vec<_> = dataLayout.buffers.iter().enumerate().map(
			|(bufIdx, vbl)| wgpu::VertexBufferLayout {
				array_stride: vbl.array_stride, step_mode, attributes: unsafe {
					// SAFETY:
					// The `filteredAttribDecls` vec will not be extended after this point, so the addresses of its
					// elements will remain stable. Also, both the `wgpuLayouts` we create here and the
					// `filteredAttribDecls` they reference will be moved into the (then self-referential)
					// `PipelineBufferLayout` struct we return, so they will have the same lifetime. While we hand out
					// references to `wgpuLayouts`, we will restrict them to the lifetime of our struct (see
					// `Self::bufferLayouts`).
					// Finally, the referenced field `attribDecls` is private and we don't provide any mutable methods
					// that modify it, so Rust's aliasing rules are not violated.
					util::notsafe::extendLifetime(filteredAttribDecls[bufIdx].as_slice())
				}
			}
		).collect();

		// Final sanity checks
		assert_eq!(filteredAttribDecls.len(), wgpuLayouts.len());
		assert_eq!(wgpuLayouts.len(), filteredOrigBufIndices.len());

		// Done!
		Self { attribDecls: filteredAttribDecls, wgpuLayouts, bufferIndices: filteredOrigBufIndices }
	}

	/// Reference the *WGPU* `VertexBufferLayout`s for use in [`wgpu::VertexState`].
	#[inline(always)]
	pub fn bufferLayouts<'this, 'outer> (&'this self) -> &'outer [wgpu::VertexBufferLayout<'this>] {
		self.wgpuLayouts.as_slice()
	}

	/// Get a slice of buffer indices for rendering. They refer to the buffers as declared in the original
	/// [`BufferLayout`] of the backing [`GpuData`](Data) and correspond 1:1 to the slice of pipeline
	/// [`bufferLayouts`](Self::bufferLayouts). [`Renderer`]s need to [set](wgpu::RenderPass::set_vertex_buffer) each
	/// indicated buffer in the order of their appearance within this slice in the [`wgpu::RenderPass`] they use for
	/// drawing.
	#[inline(always)]
	pub fn bufferIndices (&self) -> &[usize] {
		self.bufferIndices.as_slice()
	}
}
