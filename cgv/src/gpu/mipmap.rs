
//////
//
// Imports
//

// Standard library
//use std::hash::{Hasher, BuildHasher};
use std::sync::LazyLock;

// Arrayvec library
use arrayvec::ArrayVec;

// Dashmap library
use dashmap::DashMap;

// WGPU API
use wgpu;

// Local imports
use crate::*;



//////
//
// Globals
//

/*/// A simple pass-through hasher for u64 ints.
#[derive(Clone,Copy,Default)]
struct U64Hasher {
	value: u64
}
impl Hasher for U64Hasher
{
	#[inline(always)]
	fn finish (&self) -> u64 {
		self.value
	}

	fn write(&mut self, _: &[u8]) {
		unimplemented!()
	}

	#[inline(always)]
	fn write_u64 (&mut self, value: u64) {
		self.value = value;
	}
}
impl BuildHasher for U64Hasher {
	type Hasher = U64Hasher;

	fn build_hasher (&self) -> Self::Hasher {
		U64Hasher::default()
	}
}*/

/// The database of cached compute pipeline configurations
static COMPUTE_PIPELINE_CACHE: LazyLock<DashMap<
	(wgpu::TextureFormat, wgpu::TextureViewDimension, u64),
	Box<ComputePipelineInfo> // TODO: Try without boxing the `ComputePipelineInfo` once we have sufficiently many to test
>> = LazyLock::new(|| {
	DashMap::with_capacity(8)
});



//////
//
// Structs and enums
//

/// Stores a compute pipeline and associated objects the pipeline references, for use by compute shader-based mipmap
/// generators.
pub struct ComputePipelineInfo {
	bindGroupLayout: wgpu::BindGroupLayout,
	pipeline: wgpu::ComputePipeline
}



//////
//
// Traits
//

/// An algorithm for generating the full chain of mipmap levels for a given texture.
pub trait Generator
{
	fn uniqueId () -> u64;

	fn ensureShaderModule (context: &Context) -> Option<wgpu::ShaderModule>;

	fn createPass (encoder: &mut wgpu::CommandEncoder) -> gpu::Pass<'_>;

	fn ensureComputePipeline (
		context: &Context, textureFormat: wgpu::TextureFormat, dimensionality: wgpu::TextureViewDimension
	) -> &ComputePipelineInfo
	{
		let generatorId = Self::uniqueId();
		let query = (textureFormat, dimensionality, generatorId);
		let pipelineInfo = COMPUTE_PIPELINE_CACHE.get(&query);
		if let Some(pipelineInfo) = pipelineInfo {
			// We already have a suitable pipeline for this combination
			let pipelineInfo = util::notsafe::UncheckedRef::new(
				pipelineInfo.value().as_ref()
			);
			unsafe {
				// Safety: - COMPUTE_PIPELINE_CACHE is static, so it may report 'static references
				//         - the values are boxed, so their addresses never change even when iterators are invalidated
				pipelineInfo.as_ref()
			}
		}
		else
		{
			// We need a new pipeline for this combination!
			// Set up bind group layout
			let bindGroupLayoutEntries: [wgpu::BindGroupLayoutEntry; 2] = [
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						sample_type: wgpu::TextureSampleType::Float { filterable: false },
						view_dimension: dimensionality
					},
					count: None
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::StorageTexture {
						access: wgpu::StorageTextureAccess::WriteOnly,
						format: textureFormat,
						view_dimension: dimensionality
					},
					count: None
				}
			];
			let bindGroupLayout = context.device().create_bind_group_layout(
				&wgpu::BindGroupLayoutDescriptor {
					entries: bindGroupLayoutEntries.as_slice(),
					label: None
				}
			);

			// Create pipeline
			// - shader
			let shader =  Self::ensureShaderModule(context).unwrap();
			// - pipeline
			let pipeline = context.device().create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
				module: &shader,
				entry_point: Some("kernel"),
				layout: Some(&context.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					bind_group_layouts: &[&bindGroupLayout],
					push_constant_ranges: &[],
					label: Some("CGV__gpu_mipmapGenComputePipelineLayout")
				})),
				compilation_options: Default::default(),
				cache: None,
				label: Some("CGV__gpu_mipmapGenComputePipeline"),
			});
			let pipelineInfo = Box::new(ComputePipelineInfo { bindGroupLayout, pipeline });
			let pipelineInfo_unchecked = util::notsafe::UncheckedRef::new(
				pipelineInfo.as_ref()
			);
			COMPUTE_PIPELINE_CACHE.insert(query, pipelineInfo);
			unsafe {
				// Safety: - the values are boxed, so their addresses never change, even when we move the newly
				//           constructed items into the cache
				pipelineInfo_unchecked.as_ref()}
		}
	}

	fn perform<'encoder> (&self, context: &Context, pass: &mut gpu::Pass<'encoder>, texture: &mut hal::Texture);

	fn performWithEncoder (&self, context: &Context, encoder: &mut wgpu::CommandEncoder, texture: &mut hal::Texture) {
		let mut pass = Self::createPass(encoder);
		self.perform(context, &mut pass, texture);
	}

	fn performAdhoc (&self, context: &Context, texture: &mut hal::Texture)
	{
		// Create throw-away command encoder
		let mut encoder = context.device().create_command_encoder(
			&wgpu::CommandEncoderDescriptor::default()
		);

		// Encode the mipmapping commands
		self.performWithEncoder(context, &mut encoder, texture);

		// Dispatch
		context.queue().submit([encoder.finish()]);
	}

	fn requiredTextureUsages() -> wgpu::TextureUsages;
}

