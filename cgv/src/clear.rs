
//////
//
// Language config
//

// Eff this convention. We're a library so of course we'll be defining stuff that's never used inside our own crate!
#![allow(dead_code)]



//////
//
// Imports
//

// WGPU API
use wgpu;

// Local imports
use crate::*;



//////
//
// Structs
//

pub(crate) struct ClearColor<'a> {
	pub value: wgpu::Color,
	pub attachment: &'a ColorAttachment<'a>
}

pub(crate) struct ClearDepth<'a> {
	pub value: f32,
	pub attachment: &'a DepthStencilAttachment
}



//////
//
// Classes
//

pub(crate) struct Clear {
	clearColor: wgpu::Color,
	clearDepth: f32,
	pipeline: wgpu::RenderPipeline
}

impl Clear
{
	pub fn new (context: &Context, color: Option<&ClearColor>, depth: Option<&ClearDepth>) -> Self
	{
		let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Shader"),
			source: wgpu::ShaderSource::Wgsl(util::sourceFile!("/shader/common/noop.wgsl").into()),
		});

		let pipelineLayout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("CGV__ClearPipeline_layout"),
			bind_group_layouts: &[],
			push_constant_ranges: &[],
		});

		let colorTargets = if let Some(color) = color {
			&[Some(wgpu::ColorTargetState {
				format: match color.attachment {
					ColorAttachment::Surface
					=> context.config.format,
					ColorAttachment::Texture(tex)
					=> tex.descriptor.format,
					ColorAttachment::SurfaceView(_) => unreachable!()
				},
				blend: Some(wgpu::BlendState::REPLACE),
				write_mask: wgpu::ColorWrites::RED // we can't have NONE, so we arbitrarily chose a single one
			})]
		} else { &[None] };

		let pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("CGV__ClearPipeline"),
			layout: Some(&pipelineLayout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: None,
				buffers: &[],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			},
			fragment: if color.is_some() {
				Some(wgpu::FragmentState {
					module: &shader,
					entry_point: None,
					targets: colorTargets,
					compilation_options: wgpu::PipelineCompilationOptions::default(),
				})
			}
			else { None },
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::PointList,
				..Default::default()
			},
			depth_stencil: if let Some(depth) = depth {
				Some(wgpu::DepthStencilState {
					format: depth.attachment.texture.descriptor.format,
					depth_write_enabled: false,
					depth_compare: wgpu::CompareFunction::Never,
					stencil: Default::default(),
					bias: Default::default(),
				})
			}
			else { None },
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
			cache: None,
		});

		Self {
			clearColor: if let Some(color) = color {
				color.value
			} else {
				wgpu::Color::default()
			},
			clearDepth: if let Some(depth) = depth { depth.value } else { -1. },
			pipeline
		}
	}

	pub fn clear (
		&mut self, cmdEncoder: &mut wgpu::CommandEncoder, colorAttachment: &Option<wgpu::RenderPassColorAttachment>,
		depthStencilAttachment: &Option<wgpu::RenderPassDepthStencilAttachment>
	){
		/* make a render pass */ {
			let mut renderPass = cmdEncoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("SampleRenderPass"),
				color_attachments: &[if let Some(ca) = colorAttachment {
					Some(wgpu::RenderPassColorAttachment {
						view: ca.view,
						resolve_target: None,
						ops: wgpu::Operations {
							load: wgpu::LoadOp::Clear(self.clearColor),
							store: wgpu::StoreOp::Store,
						},
					})
				} else {
					None
				}],
				depth_stencil_attachment: if let Some(dsa) = depthStencilAttachment {
					Some(wgpu::RenderPassDepthStencilAttachment {
						view: &dsa.view,
						depth_ops: Some(wgpu::Operations {
							load: wgpu::LoadOp::Clear(self.clearDepth),
							store: wgpu::StoreOp::Store,
						}),
						stencil_ops: None,
					})
				} else {
					None
				},
				occlusion_query_set: None,
				timestamp_writes: None,
			});
			renderPass.set_pipeline(&self.pipeline);
			renderPass.draw(0..1, 0..0);
		}
	}

	pub fn clearImmediately (
		&mut self, context: &Context, colorAttachment: &Option<wgpu::RenderPassColorAttachment>,
		depthStencilAttachment: &Option<wgpu::RenderPassDepthStencilAttachment>
	){
		// Get a command encoder
		let mut cmdEncoder = context.device.create_command_encoder(
			&wgpu::CommandEncoderDescriptor{label: Some("CGV__ClearCommandEncoder")}
		);

		// Schedule actual clearing work
		self.clear(&mut cmdEncoder, colorAttachment, depthStencilAttachment);

		// Dispatch immediately
		context.queue.submit([cmdEncoder.finish()]);
	}
}
