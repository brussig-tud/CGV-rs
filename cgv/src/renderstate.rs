
//////
//
// Imports
//

// WGPU API
use wgpu;
use wgpu::util::DeviceExt;

// GLM library
use glm;

// Local imports
use crate::*;



//////
//
// Classes
//

////
// ViewingUniforms

// The CPU-side representation of the UniformBuffer used for storing the viewing information.
#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct ViewingUniforms
{
	/// The modelview transformation matrix.
	pub modelview: glm::Mat4,

	/// The projection matrix.
	pub projection: glm::Mat4
}


////
// RenderState

pub struct RenderState {
	pub viewing: ViewingUniforms,
	pub viewingUniformBuffer: wgpu::Buffer,
	pub viewingUniformsBindGroup: wgpu::BindGroup,
}

impl RenderState
{
	pub fn new(context: &Context) -> Self
	{
		// Uniforms and associated buffer and bind group
		// - viewing
		let viewingUniformBuffer = context.device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("ViewingUniforms"),
				contents: util::slicify(&ViewingUniforms::default()),
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			}
		);
		let viewingUniformsBindGroupLayout = context.device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						count: None,
					}
				],
				label: Some("ViewingUniformsBindGroupLayout"),
			}
		);
		let viewingUniformsBindGroup = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &viewingUniformsBindGroupLayout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: viewingUniformBuffer.as_entire_binding(),
				}
			],
			label: Some("ViewingUniformsBindGroup"),
		});

		// Done!
		Self {
			// viewing
			viewingUniformBuffer,
			viewingUniformsBindGroup,
			viewing: Default::default()
		}
	}
}