/// Provider of a reusable compute/shader function implementing a filter strategy for calculating the value of mipmap
/// texels. It provides shader code following the to-be-documented mipmap filter shader protocol.
pub trait ShaderFilter {
	fn uniqueId () -> u32;

	fn provideShader (&self);
}



//////
//
// Classes
//

pub struct BoxFilter;
impl ShaderFilter for BoxFilter {
	fn uniqueId () -> u32 {
		static ID: LazyLock<u32> = LazyLock::new(|| util::unique::uint32());
		*ID
	}

	fn provideShader(&self) {
		todo!()
	}
}

/// An implementation of a mipmap generator that applies a given [shader-based filter](MipmapShaderFilter) to the texels
/// in a compute shader.
pub struct ComputeShaderGenerator<'filter, Filter: ShaderFilter+'filter> {
	_filter: &'filter Filter
}
impl<'filter, Filter: ShaderFilter+'filter> ComputeShaderGenerator<'filter, Filter>
{
	pub fn new (filter: &'filter Filter) -> Self {
		Self { _filter: filter }
	}
}
impl<'filter, Filter: ShaderFilter+'filter> Generator for ComputeShaderGenerator<'filter, Filter>
{
	fn uniqueId () -> u64 {
		static ID: LazyLock<u64> = LazyLock::new(|| util::unique::uint64()<<32);
		let fid = Filter::uniqueId() as u64;
		*ID | fid
	}

	fn ensureShaderModule (context: &Context) -> Option<wgpu::ShaderModule> {
		Some(context.device().create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("CGV__gpu_mipmapGenComputeShaderModule"),
			source: wgpu::ShaderSource::Wgsl(util::sourceFile!("/shader/gpu/mipmapgen.wgsl").into()),
		}))
	}

	fn createPass (encoder: &mut wgpu::CommandEncoder) -> gpu::Pass<'_> {
		gpu::Pass::Compute(encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default()))
	}

	fn perform<'encoder> (
		&self, context: &Context, pass: &mut gpu::Pass<'encoder>, texture: &mut hal::Texture
	){
		// Obtain pipeline suitable for the given texture
		let vdim = texture.mipLevels[0].desc.dimension.expect(
			"Texture mip level views should be created with explicit dimensionality!"
		);
		let pi = Self::ensureComputePipeline(context, texture.descriptor.format, vdim);

		let numMipLevels = texture.mipLevels.len();
		let mut bindGroups = ArrayVec::<_, 32>::new();
		for lvl in 1..numMipLevels
		{
			bindGroups.push((texture.mipLevels[lvl].dims, context.device().create_bind_group(
				&wgpu::BindGroupDescriptor {
					layout: &pi.bindGroupLayout,
					entries: &[
						wgpu::BindGroupEntry {
							binding: 0,
							resource: wgpu::BindingResource::TextureView(&texture.mipLevels[lvl-1].view),
						},
						wgpu::BindGroupEntry {
							binding: 1,
							resource: wgpu::BindingResource::TextureView(&texture.mipLevels[lvl].view),
						}
					],
					label: None
				}
			)));
		}

		// Record compute pass
		// - extract pass reference
		let pass = pass.refCompute();
		// - record compute calls
		pass.set_pipeline(&pi.pipeline);
		for (ref dims, ref bindGroup) in bindGroups {
			pass.set_bind_group(0, bindGroup, &[]);
			let workgroupsX = (dims.x + 8 - 1) / 8;
			let workgroupsY = (dims.y + 8 - 1) / 8;
			pass.dispatch_workgroups(workgroupsX, workgroupsY, 1);
		}
	}

	fn requiredTextureUsages() -> wgpu::TextureUsages {
		wgpu::TextureUsages::STORAGE_BINDING
	}
}

/// An implementation of a mipmap generator that applies a given [shader-based filter](MipmapShaderFilter) to the texels
/// in a *blit*-like application of the classical render pipeline.
pub struct RenderPipelineGenerator<'filter, Filter: ShaderFilter+'filter> {
	_filter: &'filter Filter
}
impl<'filter, Filter: ShaderFilter+'filter> RenderPipelineGenerator<'filter, Filter>
{
	pub fn new (filter: &'filter Filter) -> Self {
		Self { _filter: filter }
	}
}
impl<'filter, Filter: ShaderFilter+'filter> Generator for RenderPipelineGenerator<'filter, Filter>
{
	fn uniqueId () -> u64 {
		static ID: LazyLock<u64> = LazyLock::new(|| util::unique::uint64()<<32);
		let fid = Filter::uniqueId() as u64;
		*ID | fid
	}

	fn ensureShaderModule (_context: &Context) -> Option<wgpu::ShaderModule> {
		None
	}

	fn createPass (encoder: &mut wgpu::CommandEncoder) -> gpu::Pass<'_> {
		gpu::Pass::Render(encoder.begin_render_pass(&wgpu::RenderPassDescriptor::default()))
	}

	fn perform<'encoder> (
		&self, _context: &Context, _pass: &mut gpu::Pass<'encoder>, _texture: &mut hal::Texture
	){
		todo!()
	}

	fn requiredTextureUsages() -> wgpu::TextureUsages {
		wgpu::TextureUsages::RENDER_ATTACHMENT
	}
}
