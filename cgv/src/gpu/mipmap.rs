
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

/// The database of cached compute pipeline configurations
static COMPUTE_PIPELINE_CACHE: LazyLock<DashMap<
	(wgpu::TextureFormat, MipmappableTextureShape, u64),
	Box<ComputePipelineInfo> // TODO: Try without boxing the `ComputePipelineInfo` once we have sufficiently many to test
>> = LazyLock::new(|| {
	DashMap::with_capacity(8)
});



//////
//
// Structs and enums
//

///
#[derive(Copy,Clone,Debug,Hash,Eq,PartialEq)]
pub enum MipmappableTextureShape {
	/// Corresponds to `wgpu::TextureViewDimension::D2`
	D2,

	/// Corresponds to `wgpu::TextureViewDimension::D2Array`
	D2Array,

	/// Corresponds to `wgpu::TextureViewDimension::Cube`
	Cube,

	/// Corresponds to `wgpu::TextureViewDimension::CubeArray`
	CubeArray,

	/// Corresponds to `wgpu::TextureViewDimension::D3`
	D3
}
impl MipmappableTextureShape
{
	#[inline]
	pub fn from (viewDimensionality: wgpu::TextureViewDimension) -> Option<Self>
	{
		match viewDimensionality {
			wgpu::TextureViewDimension::D1 => None,
			wgpu::TextureViewDimension::D2 => Some(MipmappableTextureShape::D2),
			wgpu::TextureViewDimension::D2Array => Some(MipmappableTextureShape::D2Array),
			wgpu::TextureViewDimension::Cube => Some(MipmappableTextureShape::Cube),
			wgpu::TextureViewDimension::CubeArray => Some(MipmappableTextureShape::CubeArray),
			wgpu::TextureViewDimension::D3 => Some(MipmappableTextureShape::D3)
		}
	}
}
impl From<MipmappableTextureShape> for wgpu::TextureViewDimension
{
	#[inline]
	fn from (mipmappableTextureShape: MipmappableTextureShape) -> Self
	{
		match mipmappableTextureShape {
			MipmappableTextureShape::D2 => wgpu::TextureViewDimension::D2,
			MipmappableTextureShape::D2Array => wgpu::TextureViewDimension::D2Array,
			MipmappableTextureShape::Cube => wgpu::TextureViewDimension::Cube,
			MipmappableTextureShape::CubeArray => wgpu::TextureViewDimension::CubeArray,
			MipmappableTextureShape::D3 => wgpu::TextureViewDimension::D3
		}
	}
}

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
	fn uniqueId (&self) -> u64;

	fn ensureShaderModule (&self, context: &Context, textureShape: MipmappableTextureShape)
		-> Option<(wgpu::ShaderModule, Option<&str>)>;

	fn createPass<'outer> (&'outer self, encoder: &'outer mut wgpu::CommandEncoder) -> gpu::Pass<'outer>;

	fn ensureComputePipeline (
		&self, context: &Context, textureFormat: wgpu::TextureFormat, textureShape: MipmappableTextureShape
	) -> &ComputePipelineInfo
	{
		let generatorId = self.uniqueId();
		let query = (textureFormat, textureShape, generatorId);
		let pipelineInfo = COMPUTE_PIPELINE_CACHE.get(&query);
		if let Some(pipelineInfo) = pipelineInfo {
			// We already have a suitable pipeline for this combination
			let pipelineInfo = util::notsafe::UncheckedRef::new(
				pipelineInfo.value().as_ref()
			);
			unsafe {
				// SAFETY: - COMPUTE_PIPELINE_CACHE is static, so it is allowed to report 'static references
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
						view_dimension: textureShape.into()
					},
					count: None
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::StorageTexture {
						access: wgpu::StorageTextureAccess::WriteOnly,
						format: textureFormat,
						view_dimension: textureShape.into()
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
			let (shader, specificEntryPoint) = self.ensureShaderModule(
				context, textureShape
			).unwrap();
			// - pipeline
			let pipeline = context.device().create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
				module: &shader,
				entry_point: specificEntryPoint,
				layout: Some(&context.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					bind_group_layouts: &[Some(&bindGroupLayout)],
					label: Some("CGV__gpu_mipmapGenComputePipelineLayout"),
					immediate_size: 0
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
				// SAFETY: - the values are boxed, so their addresses never change, even when we move the newly
				//           constructed items into the cache
				pipelineInfo_unchecked.as_ref()}
		}
	}

	fn perform<'encoder> (&self, context: &Context, pass: &mut gpu::Pass<'encoder>, texture: &mut hal::Texture);

	fn performWithEncoder (&self, context: &Context, encoder: &mut wgpu::CommandEncoder, texture: &mut hal::Texture) {
		let mut pass = self.createPass(encoder);
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
	///
	fn uniqueId () -> u32;

	///
	fn provideShader (&self, context: &Context, textureShape: MipmappableTextureShape)
		-> Option<(wgpu::ShaderModule, Option<&str>)>;
}



