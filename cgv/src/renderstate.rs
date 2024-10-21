
//////
//
// Imports
//

// Standard library
use std::default::Default;

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
pub enum DepthStencilFormat {
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
// DepthStencilAttachment

/// Encapsulates inter-referencing state for depth stencil attachments.
pub struct DepthStencilAttachment<'a> {
	pub texture: hal::Texture<'a>,
	pub readbackBuffer: wgpu::Buffer,
	pub defaultState: wgpu::DepthStencilState
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
	pub mainSurfaceColorView: Option<wgpu::TextureView>,

	mainSurfaceDepthStencilFormat: DepthStencilFormat,
	pub mainSurfaceDepthStencilAttachment: Option<DepthStencilAttachment<'static>>
}

impl RenderState
{
	pub fn new(context: &Context) -> Self
	{
		////
		// Prepare non-inter-referencing fields

		// Uniforms and associated buffers and bind groups
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


		////
		// Construct result object

		// Henceforth, we mutate this result object for the remaining initialization
		let mut result = Self {
			viewing: Default::default(),
			viewingUniformBuffer,
			viewingUniformsBindGroupLayout,
			viewingUniformsBindGroup,

			mainSurfaceDepthStencilFormat: DepthStencilFormat::Depth(D32),

			mainSurfaceColorAttachment: None,
			mainSurfaceColorView: None,

			mainSurfaceDepthStencilAttachment: None
		};


		////
		// Initialize inter-referencing fields

		result.recreateMainDepthStencilObjects(context);
		result
	}

	fn recreateMainDepthStencilObjects (&mut self, context: &Context)
	{
		// Early-out: no depth/stencil attachment
		if let DepthStencilFormat::Disabled = self.mainSurfaceDepthStencilFormat {
			self.mainSurfaceDepthStencilAttachment = None;
			return;
		}

		// Make sure we have a static instance to self to perform operations on
		let this = util::mutify(self);

		// Recreate according to selected main depth/stencil mode
		// - initialize fields that are reference targets
		let dims: glm::UVec2 = glm::vec2(context.config.width.max(1), context.config.height.max(1));
		let texture = match &this.mainSurfaceDepthStencilFormat {
			DepthStencilFormat::Depth(format) => hal::Texture::createDepthTexture(
				&context.device, &dims, *format, Some("MainSurfaceDepthStencilTex")
			),
			DepthStencilFormat::DepthStencil(format) => hal::Texture::createDepthStencilTexture(
				&context.device, &dims, *format, Some("MainSurfaceDepthStencilTex")
			),
			_ => { /* should be impossible */panic!("!!!INTERNAL LOGIC ERROR!!!") }
		};
		// - create the attachment struct with trivially initializable fields constructed in-place
		self.mainSurfaceDepthStencilAttachment = Some(DepthStencilAttachment {
			readbackBuffer: context.device.create_buffer(&wgpu::BufferDescriptor {
				label: Some("MainSurfaceDepthStencilReadbackBuffer"),
				size: texture.size(),
				usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
				mapped_at_creation: false
			}),
			defaultState: wgpu::DepthStencilState {
				format: texture.texture.format(),
				depth_write_enabled: true,
				depth_compare: wgpu::CompareFunction::Less, // 1.
				stencil: wgpu::StencilState::default(), // 2.
				bias: wgpu::DepthBiasState::default(),
			}, texture
		});
	}

	pub fn getMainSurfaceDepthStencilAttachment(&self) -> Option<wgpu::RenderPassDepthStencilAttachment>
	{
		if let Some(dsa) = &self.mainSurfaceDepthStencilAttachment {
			Some(wgpu::RenderPassDepthStencilAttachment {
				view: &dsa.texture.view,
				depth_ops: Some(wgpu::Operations {
					load: wgpu::LoadOp::Clear(1.)/*wgpu::LoadOp::Load*/,
					store: wgpu::StoreOp::Store,
				}),
				stencil_ops: None,
			})
		}
		else {
			None
		}
	}

	pub fn getMainSurfaceDepthStencilState (&self) -> Option<wgpu::DepthStencilState>
	{
		if let Some(dsa) = &self.mainSurfaceDepthStencilAttachment {
			Some(dsa.defaultState.clone())
		}
		else {
			None
		}
	}

	pub(crate) fn updateSize (&mut self, context: &Context)
	{
		if context.size == self.mainSurfaceDepthStencilAttachment.as_ref().unwrap().texture.physicalSizeWH() {
			return;
		}
		self.recreateMainDepthStencilObjects(context);
	}
}


////
// RenderStatePrivateInterface

pub(crate) trait RenderStatePrivateInterface {
	fn updateSurfaceColorAttachment (&mut self, context: &Context)
		-> Result<wgpu::SurfaceTexture, wgpu::SurfaceError>;
}

impl<'a> RenderStatePrivateInterface for RenderState {
	fn updateSurfaceColorAttachment (&mut self, context: &Context)
		-> Result<wgpu::SurfaceTexture, wgpu::SurfaceError>
	{
		// Make sure we have a static instance to self to perform operations on
		let this = util::mutify(self);

		// Obtain the current surface texture and view
		let output = context.surface.get_current_texture()?;
		this.mainSurfaceColorView = Some(
			output.texture.create_view(&wgpu::TextureViewDescriptor::default())
		);

		// Update the color attachment
		this.mainSurfaceColorAttachment = Some(wgpu::RenderPassColorAttachment {
			view: this.mainSurfaceColorView.as_ref().unwrap(),
			resolve_target: None,
			ops: wgpu::Operations {
				load: wgpu::LoadOp::Clear(wgpu::Color {
					r: 0.3,
					g: 0.5,
					b: 0.7,
					a: 1.,
				})/*wgpu::LoadOp::Load*/,
				store: wgpu::StoreOp::Store,
			},
		});
		Ok(output)
	}
}
