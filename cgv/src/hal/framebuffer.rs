
//////
//
// Imports
//

// Standard library
use std::hint::unreachable_unchecked;
pub use std::borrow::Borrow;

// Local imports
use crate::*;
use hal::texture;



//////
//
// Structs and enums
//

/// Convenience enum providing for the common use case that a [`Framebuffer`] can either be owned or borrowed from
/// somewhere else (e.g. as a client-provided render target).
pub enum DynamicFramebuffer<'fb> {
	/// Stores an owned [`Framebuffer`].
	Owned(Framebuffer),

	/// References a foreign [`Framebuffer`].
	Borrowed(&'fb Framebuffer),
}
impl<'fb> DynamicFramebuffer<'fb>
{
	/// Evaluates to `true` if the framebuffer is [`Owned`](DynamicFramebuffer::Owned), `false` otherwise.
	pub fn isOwned (&self) -> bool {
		if let DynamicFramebuffer::Owned(_) = self {
			true
		} else {
			false
		}
	}

	/// Evaluates to `true` if the framebuffer is [`Borrowed`](DynamicFramebuffer::Borrowed), `false` otherwise.
	pub fn isBorrowed (&self) -> bool {
		if let DynamicFramebuffer::Borrowed(_) = self {
			true
		} else {
			false
		}
	}
}
impl<'fb> AsRef<Framebuffer> for DynamicFramebuffer<'fb> {
	fn as_ref (&self) -> &Framebuffer {
		match self {
			DynamicFramebuffer::Owned(framebuffer) => framebuffer,
			DynamicFramebuffer::Borrowed(framebuffer) => framebuffer,
		}
	}
}
impl<'fb> Borrow<Framebuffer> for DynamicFramebuffer<'fb>
{
	fn borrow (&self) -> &Framebuffer {
		self.as_ref()
	}
}

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
pub struct Framebuffer {
	pub(in crate::hal::framebuffer) color: Vec<hal::Texture>,
	pub(in crate::hal::framebuffer) depthStencil: Option<hal::Texture>,
	pub(in crate::hal::framebuffer) dims: glm::UVec2
}
impl Framebuffer
{
	/// Query the current dimensions of the framebuffer.
	pub fn dims (&self) -> glm::UVec2 {
		self.dims
	}

	/// Resize the framebuffer, and adjust all its color attachments accordingly.
	pub fn resize (&mut self, context: &Context, newDims: glm::UVec2)
	{
		self.dims = newDims;
		for slot in 0..self.color.len() {
			let old = &self.color[slot];
			self.color[slot] = hal::Texture::createEmptyTexture(
				context, newDims, old.descriptor.format, old.alphaUsage, old.descriptor.usage, old.name.as_deref()
			);
		}
		if let Some(old) = &self.depthStencil {
			self.depthStencil = Some(hal::Texture::createEmptyTexture(
				context, newDims, old.descriptor.format, old.alphaUsage, old.descriptor.usage, old.name.as_deref()
			));
		}
	}

	/// Convenience method to reference the *0*-th color attachment, assuming it exists. Panics if it doesn't.
	///
	/// # Returns
	///
	/// A reference to the texture in the *0*-th color slot.
	pub fn color0 (&self) -> &hal::Texture {
		&self.color[0]
	}

	/// Reference the color attachment in the given *slot*.
	///
	/// # Returns
	///
	/// `Some` reference to the texture in the color slot if it exists, `None` otherwise.
	pub fn color (&self, slot: usize) -> Option<&hal::Texture> {
		(slot < self.color.len()).then_some(&self.color[slot])
	}

	/// Reference the depth/stencil attachment.
	///
	/// # Returns
	///
	/// `Some` reference to the depth/stencil texture if the framebuffer has one, `None` otherwise.
	pub fn depthStencil (&self) -> Option<&hal::Texture> {
		self.depthStencil.as_ref()
	}
}

/// A builder for [`Framebuffer`] instances.
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

	/// Use the specified debugging label if `Some`, otherwise don't use a debugging label.
	pub fn withLabelIfSome (&mut self, label: Option<&'label str>) -> &mut Self {
		self.label = label;
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
	pub fn build (&self, context: &Context) -> Framebuffer
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
				context, self.dims, format, texture::AlphaUsage::DontCare, usages,
				util::concatIfSome(&self.label, &format!("_colorAttachment{slot}")).as_deref()
			));
		}

		// Create depth/stencil attachment, if any
		let depthStencil = self.depthStencil.as_ref().map(|depthStencil| {
			let (format, additionalUsages) =
				if let TextureCreationParams::DepthStencil{
					format, additionalUsages
				} = depthStencil {
					(format, additionalUsages)
				} else {
					unsafe { unreachable_unchecked(); }
				};
			hal::Texture::createDepthStencilTexture(
				context, self.dims, *format, *additionalUsages,
				util::concatIfSome(&self.label, "_depthStencilAttachment").as_deref()
			)
		});

		Framebuffer {
			color, depthStencil, dims: self.dims
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
	match texels {
		hal::ReadBackTexels::U16(texels) => decodeDepthU16(texels[location]),
		hal::ReadBackTexels::U32(texels) => decodeDepthU32(texels[location]),
		hal::ReadBackTexels::F32(texels) => texels[location],
		_ => unreachable!("texel type {:?} cannot contain depth and should not be passed", texels)
	}
}
