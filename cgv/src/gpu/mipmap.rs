
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Local imports
use crate::hal;



//////
//
// Traits
//

/// An algorithm for generating the full chain of mipmap levels for any given texture.
pub trait MipmapGenerator {
	fn perform (&self, texture: &mut hal::Texture);
}

/// Provider of a reusable compute/shader function implementing a filter strategy for calculating the value of mipmap
/// texels. It provides shader code following the to-be-documented mipmap filter shader protocol.
pub trait MipmapShaderFilter {
	fn provideShader (&self);
}



//////
//
// Classes
//

pub struct BoxFilter;
impl MipmapShaderFilter for BoxFilter {
	fn provideShader(&self) {
		todo!()
	}
}

/// An implementation of a mipmap generator that applies a given [shader-based filter](MipmapShaderFilter) to the texels
/// in a compute shader.
pub struct ComputeShaderMipMapAlgorithm<'filter, Filter: MipmapShaderFilter+'filter> {
	_filter: &'filter Filter
}
impl<'filter, Filter: MipmapShaderFilter+'filter> ComputeShaderMipMapAlgorithm<'filter, Filter>
{
	pub fn new (filter: &'filter Filter) -> Self {
		Self { _filter: filter }
	}
}
impl<'filter, Filter: MipmapShaderFilter+'filter> MipmapGenerator for ComputeShaderMipMapAlgorithm<'filter, Filter>
{
	fn perform (&self, _texture: &mut hal::Texture) {
		todo!()
	}
}

/// An implementation of a mipmap generator that applies a given [shader-based filter](MipmapShaderFilter) to the texels
/// in a *blit*-like application of the classical render pipeline.
pub struct RenderPipelineMipMapAlgorithm<'filter, Filter: MipmapShaderFilter+'filter> {
	_filter: &'filter Filter
}
impl<'filter, Filter: MipmapShaderFilter+'filter> RenderPipelineMipMapAlgorithm<'filter, Filter>
{
	pub fn new (filter: &'filter Filter) -> Self {
		Self { _filter: filter }
	}
}
impl<'filter, Filter: MipmapShaderFilter+'filter> MipmapGenerator for RenderPipelineMipMapAlgorithm<'filter, Filter>
{
	fn perform (&self, _texture: &mut hal::Texture) {
		todo!()
	}
}
