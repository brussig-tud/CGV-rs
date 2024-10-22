
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

pub enum ColorAttachmentSource<'a> {
	Surface,
	Texture(&'a hal::Texture<'a>)
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
// ColorAttachment

pub struct ColorAttachment {
	view: wgpu::TextureView,
	readbackBuffer: Option<wgpu::Buffer>
}


////
// DepthStencilAttachment

/// Encapsulates inter-referencing state for depth stencil attachments.
pub struct DepthStencilAttachment<'a> {
	texture: hal::Texture<'a>,
	readbackBuffer: wgpu::Buffer,
	defaultState: wgpu::DepthStencilState
}


////
// RenderState

pub struct RenderState
{
	pub viewing: ViewingUniforms,
	pub viewingUniformBuffer: wgpu::Buffer,
	pub viewingUniformsBindGroupLayout: wgpu::BindGroupLayout,
	pub viewingUniformsBindGroup: wgpu::BindGroup,

	clearColor: wgpu::Color,
	colorAttachmentSource: ColorAttachmentSource<'static>,
	colorAttachment: Option<ColorAttachment>,

	depthStencilFormat: DepthStencilFormat,
	pub depthStencilAttachment: Option<DepthStencilAttachment<'static>>
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

			clearColor: wgpu::Color{r: 0.3, g: 0.5, b: 0.7, a: 1.},
			colorAttachmentSource: ColorAttachmentSource::Surface,
			colorAttachment: None,

			depthStencilFormat: DepthStencilFormat::Depth(D32),
			depthStencilAttachment: None
		};


		////
		// Initialize inter-referencing fields

		result.recreateMainSurfaceDepthStencilAttachment(context);
		result
	}

	fn recreateMainSurfaceDepthStencilAttachment (&mut self, context: &Context)
	{
		// Early-out: no depth/stencil attachment
		if let DepthStencilFormat::Disabled = self.depthStencilFormat {
			self.depthStencilAttachment = None;
			return;
		}

		// Make sure we have a static instance to self to perform operations on
		let this = util::mutify(self);

		// Recreate according to selected main depth/stencil mode
		// - initialize fields that are reference targets
		let dims: glm::UVec2 = glm::vec2(context.config.width.max(1), context.config.height.max(1));
		let texture = match &this.depthStencilFormat {
			DepthStencilFormat::Depth(format) => hal::Texture::createDepthTexture(
				&context.device, &dims, *format, Some("MainSurfaceDepthStencilTex")
			),
			DepthStencilFormat::DepthStencil(format) => hal::Texture::createDepthStencilTexture(
				&context.device, &dims, *format, Some("MainSurfaceDepthStencilTex")
			),
			_ => unreachable!()
		};
		// - create the attachment struct with trivially initializable fields constructed in-place
		self.depthStencilAttachment = Some(DepthStencilAttachment {
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

	pub fn getMainSurfaceColorAttachment (&self) -> Option<wgpu::RenderPassColorAttachment>
	{
		if let Some(ca) = &self.colorAttachment {
			Some(wgpu::RenderPassColorAttachment {
				view: &ca.view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: /*match self.clearColor {
						Some(color) => wgpu::LoadOp::Clear(color),
						None => wgpu::LoadOp::Load
					}*/wgpu::LoadOp::Clear(self.clearColor),
					store: wgpu::StoreOp::Store,
				},
			})
		}
		else {
			None
		}
	}

	pub fn getMainSurfaceDepthStencilAttachment (&self) -> Option<wgpu::RenderPassDepthStencilAttachment>
	{
		if let Some(dsa) = &self.depthStencilAttachment {
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
		if let Some(dsa) = &self.depthStencilAttachment {
			Some(dsa.defaultState.clone())
		}
		else {
			None
		}
	}

	pub(crate) fn updateSize (&mut self, context: &Context)
	{
		if context.size == self.depthStencilAttachment.as_ref().unwrap().texture.physicalSizeWH() {
			return;
		}
		self.recreateMainSurfaceDepthStencilAttachment(context);
	}
}


////
// RenderStatePrivateInterface

pub(crate) trait RenderStatePrivateInterface {
	fn updateSurfaceColorAttachment (&mut self, context: &Context);
}

impl<'a> RenderStatePrivateInterface for RenderState {
	fn updateSurfaceColorAttachment (&mut self, context: &Context)
	{
		// Make sure we have a static instance to self to perform operations on
		let this = util::mutify(self);

		// Update view and attachment
		match this.colorAttachmentSource
		{
			ColorAttachmentSource::Surface => {
				let surfaceTexture = context.surfaceTexture.as_ref().unwrap();
				let size = {
					let dims = surfaceTexture.texture.size();
					  hal::numBytesFromFormat(surfaceTexture.texture.format())
					* dims.width as u64 * dims.height as u64 * dims.depth_or_array_layers as u64
				};
				this.colorAttachment = Some(ColorAttachment{
					view: surfaceTexture.texture.create_view(&wgpu::TextureViewDescriptor::default()),
					readbackBuffer: Some(context.device.create_buffer(&wgpu::BufferDescriptor {
						label: Some("ColorAttachmentReadbackBuffer"),
						size, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
						mapped_at_creation: false
					}))
				});
			}
			ColorAttachmentSource::Texture(texture) => this.colorAttachment = Some(ColorAttachment{
				view: texture.texture.create_view(&wgpu::TextureViewDescriptor::default()),
				readbackBuffer: None
			})
		}
	}
}
