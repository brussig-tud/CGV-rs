
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
use crate::{self as cgv, *, renderer::*};
use data::*;



//////
//
// Enum
//

///
#[derive(Clone,Copy)]
enum PosRadLayoutType {
	/// Positions and radii are in the same `Float32x4` shader location (preferred).
	Composite,

	/// Positions and radii are in separate shader locations.
	Separate
}



//////
//
// Structs
//

///
pub struct DataReceiver {
	data: Arc<dyn renderer::GpuData>,
	posRadLayoutType: Option<PosRadLayoutType>,
	defaultAttibValues: ConstantAttributes,
}
impl DataReceiver
{
	/// Receive the provided GPU data.
	///
	/// **NOTE**: [`renderer::Spheres`] prefers having positions and radii packed into the same `Float32x4` shader
	/// location when radii are present, but will work with separate locations as well, at a (very) small performance
	/// penalty.
	pub fn new (data: Arc<dyn renderer::GpuData>) -> Self
	{
		// Infer the positions/radii layout type
		let layout = data.layout();
		let posRadLayoutType = if let Some(radii) = layout.radii {
			Some(if layout.positions.inSameBufferSlot(&radii) { PosRadLayoutType::Composite }
			     else                                         { PosRadLayoutType::Separate  })
		} else {
			None
		};

		// Done!
		Self { data, posRadLayoutType, defaultAttibValues: ConstantAttributes::default() }
	}

	/// Modify the default radius that will be used when the received [`GpuData`](renderer::GpuData) does not include
	/// radii.
	#[inline(always)]
	pub fn defaultRadius (mut self, radius: f32) -> Self {
		self.defaultAttibValues.radius = radius;
		self
	}

	/// Modify the default color that will be used when the received [`GpuData`](renderer::GpuData) does not include
	/// colors.
	#[inline(always)]
	pub fn defaultColor (mut self, color: cgv::RGBA) -> Self {
		self.defaultAttibValues.color = color;
		self
	}

	/// Modify the default attribute values that will be used when the received [`GpuData`](renderer::GpuData) does not
	/// include any optional attributes.
	#[inline(always)]
	pub fn withDefaultAttributes (self, radius: f32, color: cgv::RGBA) -> Self {
		self.defaultRadius(radius).defaultColor(color)
	}
}
impl GpuDataReceiver for DataReceiver {
	fn gpuData(&self) -> &dyn renderer::GpuData {
		self.data.as_ref()
	}
}

///
pub struct Spheres {
	shader: wgpu::ShaderModule,
	pipelineLayout: wgpu::PipelineLayout,
	_constantAttribUniforms: ConstantAttribsUniformGroup
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
		Self { shader, pipelineLayout, _constantAttribUniforms: constantAttribUniforms }
	}
}
impl Renderer for Spheres
{
	type GpuState = wgpu::RenderPipeline;
	type GpuDataReceiver = spheres::DataReceiver;

	#[inline(always)]
	fn gpuStateIsIndependentFromData (&self) -> bool {
		false
	}

	fn createGpuState (
		&self, context: &Context, renderState: &RenderState, data: &Self::GpuDataReceiver
	) -> Self::GpuState
	{
		// Instantiate our actual data layout
		let layout = data.gpuData().layout();
		let buffers = layout.withStepMode(wgpu::VertexStepMode::Instance);

		// Decide on vertex state based on available attributes
		let vertexState = match &data.posRadLayoutType
		{
			Some(posRadLayout) => {
				debug_assert!(layout.radii.is_some()); // <- sanity check
				if layout.colors.is_some() {
					todo!("not yet implemented")
				}
				else {
					// Positions only, no radii, no colors
					wgpu::VertexState {
						module: &self.shader,
						entry_point: Some("vertexMain_posRad"),
						buffers: &buffers,
						compilation_options: wgpu::PipelineCompilationOptions::default(),
					}
				}
			},

			None => {
				debug_assert!(layout.radii.is_none()); // <- sanity check
				if layout.colors.is_some() {
					todo!("not yet implemented")
				}
				else {
					// Positions only, no radii, no colors
					wgpu::VertexState {
						module: &self.shader,
						entry_point: Some("vertexMain_posOnly"),
						buffers: &buffers,
						compilation_options: wgpu::PipelineCompilationOptions::default(),
					}
				}
			}
		};

		// Create pipeline
		let pipeline = context.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("CGV__renderer_Spheres_RenderPipeline"),
			layout: Some(&self.pipelineLayout),
			vertex: vertexState,
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

	fn render (&self, _context: &Context, _gpuState: &Self::GpuState, _data: &Self::GpuDataReceiver) {
		todo!()
	}
}
