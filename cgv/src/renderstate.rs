
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
// RenderState

pub struct RenderState
{
	pub viewing: ViewingUniforms,
	pub viewingUniformBuffer: wgpu::Buffer,
	pub viewingUniformsBindGroupLayout: wgpu::BindGroupLayout,
	pub viewingUniformsBindGroup: wgpu::BindGroup,

	mainSurfaceDepthStencilFormat: DepthStencilFormat,
	pub defaultDepthStencilState: wgpu::DepthStencilState,

	pub mainSurfaceColorAttachment: Option<wgpu::RenderPassColorAttachment<'static>>,
	pub mainSurfaceColorView: Option<wgpu::TextureView>,

	pub mainSurfaceDepthStencilTex: Option<hal::Texture<'static>>,
	pub mainSurfaceDepthStencilReadbackView: Option<wgpu::ImageCopyTexture<'static>>,
	pub mainSurfaceDepthStencilReadbackBuffer: Option<wgpu::Buffer>,
	pub mainSurfaceDepthStencilAttachment: Option<wgpu::RenderPassDepthStencilAttachment<'static>>,
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
		// Construct base object

		// Henceforth, we mutate this object (doing it this way takes care of lifetime troubles)
		let mut result = Self {
			viewing: Default::default(),
			viewingUniformBuffer,
			viewingUniformsBindGroupLayout,
			viewingUniformsBindGroup,

			defaultDepthStencilState: wgpu::DepthStencilState {
				format: wgpu::TextureFormat::Depth32Float,
				depth_write_enabled: true,
				depth_compare: wgpu::CompareFunction::Less, // 1.
				stencil: wgpu::StencilState::default(), // 2.
				bias: wgpu::DepthBiasState::default(),
			},
			mainSurfaceDepthStencilFormat: DepthStencilFormat::Depth(D32),

			mainSurfaceColorAttachment: None,
			mainSurfaceColorView: None,
			mainSurfaceDepthStencilTex: None,
			mainSurfaceDepthStencilReadbackView: None,
			mainSurfaceDepthStencilReadbackBuffer: None,
			mainSurfaceDepthStencilAttachment: None
		};


		////
		// Initialize inter-referencing fields

		result.recreateMainDepthStencilObjects(context);
		result
	}

	fn recreateMainDepthStencilObjects (&mut self, context: &Context)
	{
		// Make sure we have a static instance to self to perform operations on
		let this = util::mutify(self);

		let updateRemainingFields = || {
			let this = util::mutify(self);
			this.mainSurfaceDepthStencilReadbackView = Some(
				this.mainSurfaceDepthStencilTex.as_ref().unwrap().texture.as_image_copy()
			);
			this.mainSurfaceDepthStencilReadbackBuffer =
				Some(context.device.create_buffer(&wgpu::BufferDescriptor {
					label: Some("MainSurfaceDepthStencilReadbackBuffer"),
					size: this.mainSurfaceDepthStencilTex.as_ref().unwrap().size(),
					usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
					mapped_at_creation: false
				}));
		};

		// Recreate according to selected main depth/stencil mode
		match &this.mainSurfaceDepthStencilFormat {
			DepthStencilFormat::Depth(format)
			=> {
				this.mainSurfaceDepthStencilTex = Some(hal::Texture::createDepthTexture(
					&context.device, &context.config, *format, Some("MainSurfaceDepthStencilTex")
				));
				updateRemainingFields();
			}
			DepthStencilFormat::DepthStencil(format)
			=> {
				this.mainSurfaceDepthStencilTex = Some(hal::Texture::createDepthStencilTexture(
					&context.device, &context.config, *format, Some("MainSurfaceDepthStencilTex")
				));
				updateRemainingFields();
			}
			DepthStencilFormat::Disabled => {
				this.mainSurfaceDepthStencilReadbackView = None;
				this.mainSurfaceDepthStencilTex = None;
				this.mainSurfaceDepthStencilReadbackBuffer = None;
			}
		}
	}

	pub(crate) fn updateSize (&'static mut self, context: &Context)
	{
		if context.size == self.mainSurfaceDepthStencilTex.as_ref().unwrap().physicalSizeWH() {
			return;
		}
		self.recreateMainDepthStencilObjects(context);
		self.resetDepthStencilAttachment();
	}
}


////
// RenderStatePrivateInterface

pub(crate) trait RenderStatePrivateInterface {
	fn updateSurfaceColorAttachment (&mut self, context: &Context)
		-> Result<wgpu::SurfaceTexture, wgpu::SurfaceError>;

	fn resetDepthStencilAttachment (&mut self);
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
	fn resetDepthStencilAttachment (&mut self)
	{
		// Make sure we have a static instance to self to perform operations on
		let this = util::mutify(self);

		// Obtain the current surface texture and view
		this.mainSurfaceDepthStencilAttachment = Some(wgpu::RenderPassDepthStencilAttachment {
			view: &this.mainSurfaceDepthStencilTex.as_ref().unwrap().view,
			depth_ops: Some(wgpu::Operations {
				load: wgpu::LoadOp::Clear(1.)/*wgpu::LoadOp::Load*/,
				store: wgpu::StoreOp::Store,
			}),
			stencil_ops: None,
		});
	}
}
