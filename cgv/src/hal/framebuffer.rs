
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Local imports
use crate::*;



//////
//
// Structs and enums
//

/// High-level enum encompassing all supported formats for depth/stencil buffers.
#[derive(Clone, Copy, Default)]
pub enum DepthStencilFormat
{
	/// 16-bits integer.
	D16,

	/// 24-bits integer.
	D24,

	/// 32-bits floating point.
	#[default]
	D32,

	/// 24-bits integer depth + 8-bits stencil.
	D24S8,

	/// 32-bits floating point depth + 8-bits stencil (requires feature support).
	D32S8
}
impl From<DepthStencilFormat> for wgpu::TextureFormat {
	fn from(format: DepthStencilFormat) -> Self {
		match format {
			DepthStencilFormat::D16 => wgpu::TextureFormat::Depth16Unorm,
			DepthStencilFormat::D24 => wgpu::TextureFormat::Depth24Plus,
			DepthStencilFormat::D32 => wgpu::TextureFormat::Depth32Float,
			DepthStencilFormat::D24S8 => wgpu::TextureFormat::Depth24PlusStencil8,
			DepthStencilFormat::D32S8 => wgpu::TextureFormat::Depth32FloatStencil8
		}
	}
}
impl From<&DepthStencilFormat> for wgpu::TextureFormat {
	fn from(format: &DepthStencilFormat) -> Self { (*format).into() }
}
impl From<wgpu::TextureFormat> for DepthStencilFormat {
	fn from(format: wgpu::TextureFormat) -> Self {
		match format {
			wgpu::TextureFormat::Depth16Unorm => DepthStencilFormat::D16,
			wgpu::TextureFormat::Depth24Plus => DepthStencilFormat::D24,
			wgpu::TextureFormat::Depth32Float => DepthStencilFormat::D32,
			wgpu::TextureFormat::Depth24PlusStencil8 => DepthStencilFormat::D24S8,
			wgpu::TextureFormat::Depth32FloatStencil8 => DepthStencilFormat::D32S8,
			_ => panic!("cannot convert unsupported format \"{:?}\" into cgv::hal::DepthStencilFormat!", format)
		}
	}
}
impl From<&wgpu::TextureFormat> for DepthStencilFormat {
	fn from(format: &wgpu::TextureFormat) -> Self { (*format).into() }
}



//////
//
// Classes
//

/// A logical render target attaching to any number of color and optionally a depth/stencil texture.
#[derive(Default)]
pub struct Framebuffer<'a>{
	color: [Option<&'a hal::Texture>; 8],
	depth: Option<&'a hal::Texture>,
	owned: [Option<hal::Texture>; 9]
}
impl<'a> Framebuffer<'a> {
	/*pub fn new (
		context: &Context, dims: &glm::UVec2, colorFormat: wgpu::TextureFormat,
		depthStencilFormat: hal::DepthStencilFormat, label: &str
	) -> Self
	{
		let colorLabel = format!("{label}_colorTarget");
		let depthLabel = format!("{label}_depthStencilTarget");
		Self {
			color: hal::Texture::createEmptyTexture(
				context, dims, colorFormat, wgpu::TextureUsages::RENDER_ATTACHMENT,
				Some(colorLabel.as_str())
			),
			depth: hal::Texture::createDepthStencilTexture(
				context, dims, depthStencilFormat, Some(wgpu::TextureUsages::COPY_SRC), Some(depthLabel.as_str())
			)
		}
	}*/
}

/// A builder for [framebuffers](Framebuffer).
pub struct FramebufferBuilder {}



//////
//
// Functions
//

/// t.b.d.
pub fn decodeDepthU16 (_value: u16) -> f32 {
	unimplemented!("internal representation of 16-bit integer depth is as of yet unknown");
}

/// t.b.d.
pub fn decodeDepthU32 (_value: u32) -> f32 {
	unimplemented!("internal representation of 24-bit integer depth with or without stencil is as of yet unknown");
}

/// t.b.d.
pub fn decodeDepth (location: usize, texels: hal::ReadBackTexels) -> f32
{
	match texels
	{
		hal::ReadBackTexels::U16(texels) => decodeDepthU16(texels[location]),
		hal::ReadBackTexels::U32(texels) => decodeDepthU32(texels[location]),
		hal::ReadBackTexels::F32(texels) => texels[location],
		_ => unreachable!("texel type {:?} cannot contain depth and should not be passed", texels)
	}
}
