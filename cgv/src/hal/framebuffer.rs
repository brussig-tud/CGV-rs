
//////
//
// Imports
//

// Standard library
/* nothing here yet */
use std::hint::unreachable_unchecked;
// Local imports
use crate::*;



//////
//
// Structs and enums
//

/// High-level enum encompassing all supported formats for depth/stencil buffers.
#[derive(Clone, Copy, Default, Debug)]
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
impl From<DepthStencilFormat> for wgpu::TextureFormat
{
	#[inline(always)]
	fn from (format: DepthStencilFormat) -> Self {
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
	#[inline(always)]
	fn from (format: &DepthStencilFormat) -> Self { (*format).into() }
}
impl From<wgpu::TextureFormat> for DepthStencilFormat
{
	#[inline(always)]
	fn from (format: wgpu::TextureFormat) -> Self {
		match format {
			wgpu::TextureFormat::Depth16Unorm => DepthStencilFormat::D16,
			wgpu::TextureFormat::Depth24Plus => DepthStencilFormat::D24,
			wgpu::TextureFormat::Depth32Float => DepthStencilFormat::D32,
			wgpu::TextureFormat::Depth24PlusStencil8 => DepthStencilFormat::D24S8,
			wgpu::TextureFormat::Depth32FloatStencil8 => DepthStencilFormat::D32S8,
			_ => unreachable!("wgpu format {:?} is no depth/stencil format!", format)
		}
	}
}
impl From<&wgpu::TextureFormat> for DepthStencilFormat {
	#[inline(always)]
	fn from (format: &wgpu::TextureFormat) -> Self { (*format).into() }
}

// Small helper enum to store the different arguments for the hal::Texture::create...() methods
enum TextureCreationParams {
	Color{format: wgpu::TextureFormat, usages: wgpu::TextureUsages},
	DepthStencil{format: DepthStencilFormat, additionalUsages: Option<wgpu::TextureUsages>}
}



//////
//
// Classes
//

/// A logical render target consisting of any number of (including zero) color textures and optionally a depth/stencil
/// texture.
#[derive(Default)]
pub struct Framebuffer<'label> {
	pub(in crate::hal::framebuffer) label: Option<&'label str>,
	pub(in crate::hal::framebuffer) color: Vec<hal::Texture>,
	pub(in crate::hal::framebuffer) depthStencil: Option<hal::Texture>,
	pub(in crate::hal::framebuffer) dims: glm::UVec2
}
impl<'label> Framebuffer<'label>
{
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

	/// Query the current dimensions of the framebuffer.
	pub fn dims (&self) -> glm::UVec2 {
		self.dims
	}

	/// Resize the framebuffer, and adjust all its color attachments accordingly.
	pub fn resize (&mut self, context: &Context, newDims: &glm::UVec2)
	{
		self.dims = *newDims;
		for slot in 0..self.color.len() {
			let old = &self.color[slot];
			self.color[slot] = hal::Texture::createEmptyTexture(
				context, newDims, old.descriptor.format, old.descriptor.usage, old.name.as_deref()
			);
		}
		if let Some(old) = &self.depthStencil {
			self.depthStencil = Some(hal::Texture::createEmptyTexture(
				context, newDims, old.descriptor.format, old.descriptor.usage, old.name.as_deref()
			));
		}
	}
}

pub struct FramebufferBuilder<'label>
{
	label: Option<&'label str>,
	color: Vec<TextureCreationParams>,
	depthStencil: Option<TextureCreationParams>,
	dims: glm::UVec2,
}
impl<'label> FramebufferBuilder<'label>
{
	/// Initialize the builder for a framebuffer of the given dimensions.
	pub fn withDims (dims: &glm::UVec2) -> Self { Self {
		label: None,   color: Vec::default(),   depthStencil: None,   dims: *dims
	}}

	/// Use the specified debugging label.
	pub fn withLabel (&mut self, label: &'label str) -> &mut Self {
		self.label = Some(label);
		self
	}

	/// Add a color attachment of the given format, with the given additional usages.
	pub fn attachColor (&mut self, format: wgpu::TextureFormat, additionalUsages: Option<wgpu::TextureUsages>) -> &mut Self
	{
		self.color.push(TextureCreationParams::Color {
			format, usages: if let Some(additionalUsages) = additionalUsages {
				wgpu::TextureUsages::RENDER_ATTACHMENT | additionalUsages
			} else {
				wgpu::TextureUsages::RENDER_ATTACHMENT
			}
		});
		self
	}

	/// Attach a depth/stencil texture of the given format, with the given additional usages. Subsequent calls will
	/// replace the current to-be-built depth/stencil attachment, if any.
	pub fn attachDepthStencil (&mut self, format: DepthStencilFormat, additionalUsages: Option<wgpu::TextureUsages>)
		-> &mut Self
	{
		self.depthStencil = Some(TextureCreationParams::DepthStencil { format, additionalUsages });
		self
	}

	/// Build the framebuffer as configured.
	pub fn build (&self, context: &Context) -> Framebuffer<'label>
	{
		// Create color attachments, if any
		let mut color: Vec<hal::Texture> = Vec::with_capacity(self.color.len());
		for slot in 0..self.color.len() {
			let (format, usages) =
				if let TextureCreationParams::Color{format, usages} = self.color[slot] {
					(format, usages)
				} else {
					unsafe { unreachable_unchecked(); }
				};
			color.push(hal::Texture::createEmptyTexture(
				context, &self.dims, format, usages,
				util::concatIfSome(&self.label, &format!("_colorAttachment{slot}")).as_deref()
			));
		}

		// Create depth/stencil attachment, if any
		let depthStencil = if let Some(depthStencil) = &self.depthStencil {
			let (format, additionalUsages) =
				if let TextureCreationParams::DepthStencil{
					format, additionalUsages
				} = depthStencil {
					(format, additionalUsages)
				} else {
					unsafe { unreachable_unchecked(); }
				};
			Some(hal::Texture::createDepthStencilTexture(
				context, &self.dims, *format, *additionalUsages,
				util::concatIfSome(&self.label, &format!("_depthStencilAttachment")).as_deref()
			))
		} else {
			None
		};

		Framebuffer {
			label: self.label, color, depthStencil, dims: self.dims
		}
	}
}



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
