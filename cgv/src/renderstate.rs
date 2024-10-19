
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
use hal::DepthFormat::*;



//////
//
// Enums
//

#[allow(dead_code)]
pub enum DepthStencilMode {
	Depth(hal::DepthFormat),
	DepthStencil(hal::DepthStencilFormat),
	Disabled,
}



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

pub struct RenderState<'a>
{
	pub viewing: ViewingUniforms,
	pub viewingUniformBuffer: wgpu::Buffer,
	pub viewingUniformsBindGroupLayout: wgpu::BindGroupLayout,
	pub viewingUniformsBindGroup: wgpu::BindGroup,

	pub mainSurfaceColorAttachment: Option<wgpu::RenderPassColorAttachment<'static>>,
	pub mainSurfaceColorView: Option<wgpu::TextureView>,

	mainSurfaceDepthStencilMode: DepthStencilMode,
	pub mainSurfaceDepthStencilTex: Option<hal::Texture<'a>>,
	pub mainSurfaceDepthStencilAttachment: Option<wgpu::TextureView>,
	pub depthStencilState: Option<wgpu::DepthStencilState>
}

impl<'a> RenderState<'a>
{
	pub fn new(context: &'a Context) -> Self
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

		// Depth-stencil texture
		let mainSurfaceDepthStencilMode = DepthStencilMode::Depth(D32);
		let mainSurfaceDepthStencilTex = Self::recreateMainDepthStencilTexture(
			context, &mainSurfaceDepthStencilMode
		);
		let depthStencilState = wgpu::DepthStencilState {
			format: mainSurfaceDepthStencilTex.as_ref().unwrap().descriptor.format,
			depth_write_enabled: true,
			depth_compare: wgpu::CompareFunction::Less, // 1.
			stencil: wgpu::StencilState::default(), // 2.
			bias: wgpu::DepthBiasState::default(),
		};

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
			mainSurfaceColorView: None,
			// - depth/stencil
			mainSurfaceDepthStencilMode,
			mainSurfaceDepthStencilTex: mainSurfaceDepthStencilTex,
			mainSurfaceDepthStencilAttachment: None,
			depthStencilState: Some(depthStencilState)
		}
	}

	fn recreateMainDepthStencilTexture (context: &Context, mode: &DepthStencilMode) -> Option<hal::Texture<'a>>
	{
		match mode {
			DepthStencilMode::Depth(format)
			=> {
				Some(hal::Texture::createDepthTexture(
					&context.device, &context.config, *format, Some("MainSurfaceDepthStencilTex")
				))
			}
			DepthStencilMode::DepthStencil(format)
			=> {
				Some(hal::Texture::createDepthStencilTexture(
					&context.device, &context.config, *format, Some("MainSurfaceDepthStencilTex")
				))
			}
			DepthStencilMode::Disabled => None
		}
	}

	pub(crate) fn updateSize (&mut self, context: &Context)
	{
		if context.size == self.mainSurfaceDepthStencilTex.as_ref().unwrap().physicalSizeWH() {
			return;
		}
		self.mainSurfaceDepthStencilTex = Self::recreateMainDepthStencilTexture(
			context, &self.mainSurfaceDepthStencilMode
		);
	}
}


////
// RenderStatePrivateInterface

pub(crate) trait RenderStatePrivateInterface {
	fn updateSurfaceAttachments (&'static mut self, context: &Context) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError>;
}

impl<'a> RenderStatePrivateInterface for RenderState<'a> {
	fn updateSurfaceAttachments (&'static mut self, context: &Context) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError>
	{
		// Obtain the current surface texture and view
		let output = context.surface.get_current_texture()?;
		self.mainSurfaceColorView = Some(
			output.texture.create_view(&wgpu::TextureViewDescriptor::default())
		);

		// Update the color attachment
		self.mainSurfaceColorAttachment = Some(wgpu::RenderPassColorAttachment {
			view: self.mainSurfaceColorView.as_ref().unwrap(),
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
