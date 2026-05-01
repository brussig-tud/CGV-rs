
//////
//
// Imports
//

// Local imports
use super::*;
use crate::{self as cgv, renderer::data::gpu, renderer::data::host};



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
	const POS_ONLY: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0=>Float32x4]; // 4-th component is unused
	const POS_ONLY_STRIDE: wgpu::BufferAddress = size_of::<glm::Vec4>() as wgpu::BufferAddress;

	const POS_RADIUS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0=>Float32x4]; // 4-th component is radius
	const POS_RADIUS_STRIDE: wgpu::BufferAddress = size_of::<glm::Vec4>() as wgpu::BufferAddress;

	const POS_COLOR: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0=>Float32x4, 1=>Float32x4];
	const POS_COLOR_STRIDE: wgpu::BufferAddress = (size_of::<glm::Vec4>()*2) as wgpu::BufferAddress;

	const POS_RADIUS_COLOR: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0=>Float32x4, 1=>Float32x4];
	const POS_RADIUS_COLOR_STRIDE: wgpu::BufferAddress = (size_of::<glm::Vec4>()*2) as wgpu::BufferAddress;

	/// Construct a vertex buffer layout that fits the variant represented by `self`.
	pub fn layout (&self) -> wgpu::VertexBufferLayout<'static>
	{
		match self
		{
			Self::PosOnly => wgpu::VertexBufferLayout {
				array_stride: Self::POS_ONLY_STRIDE,
				step_mode: wgpu::VertexStepMode::Vertex,
				attributes: &Self::POS_ONLY,
			},
			Self::PosRadius => wgpu::VertexBufferLayout {
				array_stride: Self::POS_RADIUS_STRIDE,
				step_mode: wgpu::VertexStepMode::Vertex,
				attributes: &Self::POS_RADIUS,
			},
			Self::PosColor => wgpu::VertexBufferLayout {
				array_stride: Self::POS_COLOR_STRIDE,
				step_mode: wgpu::VertexStepMode::Vertex,
				attributes: &Self::POS_COLOR,
			},
			Self::PosRadiusColor => wgpu::VertexBufferLayout {
				array_stride: Self::POS_RADIUS_COLOR_STRIDE,
				step_mode: wgpu::VertexStepMode::Vertex,
				attributes: &Self::POS_RADIUS_COLOR,
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
#[derive(Default)]
pub struct ConstantAttributes {
	///
	pub _radius: f32,

	///
	pub _color: Rgba,
}
pub type ConstantAttribsUniformGroup = hal::UniformGroup<ConstantAttributes>;

/// A [`renderer::GpuData`]-compliant storage for sphere attributes.
pub struct GpuData {
	num: u32,
	variant: LayoutVariant,
	layout: [wgpu::VertexBufferLayout<'static>; 1],
	attributes: wgpu::Buffer
}
impl GpuData
{
	/// Helper function for common initialization.
	fn commonInit<D: HostData, T> (context: &Context, variant: LayoutVariant, data: &D, label: Option<&str>)
		-> (LayoutVariant, wgpu::Buffer, std::ptr::NonNull<T>)
	{
		// Prepare the buffer
		let attributes = variant.createBuffer(context, data.num(), label);

		// Gain mapped pointer for uploading
		let ptr = attributes.get_mapped_range_mut(..).slice(..).as_raw_ptr()
			.cast::<T>();

		// Done!
		(variant, attributes, ptr)
	}


	///
	pub fn new<D: HostData> (context: &Context, data: D, label: Option<&str>) -> Self
	{
		// Common initialization
		let (variant, attributes, mut ptr)
			= Self::commonInit(context, LayoutVariant::PosOnly, &data, label);

		// Upload the data
		for pos in data.positions() {
			unsafe {
				// SAFETY: What could possibly go wrong? It'll be fine.
				ptr.write(/* pos_rad: */glm::vec3_to_vec4(pos));
				ptr = ptr.add(1);
			}
		}
		attributes.unmap(); // <- make uploaded data visible to GPU

		// Done!
		Self { num: data.num(), layout: [variant.layout()], variant, attributes }
	}

	///
	pub fn withRadii<D: HostData+host::HasRadii> (context: &Context, data: D, label: Option<&str>) -> Self
	{
		// Common initialization
		let (variant, attributes, mut ptr)
			= Self::commonInit(context, LayoutVariant::PosRadius, &data, label);

		// Upload the data
		for (pos, radius) in data.positions().zip(data.radii()) {
			unsafe {
				// SAFETY: What could possibly go wrong? It'll be fine.
				ptr.write(/* pos_rad: */glm::vec4(pos.x, pos.y, pos.z, *radius));
				ptr = ptr.add(1);
			}
		}
		attributes.unmap(); // <- make uploaded data visible to GPU

		// Done!
		Self { num: data.num(), layout: [variant.layout()], variant, attributes }
	}

	///
	pub fn withColors<D: HostData+host::HasColors> (context: &Context, data: D, label: Option<&str>) -> Self
	{
		// Common initialization
		let (variant, attributes, mut ptr)
			= Self::commonInit(context, LayoutVariant::PosColor, &data, label);

		// Upload the data
		for (pos, color) in data.positions().zip(data.colors()) {
			unsafe {
				// SAFETY: What could possibly go wrong? It'll be fine.
				ptr.write((/* pos_rad: */glm::vec3_to_vec4(pos), /* color: */*color));
				ptr = ptr.add(1);
			}
		}
		attributes.unmap(); // <- make uploaded data visible to GPU

		// Done!
		Self { num: data.num(), layout: [variant.layout()], variant, attributes }
	}

	///
	pub fn withRadiiAndColors<D: HostData+host::HasRadii+host::HasColors> (
		context: &Context, data: D, label: Option<&str>
	) -> Self {
		// Common initialization
		let (variant, attributes, mut ptr)
			= Self::commonInit(context, LayoutVariant::PosRadiusColor, &data, label);

		// Upload the data
		for ((pos, radius), color) in data.positions().zip(data.radii()).zip(data.colors()) {
			unsafe {
				// SAFETY: What could possibly go wrong? It'll be fine.
				ptr.write((/* pos_rad: */glm::vec4(pos.x, pos.y, pos.z, *radius), /* color: */*color));
				ptr = ptr.add(1);
			}
		}
		attributes.unmap(); // <- make uploaded data visible to GPU

		// Done!
		Self { num: data.num(), layout: [variant.layout()], variant, attributes }
	}
}
impl renderer::GpuData for GpuData
{
	fn num (&self) -> u32 {
		self.num
	}

	fn layout (&self) -> &[wgpu::VertexBufferLayout<'static>] {
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
impl gpu::CanHaveRadii for GpuData
{
	fn hasRadii (&self) -> bool {
		matches!(self.variant, LayoutVariant::PosRadius | LayoutVariant::PosRadiusColor)
	}
}
impl gpu::CanHaveColors for GpuData
{
	fn hasColors (&self) -> bool {
		matches!(self.variant, LayoutVariant::PosColor | LayoutVariant::PosRadiusColor)
	}
}