//////
//
// Classes
//

pub struct PolyphaseBoxFilter;
impl ShaderFilter for PolyphaseBoxFilter {
	fn uniqueId () -> u32 {
		static ID: LazyLock<u32> = LazyLock::new(|| util::unique::uint32());
		*ID
	}

	fn provideShader (&self, context: &Context, textureShape: MipmappableTextureShape) -> Option<(
		wgpu::ShaderModule, Option<&str>
	)>{
		let shaderPackage = shader::Package::deserialize(
			util::sourceGeneratedBytes!("/shader/gpu/mipmapgen/box_polyphase.spk")
		).ok()?;
		use MipmappableTextureShape::*;
		shaderPackage.createShaderModuleFromBestInstance(
			context.device(), None, Some("CGV__gpu_mipmapGenComputeShaderModule")
		).map(
			|sm| (sm, /* entryPointName: */Some(match textureShape {
				D2 => "boxPolyphase2D",
				D3 => "boxPolyphase3D",
				D2Array | Cube | CubeArray => unimplemented!(
					"Polyphase box filter is not yet implemented for cube and/or array textures!"
				)
			}))
		)
	}
}

/// An implementation of a mipmap generator that applies a given [shader-based filter](MipmapShaderFilter) to the texels
/// in a compute shader.
pub struct ComputeShaderGenerator<'filter, Filter: ShaderFilter+'filter> {
	filter: &'filter Filter
}
impl<'filter, Filter: ShaderFilter+'filter> ComputeShaderGenerator<'filter, Filter>
{
	pub fn new (filter: &'filter Filter) -> Self { Self {
		filter
	}}
}
impl<'filter, Filter: ShaderFilter+'filter> Generator for ComputeShaderGenerator<'filter, Filter>
{
	fn uniqueId (&self) -> u64 {
		static ID: LazyLock<u64> = LazyLock::new(|| util::unique::uint64()<<32);
		let fid = Filter::uniqueId() as u64;
		*ID | fid
	}

	fn ensureShaderModule (&self, context: &Context, textureShape: MipmappableTextureShape)
		-> Option<(wgpu::ShaderModule, Option<&str>)>
	{
		self.filter.provideShader(context, textureShape)
	}

	fn createPass<'outer> (&'outer self, encoder: &'outer mut wgpu::CommandEncoder) -> gpu::Pass<'outer> {
		gpu::Pass::Compute(encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default()))
	}

	fn perform<'encoder> (
		&self, context: &Context, pass: &mut gpu::Pass<'encoder>, texture: &mut hal::Texture
	){
		// Obtain pipeline suitable for the given texture
		let vdim = texture.mipLevels[0].desc.dimension.expect(
			"Texture mip level views should be created with explicit dimensionality!"
		);
		let pi = self.ensureComputePipeline(
			context, texture.descriptor.format, MipmappableTextureShape::from(vdim).expect(
				"Mipmap generation must be performed on a texture with mip-mappable shape!"
			)
		);

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
///
/// # TODO
///
/// This is still a stub, actually implement it!
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
	fn uniqueId (&self) -> u64 {
		static ID: LazyLock<u64> = LazyLock::new(|| util::unique::uint64()<<32);
		let fid = Filter::uniqueId() as u64;
		*ID | fid
	}

	fn ensureShaderModule (&self, _context: &Context, _textureShape: MipmappableTextureShape) -> Option<(
		wgpu::ShaderModule, Option<&str>
	)>{
		None
	}

	fn createPass<'outer> (&'outer self, encoder: &'outer mut wgpu::CommandEncoder) -> gpu::Pass<'outer> {
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
