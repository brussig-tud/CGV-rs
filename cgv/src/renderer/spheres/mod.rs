
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
use crate::{*, renderer::{data::*, *}};
use data::*;



//////
//
// Structs
//

///
pub struct DataReceiver {
	data: Arc<dyn renderer::GpuData>,
	includedAttribs: GeometryAttributeFlags,
	layout: GpuPipelineBufferLayout,
	vsEntryPoint: String,
	fsEntryPoint: String
}
impl DataReceiver
{
	/// Receive the provided GPU data.
	///
	#[doc=include_str!("_doc/_spheres_layoutRemarks.md")]
	///
	/// # Arguments
	///
	/// * `data` – The `GpuData` to render.
	///
	/// # Returns
	///
	/// A `DataReceiver` for feeding into a `renderer::Spheres`.
	#[inline(always)]
	pub fn new (data: Arc<dyn renderer::GpuData>) -> Self {
		Self::withAttributes(data, GAF::all())
	}
	/// Receive the provided GPU data, using only the specified attributes, the rest will be fixed as indicated by the
	/// current [style](Spheres::setStyleUniforms).
	///
	#[doc=include_str!("_doc/_spheres_layoutRemarks.md")]
	///
	/// # Arguments
	///
	/// * `data` – The `GpuData` to render.
	/// * `filter` – The attributes to use from the data. Only specifying [`GAF::RADII`] and [`GAF::COLORS`] will have
	///              an effect; other attributes are always ignored by `renderer::Spheres`.
	///
	/// # Returns
	///
	/// A `DataReceiver` for feeding into a `renderer::Spheres`.
	pub fn withAttributes (data: Arc<dyn renderer::GpuData>, filter: GeometryAttributeFlags) -> Self
	{
		// Infer the right shader entry point and vertex shader locations from the available attributes
		let layout = data.layout();
		let mut vsEntryPoint = "vertexMain_pos".to_string();
		let mut fsEntryPoint = "fragmentMain_pos".to_string();
		let mut shaderLoc = 0;
		let mut includeAttribs = vec![];
		let mut includedAttribs = GAF::empty();
		if filter.contains(GAF::RADII) && let Some(radii) = layout.attribute(GA::Radii) {
			if layout.positions.inSameBufferSlot(&radii) { vsEntryPoint += "Rad" }
			else                                         { vsEntryPoint += "SepRad"; shaderLoc = 1 }
			fsEntryPoint += "Rad";
			includeAttribs.push((GA::Radii, shaderLoc));
			includedAttribs |= GAF::RADII;
		}
		if filter.contains(GAF::COLORS) && layout.hasAttribute(GA::Colors) {
			vsEntryPoint += "Color"; shaderLoc += 1;
			fsEntryPoint += "Color";
			includeAttribs.push((GA::Colors, shaderLoc));
			includedAttribs |= GAF::COLORS;
		}

		// Create pipeline buffer layout
		let layout = GpuPipelineBufferLayout::create(
			layout, 0, wgpu::VertexStepMode::Instance, &includeAttribs
		);

		// Done!
		Self { data, includedAttribs, layout, vsEntryPoint, fsEntryPoint }
	}
}
impl GpuDataReceiver for DataReceiver {
	fn gpuData (&self) -> &dyn renderer::GpuData {
		self.data.as_ref()
	}

	/// Custom implementation to also take ignored/included attributes into account
	#[inline]
	fn isCompatible (&self, otherReceiver: &Self) -> bool {
		self.layout == otherReceiver.layout && self.includedAttribs.bits() == otherReceiver.includedAttribs.bits()
	}
}
impl From<Arc<dyn renderer::GpuData+'static>> for DataReceiver {
	#[inline(always)]
	fn from (data: Arc<dyn renderer::GpuData>) -> Self {
		Self::new(data)
	}
}
impl Deref for DataReceiver {
	type Target = dyn renderer::GpuData;

	#[inline(always)]
	fn deref (&self) -> &Self::Target {
		self.data.as_ref()
	}
}

///
pub struct Spheres {
	shader: wgpu::ShaderModule,
	pipelineLayout: wgpu::PipelineLayout,
	styleUniforms: StyleUniformGroup
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
		let styleUniforms = StyleUniformGroup::createAndUpload(
			context, wgpu::ShaderStages::VERTEX_FRAGMENT,
			Some("CGV__renderer_Spheres_styleUniforms").as_deref()
		);
		let pipelineLayout =
			context.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("CGV__renderer_Spheres_renderPipelineLayout"),
				bind_group_layouts: &[
					Some(&renderSetup.bindGroupLayouts().viewing), Some(&styleUniforms.bindGroupLayout)
				],
				immediate_size: 0
			});
		let shader = Self::shaderPackage().createShaderModuleFromBestInstance(
			context.device(), None, Some("CGV__renderer_Spheres_shaderModule")
		).expect("shader module could not be compiled by WGPU");

		// Done!
		Self { shader, pipelineLayout, styleUniforms }
	}

	#[inline(always)]
	pub fn setStyleUniforms <R, Setter: FnOnce(&mut Style)->R> (
		&mut self, context: &Context, setter: Setter
	) -> R {
		self.styleUniforms.update(context, setter)
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
				cull_mode: Some(wgpu::Face::Back),
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

	fn render (
		&self, _: &Context, renderState: &RenderState, renderPass: &mut wgpu::RenderPass, gpuState: &Self::GpuState,
		data: &Self::GpuDataReceiver
	){
		renderPass.set_pipeline(gpuState); // <- in our case it's literally just the pipeline
		renderPass.set_bind_group(0, &renderState.viewingUniforms.bindGroup, &[]);
		renderPass.set_bind_group(1, &self.styleUniforms.bindGroup, &[]);
		let buffers = data.data.geometry();
		for (slot, buffer) in data.layout.bufferIndices().iter().enumerate() {
			renderPass.set_vertex_buffer(slot as u32, buffers[*buffer]);
		}
		renderPass.draw(0..4, 0..data.num());
	}
}
