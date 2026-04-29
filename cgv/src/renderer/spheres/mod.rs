
//////
//
// Module definitions
//

/// Private submodule defining our various GPU-side data representations.
mod data;
use data::Data;



//////
//
// Imports
//

// Standard library
use std::sync::LazyLock;

// Egui library
use egui::ecolor::Rgba;

// Local imports
use crate::{*, renderer::*};
use data::*;



//////
//
// Structs
//

///
pub struct Spheres {
	shader: wgpu::ShaderModule,
	pipelineLayout: wgpu::PipelineLayout,
	constantAttribUniforms: ConstantAttribsUniformGroup,
	data: Option<Box<dyn GpuData>>
}
impl Spheres
{
	fn shaderPackage<'outer> () -> &'outer shader::Package
	{
		static SHADER_PACKAGE: LazyLock<shader::Package> = LazyLock::new(||
			shader::Package::deserialize(
				util::sourceGeneratedBytes!("/shader/renderer/spheres.spk")
			).expect("baked 'spheres.spk' shader package should be available and valid")
		);
		&SHADER_PACKAGE
	}

	pub fn new (context: &Context, renderSetup: &RenderSetup) -> Self
	{
		// Create constant (not state-dependent) GPU objects
		let constantAttribUniforms = ConstantAttribsUniformGroup::create(
			context, wgpu::ShaderStages::VERTEX_FRAGMENT,
			Some("CGV__renderer_Spheres_constantAttribUniforms").as_deref()
		);
		let pipelineLayout =
			context.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("CGV__renderer_Spheres_renderPipelineLayout"),
				bind_group_layouts: &[
					Some(&renderSetup.bindGroupLayouts().viewing), Some(&constantAttribUniforms.bindGroupLayout)
				],
				immediate_size: 0
			});
		let shader = Self::shaderPackage().createShaderModuleFromBestInstance(
			context.device(), None, Some("CGV__renderer_Spheres_shaderModule")
		).expect("shader module could not be compiled by WGPU");

		// Done!
		Self { shader, pipelineLayout, constantAttribUniforms, data: None }
	}

	pub fn setData (&mut self, data: impl GpuData+'static) {
		self.data.replace(Box::new(data));
	}
}
impl Renderer for Spheres
{
	type GpuState = wgpu::RenderPipeline;

	fn createGpuState (&self, context: &Context, renderState: &RenderState) -> Self::GpuState
	{
		// Create pipeline
		let pipeline = context.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("CGV__renderer_Spheres_RenderPipeline"),
			layout: Some(&self.pipelineLayout),
			vertex: wgpu::VertexState {
				module: &self.shader,
				entry_point: Some("vertexMain_posOnly"),
				buffers: &[/* no vertex buffers, we use a shader-internal constant billboard */],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			},
			fragment: Some(wgpu::FragmentState {
				module: &self.shader,
				entry_point: Some("fragmentMain_posOnly"),
				targets: &[Some(renderstate::changeColorTargetState_blending(
					renderState.colorTargetState(), renderstate::BlendingOperation::AlphaPreMultiplied
				))],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleStrip,
				..Default::default()
			},
			depth_stencil: Some(renderState.depthStencilState().clone()),
			multisample: wgpu::MultisampleState::default(),
			multiview_mask: None,
			cache: None
		});

		// Done!
		pipeline
	}

	fn render (&self, _context: &Context, _gpuObjects: &Self::GpuState) {
		todo!()
	}
}
