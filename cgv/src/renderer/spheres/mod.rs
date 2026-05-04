
//////
//
// Module definitions
//

/// Private submodule defining our various GPU-side data representations.
mod data;
pub use data::GpuData; // re-export



//////
//
// Imports
//

// Standard library
use std::sync::{LazyLock, Arc};

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
	_constantAttribUniforms: ConstantAttribsUniformGroup,
	data: Option<Arc<dyn renderer::GpuData>>
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
		Self { shader, pipelineLayout, _constantAttribUniforms: constantAttribUniforms, data: None }
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
				entry_point: Some("vertexMain"),
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

	fn setData (&mut self, data: Arc<dyn renderer::GpuData>) {
		self.data.replace(data);
	}

	fn render (&self, _context: &Context, _gpuObjects: &Self::GpuState) {
		todo!()
	}
}
