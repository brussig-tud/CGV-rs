
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// WGPU API
use wgpu;

// Local imports
use crate::*;



//////
//
// Classes
//

pub struct UniformGroup<UniformsStruct: Sized+Default> {
	pub data: UniformsStruct,
	buffer: wgpu::Buffer,
	pub bindGroupLayout: wgpu::BindGroupLayout,
	pub bindGroup: wgpu::BindGroup,
}

impl<UniformsStruct: Sized+Default> UniformGroup<UniformsStruct>
{
	pub fn create (context: &Context, visibility: wgpu::ShaderStages, label: Option<&str>) -> Self
	{
		// Create device objects
		// - buffer
		let buffer = {
			let labelString: String;
			let label = if let Some(label) = label {
				labelString = format!("{label}_buffer");
				Some(labelString.as_str())
			} else { None };
			context.device.create_buffer(
				&wgpu::BufferDescriptor {
					label, size: size_of::<UniformsStruct>() as u64,
					usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
					mapped_at_creation: false,
				}
			)
		};
		// - bind group layout
		let bindGroupLayout = {
			let labelString: String;
			let label = if let Some(label) = label {
				labelString = format!("{label}_bindGroupLayout");
				Some(labelString.as_str())
			} else { None };
			context.device.create_bind_group_layout(
				&wgpu::BindGroupLayoutDescriptor {
					label, entries: &[
						wgpu::BindGroupLayoutEntry {
							binding: 0, visibility,
							ty: wgpu::BindingType::Buffer {
								ty: wgpu::BufferBindingType::Uniform,
								has_dynamic_offset: false,
								min_binding_size: None,
							},
							count: None,
						}
					],
				}
			)
		};
		// - bind group
		let bindGroup = {
			let labelString: String;
			let label = if let Some(label) = label {
				labelString = format!("{label}_bindGroup");
				Some(labelString.as_str())
			} else { None };
			context.device.create_bind_group(&wgpu::BindGroupDescriptor {
				label, layout: &bindGroupLayout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0, resource: buffer.as_entire_binding(),
					}
				]
			})
		};

		// Done!
		Self {data: Default::default(), buffer, bindGroupLayout, bindGroup}
	}

	pub fn upload (&self, context: &Context, immediate: bool) {
		context.queue.write_buffer(&self.buffer, 0, util::slicify(&self.data));
		if immediate {
			context.queue.submit([]);
		}
	}
}
