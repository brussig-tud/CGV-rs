
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
///
/// **TODO**: Introduce builder-like API to prevent construction with [invalid](Self::validate) settings.
#[derive(Clone,Copy)]
pub struct InterleavedBufferOptions {
	pub topology: wgpu::PrimitiveTopology,
	pub radiusStorage: ScalarAttributeStorage,
	pub radiusDerivStorage: ScalarAttributeStorage
}
impl InterleavedBufferOptions
{
	/// Checks logical consistency of the selected options. For example, [`radiusStorage`](Self.radiusStorage) and
	/// [`radiusDerivStorage`](Self.radiusDerivStorage) must not request
	/// [co-location](ScalarAttributeStorage::isColocated) at the same host attribute.
	pub fn validate<D: HostData+?Sized> (&self, data: &D) -> bool
	{
		let mut  rHostAttrib: Option<GeometryAttribute> = None;
		let mut rdHostAttrib: Option<GeometryAttribute> = None;

		// Radius is co-located with a non-existing or unsuited other attribute
		if let SAS::InWComponent(hostAttrib) = self.radiusStorage {
			if !data.hasAttrib(hostAttrib) || hostAttrib.isScalar() || hostAttrib.components() > 3 {
				return false;
			}
			rHostAttrib = Some(hostAttrib);
		}

		// Radius derivative is co-located with a non-existing or unsuited other attribute
		if let SAS::InWComponent(hostAttrib) = self.radiusDerivStorage {
			if !data.hasAttrib(hostAttrib) || hostAttrib.isScalar() || hostAttrib.components() > 3 {
				return false;
			}
			rdHostAttrib = Some(hostAttrib);
		}

		// Radius and derivative both try to co-locate with the position
		if self.radiusStorage==SAS::InPosWComponent && self.radiusDerivStorage==SAS::InPosWComponent {
			return false;
		}

		// Radius and derivative are co-located with the same attribute
		if    let Some(rHostAttrib) = rHostAttrib
		   && let Some(rdHostAttrib) = rdHostAttrib && rHostAttrib == rdHostAttrib {
			return false
		}

		// All clear
		true
	}
}
impl Default for InterleavedBufferOptions {
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
impl InterleavedBuffer
{
	/// Create a single-buffer [interleaved](Interleaved) [`GpuData`] from arbitrary host data.
	///
	/// # Arguments
	///
	/// * `context` – The *CGV-rs* context under which to create the GPU resources.
	/// * `data` – The host-side collection of renderable attributes and their values.
	/// * `options` – Additional options to influence how the buffer is laid out (the desired [`GpuData::topology`] can
	///               be set here as well).
	/// * `label` – A string to internally label the GPU-side buffer object with, if desired.
	///
	/// # Returns
	///
	/// An `Arc` owning a fully-built, interleaved `GpuData`-compliant instance of `InterleavedBuffer` containing a
	/// copy of the provided host data.
	///
	/// # Panics
	///
	/// If the `options` contain [invalid](InterleavedBufferOptions::validate) settings combinations.
	pub fn fromHost<D: HostData+?Sized> (
		context: &Context, data: &D, options: InterleavedBufferOptions, label: Option<&str>
	) -> Arc<Self>
	{
		// Sanity check scalar attribute storage
		assert!(options.validate(data));

		// Determine the layout we'll be using
		// - helper function
		fn registerAttrib<T> (layout: &mut BufferLayout, attrib: GA, format: wgpu::VertexFormat) {
			let offset = layout.buffers[0].array_stride;
			let sloc = layout.buffers[0].attributes.len() as u16;
			layout.buffers[0].array_stride += size_of::<T>() as wgpu::BufferAddress;
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
		if data.hasRadii() { match options.radiusStorage
		{
			SAS::InPosWComponent => {
				layout.attribs[GA::Radii.slot()].replace(layout.positions.withNewOffset(3));
			},
			ScalarAttributeStorage::InWComponent(hostAttrib) => {
				let hostAttribLoc = &layout.attribs[hostAttrib.slot()].expect(
					"all attributes suitable for co-hosting a scalar should have been registered already"
				);
				layout.attribs[GA::Radii.slot()].replace(hostAttribLoc.withNewOffset(3));
			},
			SAS::Separate => registerAttrib::<f32>(&mut layout, GA::Radii, GA::Radii.vertexFormat())
		}}
		if data.hasRadiusDerivs() { match options.radiusDerivStorage
		{
			SAS::InPosWComponent => {
				layout.attribs[GA::RadiusDerivs.slot()].replace(layout.positions.withNewOffset(3));
			},
			ScalarAttributeStorage::InWComponent(hostAttrib) => {
				let hostAttribLoc = &layout.attribs[hostAttrib.slot()].expect(
					"all attributes suitable for co-hosting a scalar should have been registered already"
				);
				layout.attribs[GA::RadiusDerivs.slot()].replace(hostAttribLoc.withNewOffset(3));
			},
			SAS::Separate => registerAttrib::<f32>(&mut layout, GA::RadiusDerivs, GA::RadiusDerivs.vertexFormat())
		}}
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
		let size = layout.buffers[0].array_stride  *  data.num() as wgpu::BufferAddress;
		debug_assert_eq!(
			size, hostDataGpuSize(data, options.radiusStorage, options.radiusDerivStorage),
			"buffer size calculation consistency check failed"
		);
		let buffer = context.device().create_buffer(&wgpu::BufferDescriptor {
			label, size, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::STORAGE,
			mapped_at_creation: true,
		});

		// Upload the data
		layout.structuredUpload(data, std::slice::from_mut(&mut buffer.get_mapped_range_mut(..)));
		buffer.unmap();

		// Done!
		Arc::new(Self { num: data.num(), layout, buffer, topology: options.topology })
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
