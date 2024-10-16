
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

pub struct RenderState
{
	pub viewing: ViewingUniforms,
	pub viewingUniformBuffer: wgpu::Buffer,
	pub viewingUniformsBindGroupLayout: wgpu::BindGroupLayout,
	pub viewingUniformsBindGroup: wgpu::BindGroup,

	pub mainSurfaceColorAttachment: Option<wgpu::RenderPassColorAttachment<'static>>,
	pub mainSurfaceColorView: Option<wgpu::TextureView>
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
			// Uniforms
			// - viewing
			viewingUniformBuffer,
			viewingUniformsBindGroupLayout,
			viewingUniformsBindGroup,
			viewing: Default::default(),

			// Main surface attachments
			mainSurfaceColorAttachment: None,
			mainSurfaceColorView: None
		}
	}
}


////
// RenderStatePrivateInterface

pub(crate) trait RenderStatePrivateInterface {
	fn updateSurfaceAttachments (&mut self, context: &Context) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError>;
}

impl RenderStatePrivateInterface for RenderState {
	fn updateSurfaceAttachments (&mut self, context: &Context) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError>
	{
		// Obtain the current surface texture and view
		let output = context.surface.get_current_texture()?;
		self.mainSurfaceColorView = Some(
			output.texture.create_view(&wgpu::TextureViewDescriptor::default())
		);

		// Update the color attachment
		self.mainSurfaceColorAttachment = Some(wgpu::RenderPassColorAttachment {
			view: util::statify(self).mainSurfaceColorView.as_ref().unwrap(),
			resolve_target: None,
			ops: wgpu::Operations {
				load: wgpu::LoadOp::Clear(wgpu::Color {
					r: 0.3,
					g: 0.5,
					b: 0.7,
					a: 1.,
				}),
				store: wgpu::StoreOp::Store,
			},
		});
		Ok(output)
	}
}
