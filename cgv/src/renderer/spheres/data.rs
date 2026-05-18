
//////
//
// Imports
//

// Local imports
use crate::{self as cgv, renderer::{data::{gpu, host}, spheres::*}};



//////
//
// Enums
//

/// Possible [buffer layouts](GpuData::layout) of a [`spheres::GpuData`] instance.
///
/// **TODO: This seems like a good candidate to extend, generalize and make available as a utility for all renderers.**
enum LayoutVariant {
	PosOnly, PosRadius, PosColor, PosRadiusColor
}
impl LayoutVariant {
	const RADIUS_SLOT: u16 = 0;  // <- always in the 0-th slot with the position
	const RADIUS_OFFSET: u8 = 3; // <- radius starts after the 3 position components

	const COLOR_SLOT: u16 = 1;   // <- always in the 1st slot, after position
	const COLOR_OFFSET: u8 = 0;  // <- no offset, color uses the whole slot exclusively

	const POS_ONLY: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0=>Float32x4]; // 4-th component is unused
	const POS_ONLY_STRIDE: wgpu::BufferAddress = size_of::<glm::Vec4>() as wgpu::BufferAddress;

	const POS_RADIUS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0=>Float32x4]; // 4-th component is radius
	const POS_RADIUS_STRIDE: wgpu::BufferAddress = size_of::<glm::Vec4>() as wgpu::BufferAddress;

	const POS_COLOR: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0=>Float32x4, 1=>Float32x4];
	const POS_COLOR_STRIDE: wgpu::BufferAddress = (size_of::<glm::Vec4>()*2) as wgpu::BufferAddress;

	const POS_RADIUS_COLOR: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0=>Float32x4, 1=>Float32x4];
	const POS_RADIUS_COLOR_STRIDE: wgpu::BufferAddress = (size_of::<glm::Vec4>()*2) as wgpu::BufferAddress;

	/// Construct a buffer layout that fits the variant represented by `self`.
	pub fn layout (&self) -> gpu::BufferLayout
	{
		let positions = gpu::BufferAttributeSlot::new(0, 0, 0);
		match self
		{
			Self::PosOnly => gpu::BufferLayout {
				buffers: vec![gpu::VertexBufferLayoutDesc {
					array_stride: Self::POS_ONLY_STRIDE,
					attributes: Vec::from(Self::POS_ONLY)
				}], positions,
				attribs: Default::default(),
			},
			Self::PosRadius => gpu::BufferLayout {
				buffers: vec![gpu::VertexBufferLayoutDesc {
					array_stride: Self::POS_RADIUS_STRIDE,
					attributes: Vec::from(Self::POS_RADIUS),
				}], positions,
				attribs: gpu::GeometryAttributeOccupancy::default().withAttribute(
					GA::Radii, gpu::BufferAttributeSlot::new(0, Self::RADIUS_SLOT, Self::RADIUS_OFFSET)
				)
			},
			Self::PosColor => gpu::BufferLayout {
				buffers: vec![gpu::VertexBufferLayoutDesc {
					array_stride: Self::POS_COLOR_STRIDE,
					attributes: Vec::from(Self::POS_COLOR),
				}], positions,
				attribs: gpu::GeometryAttributeOccupancy::default().withAttribute(
					GA::Colors, gpu::BufferAttributeSlot::new(0, Self::COLOR_SLOT, Self::COLOR_OFFSET)
				)
			},
			Self::PosRadiusColor => gpu::BufferLayout {
				buffers: vec![gpu::VertexBufferLayoutDesc {
					array_stride: Self::POS_RADIUS_COLOR_STRIDE,
					attributes: Vec::from(Self::POS_RADIUS_COLOR),
				}], positions,
				attribs: gpu::GeometryAttributeOccupancy::default().withAttribute(
					GA::Radii, gpu::BufferAttributeSlot::new(0, Self::RADIUS_SLOT, Self::RADIUS_OFFSET)
				).withAttribute(
					GA::Colors, gpu::BufferAttributeSlot::new(0, Self::COLOR_SLOT, Self::COLOR_OFFSET)
				)
			}
		}
	}

	/// Construct an attribute buffer according to the needs of the variant represented by `self`. The buffer will be
	/// [mapped at creation](wgpu::BufferDescriptor.mapped_at_creation).
	pub fn createBuffer (&self, context: &Context, numInstances: u32, label: Option<&str>) -> wgpu::Buffer
	{
		fn createBuffer<A: Sized> (dev: &wgpu::Device, numInstances: u32, label: Option<&str>) -> wgpu::Buffer {
			dev.create_buffer(&wgpu::BufferDescriptor {
				label, size: (numInstances as usize * size_of::<A>()) as wgpu::BufferAddress,
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::STORAGE,
				mapped_at_creation: true,
			})
		}
		match self {
			Self::PosOnly => createBuffer::<glm::Vec4>(context.device(), numInstances, label),
			Self::PosRadius => createBuffer::<glm::Vec4>(context.device(), numInstances, label),
			Self::PosColor => createBuffer::<(glm::Vec4, cgv::RGBA)>(context.device(), numInstances, label),
			Self::PosRadiusColor => createBuffer::<(glm::Vec4, cgv::RGBA)>(context.device(), numInstances, label),
		}
	}
}



