
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
use crate::{self as cgv, *, renderer::{data::*, *}};
use data::*;



//////
//
// Structs
//

///
pub struct DataReceiver {
	data: Arc<dyn renderer::GpuData>,
	layout: GpuPipelineBufferLayout,
	defaultAttribValues: ConstantAttributes,
	vsEntryPoint: String,
	fsEntryPoint: String
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
		// Infer the right shader entry point and vertex shader locations from the available attributes
		let layout = data.layout();
		let mut vsEntryPoint = "vertexMain_pos".to_string();
		let mut fsEntryPoint = "fragmentMain_pos".to_string();
		let mut shaderLoc = 0;
		let mut includeAttribs = vec![];
		if let Some(radii) = layout.attribute(GA::Radii) {
			if layout.positions.inSameBufferSlot(&radii) { vsEntryPoint += "Rad" }
			else                                         { vsEntryPoint += "SepRad"; shaderLoc = 1 }
			fsEntryPoint += "Rad";
			includeAttribs.push((GA::Radii, shaderLoc));
		}
		if layout.hasAttribute(GA::Colors) {
			vsEntryPoint += "Color"; shaderLoc += 1;
			fsEntryPoint += "Color";
			includeAttribs.push((GA::Colors, shaderLoc));
		}

		// Create pipeline buffer layout
		let layout = GpuPipelineBufferLayout::create(
			layout, 0, wgpu::VertexStepMode::Instance, &includeAttribs
		);

		// Done!
		Self { data, layout, defaultAttribValues: ConstantAttributes::default(), vsEntryPoint, fsEntryPoint }
	}

	/// Modify the default radius that will be used when the received [`GpuData`](renderer::GpuData) does not include
	/// radii.
	#[inline(always)]
	pub fn defaultRadius (mut self, radius: f32) -> Self {
		self.defaultAttribValues.radius = radius;
		self
	}

	/// Modify the default color that will be used when the received [`GpuData`](renderer::GpuData) does not include
	/// colors.
	#[inline(always)]
	pub fn defaultColor (mut self, color: cgv::RGBA) -> Self {
		self.defaultAttribValues.color = color;
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
		// Since we use instancing, our pipeline depends on the instance attributes in the vertex state. This could be
		// avoided with attribute-less rendering, but we don't want to give up on the potentially significant
		// performance advantage afforded by the vertex pipeline FIFO cache, which we stand to gain a lot from due to
		// our low number of vertex attributes. (Note: using a compute shader to emulate the geometry shader would not
		// fundamentally change this argument – we still need to get the attributes to the fragment shader)
		false
	}

	fn createGpuState (
		&self, context: &Context, renderState: &RenderState, data: &Self::GpuDataReceiver
	) -> Self::GpuState
	{
		// Construct vertex state
		let vertexState = wgpu::VertexState {
			module: &self.shader,
			entry_point: Some(&data.vsEntryPoint),
			buffers: &data.layout.bufferLayouts(),
			compilation_options: wgpu::PipelineCompilationOptions::default(),
		};

		// Create pipeline
		let pipeline = context.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("CGV__renderer_Spheres_RenderPipeline"),
			layout: Some(&self.pipelineLayout),
			vertex: vertexState,
			fragment: Some(wgpu::FragmentState {
				module: &self.shader,
				entry_point: Some(&data.fsEntryPoint),
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
