
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

	/// Construct the variant that defines only positions.
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
	pub radius: f32,

	///
	pub color: Rgba,
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
	///
	pub fn new<D: HostData> (data: D) -> Self {
		let variant = LayoutVariant::PosOnly;
		let _layout = variant.layout();
		todo!("fill buffer")
	}

	///
	pub fn withRadii<D: HostData+host::HasRadii> (data: D) -> Self {
		let variant = LayoutVariant::PosRadius;
		let _layout = variant.layout();
		todo!("fill buffer")
	}

	///
	pub fn withColors<D: HostData+host::HasColors> (data: D) -> Self {
		let variant = LayoutVariant::PosColor;
		let _layout = variant.layout();
		todo!("fill buffer")
	}

	///
	pub fn withRadiiAndColors<D: HostData+host::HasRadii+host::HasColors> (data: D) -> Self {
		let variant = LayoutVariant::PosRadiusColor;
		let _layout = variant.layout();
		todo!("fill buffer")
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

	fn geometry (&self) -> &[wgpu::BufferSlice<'_>] {
		todo!("return slice of filled buffer")
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
impl<D: HostData+host::CanHaveRadii+host::CanHaveColors> From<&D> for GpuData {
	fn from (other: &D) -> Self
	{
		let variant = match (other.hasRadii(), other.hasColors()) {
			(false, false) => LayoutVariant::PosOnly,
			(true, false) => LayoutVariant::PosRadius,
			(false, true) => LayoutVariant::PosColor,
			(true, true) => LayoutVariant::PosRadiusColor
		};
		let _layout = variant.layout();
		todo!("fill buffer")
	}
}
