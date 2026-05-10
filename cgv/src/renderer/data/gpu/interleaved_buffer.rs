
//////
//
// Imports
//

// Standard library
use std::sync::Arc;

// Local imports
use crate::{*, renderer::{*, data::{*, gpu::*}}};



//////
//
// Structs
//

/// Additional options to influence how an [`InterleavedBuffer`] stores and layouts its contents.
#[derive(Clone,Copy)]
pub struct InterleavedBufferOptions {
	topology: wgpu::PrimitiveTopology,
	radiusStorage: ScalarAttributeStorage,
	radiusDerivStorage: ScalarAttributeStorage
}
impl InterleavedBufferOptions {
	pub fn validate<D: HostData+?Sized> (&self, data: &D) -> bool
	{
		if self.radiusStorage.isColocated() && self.radiusDerivStorage.isColocated() {
			if self.radiusStorage == self.radiusDerivStorage {
				return false;
			}
		}
		if let SAS::InWComponent(hostAttrib) = self.radiusStorage {
			if !data.hasAttrib(hostAttrib) {
				return false;
			}
		}
		if let SAS::InWComponent(hostAttrib) = self.radiusDerivStorage {
			if !data.hasAttrib(hostAttrib) {
				return false;
			}
		}
		true
	}
}
impl Default for InterleavedBufferOptions
{
	#[inline(always)]
	fn default () -> Self { Self {
		radiusStorage: SAS::InPosWComponent, radiusDerivStorage: SAS::InWComponent(GA::Tangents),
		topology: wgpu::PrimitiveTopology::PointList
	}}
}

/// A reference implementation of [`renderer::GpuData`] that stores all attributes of a given [`renderer::HostData`]
/// inside a single [`wgpu::Buffer`] in an interleaved (array-of-structs) fashion.
pub struct InterleavedBuffer {
	num: u32,
	layout: BufferLayout,
	buffer: wgpu::Buffer,
	topology: wgpu::PrimitiveTopology
}
impl InterleavedBuffer {
	pub fn fromHost<D: HostData+?Sized> (
		context: &Context, data: &D, options: InterleavedBufferOptions, label: Option<&str>
	)// -> Arc<Self>
	{
		// Sanity check scalar attribute storage
		assert!(options.validate(data));

		// Determine the layout we'll be using
		// - helper function
		fn registerAttrib<T> (layout: &mut BufferLayout, attrib: GA, format: wgpu::VertexFormat) {
			let offset = layout.buffers[0].array_stride;
			let sloc = layout.buffers[0].attributes.len() as u16;
			layout.buffers[0].array_stride += size_of::<glm::Vec4>() as wgpu::BufferAddress;
			layout.attribs[attrib.slot()] = Some(BufferAttributeSlot::new(0, sloc, 0));
			layout.buffers[0].attributes.push(wgpu::VertexAttribute {
				format, offset, shader_location: sloc as wgpu::ShaderLocation
			});
		}
		// - create the layout
		let mut layout = BufferLayout::empty(); // <- will pre-create a (0,0,0) location for positions
		layout.buffers.push(VertexBufferLayoutDesc {
			array_stride: size_of::<glm::Vec4>() as wgpu::BufferAddress, // <- to be updated as we add more attributes
			attributes: vec![
				wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 0, shader_location: 0 }
			],
		});
		if data.hasNormals() {
			registerAttrib::<glm::Vec4>(&mut layout, GA::Normals, GA::Normals.vertexFormat());
		}
		if data.hasTangents() {
			registerAttrib::<glm::Vec4>(&mut layout, GA::Tangents, GA::Tangents.vertexFormat());
		}
		if data.hasRadii()
		{
			match options.radiusStorage
			{
				SAS::InPosWComponent => {

				},
				ScalarAttributeStorage::InWComponent(parent) => {

				},
				SAS::Separate => registerAttrib::<f32>(&mut layout, GA::Radii, GA::Radii.vertexFormat())
			};
		}
		if data.hasOrientations() {
			registerAttrib::<glm::Vec4>(&mut layout, GA::Orientations, GA::Orientations.vertexFormat());
		}
		if data.hasScalings() {
			registerAttrib::<glm::Vec4>(&mut layout, GA::Scalings, GA::Scalings.vertexFormat());
		}
		if data.hasColors() {
			registerAttrib::<glm::Vec4>(&mut layout, GA::Colors, GA::Colors.vertexFormat());
		}

		// Create buffer
		let size = hostDataGpuSize(data, options.radiusStorage, options.radiusDerivStorage);
		let buffer = context.device().create_buffer(&wgpu::BufferDescriptor {
			label, size, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::STORAGE,
			mapped_at_creation: true,
		});
	}
}
impl GpuData for InterleavedBuffer
{
	fn num (&self) -> u32 {
		self.num
	}

	fn layout (&self) -> &BufferLayout {
		&self.layout
	}

	fn geometry (&self) -> Vec<wgpu::BufferSlice<'_>> {
		vec![self.buffer.slice(..)]
	}

	fn topology (&self) -> wgpu::PrimitiveTopology {
		self.topology
	}
}
impl Interleaved for InterleavedBuffer {}
