
//////
//
// Imports
//

// Standard library
use std::sync::Arc;

// Local imports
use crate::{*, renderer::{*, data::{*, gpu::{self, SAS}}}};



//////
//
// Structs
//

/// Additional options to influence how an [`InterleavedBuffer`] stores and layouts its contents.
#[derive(Clone,Copy)]
pub struct InterleavedBufferOptions {
	topology: wgpu::PrimitiveTopology,
	radiusStorage: gpu::ScalarAttributeStorage,
	radiusDerivStorage: gpu::ScalarAttributeStorage
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
	layout: gpu::BufferLayout,
	buffer: wgpu::Buffer,
	topology: wgpu::PrimitiveTopology
}
impl InterleavedBuffer {
	/*pub fn fromHost<D: HostData> (
		context: &Context, data: D, specialOptions: Option<InterleavedBufferOptions>, label: Option<&str>
	) -> Arc<Self>
	{
		/// Gather host attributes
	}*/
}
impl GpuData for InterleavedBuffer
{
	fn num (&self) -> u32 {
		self.num
	}

	fn layout (&self) -> &gpu::BufferLayout {
		&self.layout
	}

	fn geometry (&self) -> Vec<wgpu::BufferSlice<'_>> {
		vec![self.buffer.slice(..)]
	}

	fn topology (&self) -> wgpu::PrimitiveTopology {
		self.topology
	}
}
impl gpu::Interleaved for InterleavedBuffer {}