//////
//
// Structs
//

/// Stores the default attributes that the [`Spheres`](renderer::Spheres) will use when rendering spheres when the
/// corresponding attributes are not sourced from user data.
#[repr(C,align(16))]
#[derive(Clone, Copy, bytemuck::NoUninit)]
pub struct DefaultAttributes {
	/// The default color of the rendered spheres, used when the color attribute is not sourced from user data.
	pub color: Rgba,

	/// The default radius of the rendered spheres, used when the radius attribute is not sourced from user data.
	pub radius: f32,
	pub pad: [u32; 3],
}
impl Default for DefaultAttributes {
	fn default () -> Self { Self {
		radius: 1.0, color: Rgba::from_rgb(2./5., 2./5., 2./5.), pad: [0; 3]
	}}
}
pub type DefaultAttribsUniformGroup = hal::UniformGroup<DefaultAttributes>;

/// A [`renderer::GpuData`]-compliant interleaved storage optimized for use with the [spheres renderer](Spheres).
pub struct GpuData {
	num: u32,
	layout: gpu::BufferLayout,
	attributes: wgpu::Buffer
}
impl GpuData
{
	/// Helper function for common initialization.
	fn commonInit<D: HostData+?Sized, T> (context: &Context, variant: LayoutVariant, data: &D, label: Option<&str>)
		-> (gpu::BufferLayout, wgpu::Buffer, std::ptr::NonNull<T>, Option<Vec<u8>>)
	{
		// Under WASM, we have a super weird buffer mapping problem that seems related to the one we encountered for
		// `InterleavedBuffer`, although it manifests in randomly corrupted heap memory. To work around this, we compile
		// the attributes in a staging area first and let WGPU handle the upload.
		#[cfg(target_arch="wasm32")]
			let mut stagingMem = Some(vec![0; data.num() as usize * size_of::<T>()]);
		#[cfg(not(target_arch="wasm32"))]
			let stagingMem: Option<Vec<u8>> = None;

		// Prepare the buffer
		let attributes = variant.createBuffer(context, data.num(), label);

		// Gain mapped pointer for uploading
		let ptr;
		#[cfg(target_arch="wasm32")] {
			ptr = std::ptr::NonNull::new(stagingMem.as_mut().unwrap().as_mut_ptr()).unwrap().cast::<T>();
		}
		#[cfg(not(target_arch="wasm32"))] {
			ptr = attributes.get_mapped_range_mut(..).slice(..).as_raw_ptr().cast::<T>();
		}

		// Done!
		(variant.layout(), attributes, ptr, stagingMem)
	}


	///
	pub fn new<D: HostData+?Sized> (context: &Context, data: &D, label: Option<&str>) -> Arc<Self>
	{
		// Common initialization
		let (layout, attributes, mut ptr, _stagingMem)
			= Self::commonInit(context, LayoutVariant::PosOnly, data, label);

		// Upload the data
		for ref pos in data.positions() {
			unsafe {
				// SAFETY: What could possibly go wrong? It'll be fine.
				ptr.write(/* pos_rad: */glm::vec3_to_vec4(pos));
				ptr = ptr.add(1);
			}
		}
		#[cfg(target_arch="wasm32")] {
			attributes.get_mapped_range_mut(..).copy_from_slice(&_stagingMem.unwrap());
		}
		attributes.unmap(); // <- make uploaded data visible to GPU

		// Done!
		Arc::new(Self { num: data.num(), layout, attributes })
	}

	///
	pub fn withRadii<D: HostData+host::HasRadii+?Sized> (context: &Context, data: &D, label: Option<&str>) -> Arc<Self>
	{
		// Common initialization
		let (layout, attributes, mut ptr, _stagingMem)
			= Self::commonInit(context, LayoutVariant::PosRadius, data, label);

		// Upload the data
		for (pos, radius) in data.positions().zip(data.radii()) {
			unsafe {
				// SAFETY: What could possibly go wrong? It'll be fine.
				ptr.write(/* pos_rad: */glm::vec4(pos.x, pos.y, pos.z, radius));
				ptr = ptr.add(1);
			}
		}
		#[cfg(target_arch="wasm32")] {
			attributes.get_mapped_range_mut(..).copy_from_slice(&_stagingMem.unwrap());
		}
		attributes.unmap(); // <- make uploaded data visible to GPU

		// Done!
		Arc::new(Self { num: data.num(), layout, attributes })
	}

	///
	pub fn withColors<D: HostData+host::HasColors+?Sized> (context: &Context, data: &D, label: Option<&str>)
		-> Arc<Self>
	{
		// Common initialization
		let (layout, attributes, mut ptr, _stagingMem)
			= Self::commonInit(context, LayoutVariant::PosColor, data, label);

		// Upload the data
		for (ref pos, color) in data.positions().zip(data.colors()) {
			unsafe {
				// SAFETY: What could possibly go wrong? It'll be fine.
				ptr.write((/* pos_rad: */glm::vec3_to_vec4(pos), /* color: */color));
				ptr = ptr.add(1);
			}
		}
		#[cfg(target_arch="wasm32")] {
			attributes.get_mapped_range_mut(..).copy_from_slice(&_stagingMem.unwrap());
		}
		attributes.unmap(); // <- make uploaded data visible to GPU

		// Done!
		Arc::new(Self { num: data.num(), layout, attributes })
	}

	///
	pub fn withRadiiAndColors<D: HostData+host::HasRadii+host::HasColors+?Sized> (
		context: &Context, data: &D, label: Option<&str>
	) -> Arc<Self> {
		// Common initialization
		let (layout, attributes, mut ptr, _stagingMem)
			= Self::commonInit(context, LayoutVariant::PosRadiusColor, data, label);

		// Upload the data
		for ((pos, radius), color) in data.positions().zip(data.radii()).zip(data.colors()) {
			unsafe {
				// SAFETY: What could possibly go wrong? It'll be fine.
				ptr.write((/* pos_rad: */glm::vec4(pos.x, pos.y, pos.z, radius), /* color: */color));
				ptr = ptr.add(1);
			}
		}
		#[cfg(target_arch="wasm32")] {
			attributes.get_mapped_range_mut(..).copy_from_slice(&_stagingMem.unwrap());
		}
		attributes.unmap(); // <- make uploaded data visible to GPU

		// Done!
		Arc::new(Self { num: data.num(), layout, attributes })
	}
}
impl renderer::GpuData for GpuData
{
	fn num (&self) -> u32 {
		self.num
	}

	fn layout (&self) -> &gpu::BufferLayout {
		&self.layout
	}

	fn geometry (&self) -> Vec<wgpu::BufferSlice<'_>> {
		vec![self.attributes.slice(..)]
	}

	fn topology (&self) -> wgpu::PrimitiveTopology {
		wgpu::PrimitiveTopology::PointList
	}
}
impl gpu::Interleaved for GpuData {}
