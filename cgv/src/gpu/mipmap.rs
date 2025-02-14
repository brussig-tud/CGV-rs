
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// WGPU API
use wgpu;

// Local imports
use crate::*;
use crate::hal;



//////
//
// Traits
//

/// An algorithm for generating the full chain of mipmap levels for any given texture.
pub trait Generator {
	fn perform (&self, context: &Context, texture: &mut hal::Texture);
	fn requiredTextureUsages() -> wgpu::TextureUsages;
}

/// Provider of a reusable compute/shader function implementing a filter strategy for calculating the value of mipmap
/// texels. It provides shader code following the to-be-documented mipmap filter shader protocol.
pub trait ShaderFilter {
	fn provideShader (&self);
}



//////
//
// Classes
//

pub struct BoxFilter;
impl ShaderFilter for BoxFilter {
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
	fn perform (&self, context: &Context, texture: &mut hal::Texture)
	{
		// Prepare bind group layout
		let texBindGroupLayoutEntries: [wgpu::BindGroupLayoutEntry; 2] = [
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Texture {
					multisampled: false,
					sample_type: wgpu::TextureSampleType::Float { filterable: false },
					view_dimension: wgpu::TextureViewDimension::D2
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry {
				binding: 1,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::StorageTexture {
					access: wgpu::StorageTextureAccess::WriteOnly,
					format: texture.descriptor.format,
					view_dimension: wgpu::TextureViewDimension::D2,
				},
				count: None,
			}
		];
		let texBindGroupLayout = context.device().create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				entries: texBindGroupLayoutEntries.as_slice(),
				label: Some("Example__TestBindGroupLayout"),
			}
		);
		let _texBindGroup = context.device().create_bind_group(
			&wgpu::BindGroupDescriptor {
				layout: &texBindGroupLayout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&texture.view()),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&texture.sampler),
					}
				],
				label: Some("Example__TestBindGroup"),
			}
		);
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
	fn perform (&self, _context: &Context, _texture: &mut hal::Texture) {
		todo!()
	}
	fn requiredTextureUsages() -> wgpu::TextureUsages {
		wgpu::TextureUsages::RENDER_ATTACHMENT
	}
}
