
//////
//
// Imports
//

// Standard library
use std::sync::LazyLock;

// WGPU API
use crate::wgpu;

// Image library
use image::GenericImageView;

// Local imports
use crate::*;
use util::math::alignToFactor;



//////
//
// Structs and enums
//

/// Encapsulates the logical and real GPU-side physical size (including padding for alignment) of a texture
#[derive(Copy, Clone)]
pub struct TextureSize {
	pub logical: usize,
	pub actual: usize
}

/// Encapsulates per-mip level data that [`Texture`] needs to store.
pub struct MipLevel<'labels> {
	pub dims: glm::UVec3,
	pub size: TextureSize,
	pub desc: wgpu::TextureViewDescriptor<'labels>,
	pub view: wgpu::TextureView,
}

/// Encapsulates the slice of texels provided during [texture readback](Texture::readback).
#[derive(Debug)]
pub enum ReadBackTexels<'a> {
	U16(&'a [u16]),
	U32(&'a [u32]),
	U64(&'a [u64]),
	F32(&'a [f32])
}

/// Indicates the correct usage of texture alpha values for blending
#[derive(Debug,Clone,Copy)]
pub enum AlphaUsage {
	DontCare, Straight, PreMultiplied
}

/// The default mipmap generator that will be referenced by [`defaultMipmapping()`].
static DEFAULT_MIPMAP_GENERATOR: LazyLock<
	gpu::mipmap::ComputeShaderGenerator<gpu::mipmap::BoxFilter>
> = LazyLock::new(|| gpu::mipmap::ComputeShaderGenerator::new(&gpu::mipmap::BoxFilter{}));

/// A `None` constant for the `Option` type used in [`Texture::fromImage()`] and related methods. For a reasonable default
/// choice of `Some` [mipmap generator](gpu::mipmap::Generator), use [`defaultMipmapping()`].
pub const NO_MIPMAPS: Option<&NoopMipmapGenerator> = None;



//////
//
// Classes
//

/// A stub implementation of a [`gpu::mipmap::MipmapGenerator`] that does nothing. It's sole reason for existance is to
/// serve as the type parameter of [`NO_MIPMAPS`]'s `Option` type, such that automatic type inference is possible when
/// calling [`Texture::fromImage`] and related functions with mipmap generation disabled.
pub struct NoopMipmapGenerator;
impl gpu::mipmap::Generator for NoopMipmapGenerator
{
	fn uniqueId () -> u64 {
		u64::MAX
	}

	fn ensureShaderModule (_: &Context) -> Option<wgpu::ShaderModule> {
		None
	}

	fn createPass (_: &mut wgpu::CommandEncoder) -> gpu::Pass {
		panic!("The NoopMipmapGenerator is a dummy and cannot create any actual GPU passes!");
	}

	fn perform<'encoder> (&self, _: &Context, _: &mut gpu::Pass<'encoder>, _: &mut hal::Texture) {
		panic!("Attempting to use NoopMipmapGenerator for actual mipmap generation!");
	}

	fn performWithEncoder (&self, _: &Context, _: &mut wgpu::CommandEncoder, _: &mut Texture) {
		panic!("Attempting to use NoopMipmapGenerator for actual mipmap generation!");
	}

	fn requiredTextureUsages() -> wgpu::TextureUsages {
		wgpu::TextureUsages::empty()
	}
}


/// Represents a texture object, its data and interface to that data.
#[allow(unused)]
pub struct Texture {
	/// The name (if any) of the texture object.
	pub name: Option<String>,

	/// The device texture object.
	pub texture: Box<wgpu::Texture>,

	/// The descriptor used to create the [texture object](texture).
	pub descriptor: wgpu::TextureDescriptor<'static>,

	/// How to interpret the alpha channel (if any) when blending.
	pub alphaUsage: AlphaUsage,

	/// The view on the whole texture, including mipmaps if there are any.
	pub view: wgpu::TextureView,

	/// The texture views for interfacing with the individual mipmap levels of the texture object. If you just want to
	/// access the level-0 mipmap (original image), you can use the convenience method [`Texture::view`].
	pub mipLevels: Vec<MipLevel<'static>>,

	/// The buffer object for readback operations in case the texture usage allows for that
	pub readbackBuffer: Option<Box<wgpu::Buffer>>,

	/// The TexelCopyTextureInfo-compatible view on the texture in case readback is enabled
	pub readbackView_tex: Option<wgpu::TexelCopyTextureInfo<'static>>,

	/// The TexelCopyBufferInfo-compatible view on the texture in case readback is enabled
	pub readbackView_buf: Option<wgpu::TexelCopyBufferInfo<'static>>,
}

impl Texture
{
	/// Create a generic uninitialized texture of arbitrary format.
	///
	/// # Arguments
	///
	/// * `context` – The *CGV-rs* context under which to create the texture.
	/// * `dims` – The desired dimensions in terms of width, height and depth (or layers).
	/// * `format` – The desired format of the texture.
	/// * `numMipLevels` – How many mipmap levels to create for the texture. Setting this to zero or a value greater
	///    than the number calculated by [`numMipLevels`] for the given `dims` will result in undefined behavior.
	///    **TODO: take explicit chain of miplevel descriptors instead**
	/// * `alphaUsage` – How the alpha channel of the texture (if any) should be used when blending.
	/// * `usageFlags` – The set of [texture usages](wgpu::TextureUsages) the texture is intended for.
	/// * `label` – The string to internally label the GPU-side texture object with.
	pub fn createEmpty (
		context: &Context, dims: glm::UVec3, format: wgpu::TextureFormat, numMipLevels: u32, alphaUsage: AlphaUsage,
		usageFlags: wgpu::TextureUsages, label: Option<&str>
	) -> Self
	{
		// Store name in owned memory
		let name = label.map(String::from);
		let label = if let Some(name) = &name {
			Some(util::statify(name.as_str()))
		} else {
			None
		};

		// Create texture object
		let descriptor = wgpu::TextureDescriptor {
			format,	label, size: wgpu::Extent3d {width: dims.x, height: dims.y, depth_or_array_layers: dims.z},
			mip_level_count: numMipLevels,
			sample_count: 1,
			dimension: textureDimensionsFromVec(&dims),
			usage: usageFlags,
			view_formats: &[],
		};
		let texture = Box::new(context.device().create_texture(&descriptor));

		// Create main view on the texture
		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

		// Create views on the texture surfaces of each mip level
		let mut mipLevels = Vec::with_capacity(numMipLevels as usize);
		for lvl in 0..numMipLevels
		{
			let mipDims = {
				let pow = f32::powi(0.5, lvl as i32);
				glm::vec3(
					(f32::floor(dims.x as f32 * pow) as u32).max(1),
					(f32::floor(dims.y as f32 * pow) as u32).max(1),
					(f32::floor(dims.z as f32 * pow) as u32).max(1),
				)
			};
			let size = {
				let logicalBytesPerRow = numBytesFromFormat(descriptor.format) * mipDims.x as usize;
				let heightTimesDepth = (mipDims.y * mipDims.z) as usize;
				TextureSize {
					logical: logicalBytesPerRow * heightTimesDepth,
					actual: heightTimesDepth * alignToFactor(
						logicalBytesPerRow, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize
					)
				}
			};
			let desc = wgpu::TextureViewDescriptor {
				//format: None,
				dimension: Some(textureViewDimensionsEquiv(descriptor.dimension)),
				//usage: None,
				base_mip_level: lvl,
				mip_level_count: Some(1),
				..Default::default()
			};
			let view = texture.create_view(&desc);
			mipLevels.push(MipLevel { dims: mipDims, size, desc, view });
		}
		let size = &mipLevels[0].size;
		let readbackBuffer = usageFlags.contains(wgpu::TextureUsages::COPY_SRC).then(||
			Box::new(context.device().create_buffer(&wgpu::BufferDescriptor {
				label: util::concatIfSome(&label, "_readbackBuf").as_deref(),
				size: size.actual as u64,
				usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
				mapped_at_creation: false
			}))
		);
		let readbackView_tex = match &readbackBuffer {
			Some(_) => Some(wgpu::TexelCopyTextureInfo {
				texture: util::statify(texture.as_ref()),
				mip_level: 0,
				origin: Default::default(),
				aspect: wgpu::TextureAspect::DepthOnly,
			}),
			_ => None
		};
		let readbackView_buf = match &readbackBuffer {
			Some(buffer) => Some(wgpu::TexelCopyBufferInfo {
				buffer: util::statify(buffer.as_ref()),
				layout: wgpu::TexelCopyBufferLayout {
					bytes_per_row: Some(alignToFactor(
						descriptor.size.width * numBytesFromFormat(descriptor.format) as u32,
						wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
					)),
					..Default::default()
				}
			}),
			_ => None
		};

		// Done!
		Self {
			name, texture, descriptor, alphaUsage, view, mipLevels, readbackBuffer, readbackView_tex, readbackView_buf
		}
	}

	/// Create the texture from the given blob, which should contain the bytes of a valid image format.
	///
	/// # Arguments
	///
	/// * `context` – The *CGV-rs* context under which to create the texture.
	/// * `blob` – The memory slice containing the raw bytes making up the image.
	/// * `alphaUsage` – How the alpha channel of the texture (if any) should be used when blending.
	/// * `specialUsageFlags` – An optional set of [texture usage flags](wgpu::TextureUsages) to add on to the minimum
	///    required usages for creating a texture from host-data (currently, only [`wgpu::TextureUsages::COPY_DST`]).
	///    Note that making use of automatic `mipmapGeneration` may enforce additional usages depending on the chosen
	///    [`gpu::mipmap::MipmapGenerator`].
	/// * `mipmapGeneration` – If automatic mipmap generation is desired, which [`gpu::mipmap::MipmapGenerator`] to use.
	       #[doc=include_str!("_doc/_texture_defaultMipmapping.md")]
	/// * `label` – An optional name to internally label the GPU-side texture object with.
	///
	/// # Returns
	///
	/// The fully constructed texture object containing the image encoded in the blob (and its mipmaps if requested) if
	/// the bytes could be successfully interpreted as such, or some [`image::ImageError`] if there were problems
	/// decoding the blob into an image.
	pub fn fromBlob<MipmapGenerator: gpu::mipmap::Generator> (
		context: &Context, blob: &[u8], alphaUsage: AlphaUsage, specialUsageFlags: Option<wgpu::TextureUsages>,
		mipmapGeneration: Option<&MipmapGenerator>, label: Option<&str>
	) -> Result<Self> {
		let img = image::load_from_memory(blob)?;
		Ok(Self::fromImage(context, &img, alphaUsage, specialUsageFlags, mipmapGeneration, label))
	}

	/// Create the texture from the given image.
	///
	/// # Arguments
	///
	/// * `context` – The *CGV-rs* context under which to create the texture.
	/// * `image` – The image the texture should contain.
	/// * `alphaUsage` – How the alpha channel of the texture (if any) should be used when blending.
	/// * `specialUsageFlags` – An optional set of [texture usage flags](wgpu::TextureUsages) to add on to the minimum
	///    required usages for creating a texture from host-data (currently, only [`wgpu::TextureUsages::COPY_DST`]).
	///    Note that making use of automatic `mipmapGeneration` may enforce additional usages depending on the chosen
	///    [`gpu::mipmap::MipmapGenerator`].
	/// * `mipmapGeneration` – If automatic mipmap generation is desired, which [`gpu::mipmap::MipmapGenerator`] to use.
	       #[doc=include_str!("_doc/_texture_defaultMipmapping.md")]
	/// * `label` – The string to internally label the GPU-side texture object with.
	///
	/// # Returns
	///
	/// The fully constructed texture object containing the image (and its mipmaps if requested).
	pub fn fromImage<MipmapGenerator: gpu::mipmap::Generator> (
		context: &Context, image: &image::DynamicImage, alphaUsage: AlphaUsage,
		specialUsageFlags: Option<wgpu::TextureUsages>, mipmapGeneration: Option<&MipmapGenerator>,
		label: Option<&str>
	) -> Self
	{
		// Compile usage flags
		let usageFlags = if let Some(usages) = specialUsageFlags {
			wgpu::TextureUsages::COPY_DST | usages
		} else {
			wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
		} | mipmapGeneration.map_or(
			wgpu::TextureUsages::empty(), |_| MipmapGenerator::requiredTextureUsages()
		);

		// Infer texture parameters from image meta information
		let (dims, size) = {
			let res = image.dimensions();
			let dims = glm::vec3(res.0, res.1, 1);
			(dims, wgpu::Extent3d {width: dims.x, height: dims.y, depth_or_array_layers: dims.z})
		};
		let mipmapLevels = if mipmapGeneration.is_some() { numMipLevels(&dims) }
		                         else                          { 1 };

		// Create actual texture
		let mut texture = Self::createEmpty(
			context, dims, wgpu::TextureFormat::Rgba8Unorm, mipmapLevels, alphaUsage, usageFlags, label
		);

		// Upload image data
		context.queue().write_texture(
			wgpu::TexelCopyTextureInfo {
				aspect: wgpu::TextureAspect::All,
				texture: &texture.texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
			},
			&image.to_rgba8(),
			wgpu::TexelCopyBufferLayout {
				offset: 0,
				bytes_per_row: Some(4 * size.width),
				rows_per_image: Some(size.height),
			},
			size,
		);
		context.queue().submit([]); // make sure the texture transfer starts immediately

		// Generate mipmaps if requested
		if let Some(generator) = mipmapGeneration {
			generator.performAdhoc(context, &mut texture);
		}

		// Done!
		texture
	}

	/// Create an uninitialized texture suitable for use as a depth/stencil attachment.
	///
	/// # Arguments
	///
	/// * `context` – The *CGV-rs* context under which to create the texture.
	/// * `dims` – The desired dimensions in terms of width and height.
	/// * `format` – The desired depth/stencil format of the texture.
	/// * `specialUsageFlags` – An optional set of [texture usage flags](wgpu::TextureUsages) to add on to the minimum
	///    required usages for creating a texture from host-data (currently, only
	///    [`wgpu::TextureUsages::RENDER_ATTACHMENT`]).
	/// * `label` – The string to internally label the GPU-side texture object with.
	///
	/// # Returns
	///
	/// The fully constructed depth/stencil buffer-compatible texture object.
	pub fn createDepthStencil (
		context: &Context, dims: glm::UVec2, format: hal::DepthStencilFormat,
		specialUsageFlags: Option<wgpu::TextureUsages>, label: Option<&str>
	) -> Self
	{
		// Compile depth/stencil-specific usage flags
		let usageFlags = if let Some(usages) = specialUsageFlags {
			wgpu::TextureUsages::RENDER_ATTACHMENT | usages
		} else {
			wgpu::TextureUsages::RENDER_ATTACHMENT
		};

		// Create and actual texture
		Self::createEmpty(
			context, glm::vec3(dims.x, dims.y, 1), format.into(), 1, AlphaUsage::DontCare, usageFlags,
			label
		)
	}

	/// Return a view for the level-0 mipmap (i.e. the original-resolution version).
	pub fn view (&self) -> &wgpu::TextureView {
		&self.view
	}

	/// Return the sizing information of the level-0 mipmap (i.e. for the original resolution).
	pub fn size (&self) -> &TextureSize { &self.mipLevels[0].size }

	pub fn dims (&self) -> glm::UVec3 {
		glm::vec3(self.descriptor.size.width, self.descriptor.size.height, self.descriptor.size.depth_or_array_layers)
	}

	pub fn dimsWH (&self) -> glm::UVec2 {
		glm::vec2(self.descriptor.size.width, self.descriptor.size.height)
	}

	pub fn dimsWD (&self) -> glm::UVec2 {
		glm::vec2(self.descriptor.size.width, self.descriptor.size.depth_or_array_layers)
	}

	pub fn dimsHD (&self) -> glm::UVec2 {
		glm::vec2(self.descriptor.size.height, self.descriptor.size.depth_or_array_layers)
	}

	pub fn readbackAsync<'map, Closure: FnOnce(ReadBackTexels<'map>, usize) + wgpu::WasmNotSend + 'static> (
		&self, context: &Context, callback: Closure
	){
		let mut enc = context.device().create_command_encoder(
			&wgpu::CommandEncoderDescriptor {label: Some("ReadbackTestCommandEncoder")}
		);
		enc.copy_texture_to_buffer(
			*self.readbackView_tex.as_ref().unwrap(),
			*self.readbackView_buf.as_ref().unwrap(), self.descriptor.size
		);
		context.queue().submit(Some(enc.finish()));
		let dims = self.dimsWH();
		let this = util::statify(self);
		let buf = this.readbackBuffer.as_ref().unwrap().as_ref();
		buf.slice(0..self.size().actual as u64).map_async(
			wgpu::MapMode::Read, move |result| {
				if result.is_ok()
				{
					let bufView = buf.slice(..).get_mapped_range();
					let bytes = bufView.iter().as_slice();
					let rowStride;
					let readbackInfo = match this.descriptor.format {
						wgpu::TextureFormat::Depth16Unorm => {
							rowStride = this.size().actual / (dims.y as usize * size_of::<u16>());
							ReadBackTexels::U16(unsafe { std::slice::from_raw_parts(
								bytes.as_ptr() as *const u16, bytes.len() / size_of::<u16>()
							)})
						},
						wgpu::TextureFormat::Depth24PlusStencil8 => {
							rowStride = this.size().actual / (dims.y as usize * size_of::<u32>());
							ReadBackTexels::U32(unsafe { std::slice::from_raw_parts(
								bytes.as_ptr() as *const u32, bytes.len() / size_of::<u32>()
							)})
						},
						wgpu::TextureFormat::Depth32Float => {
							rowStride = this.size().actual / (dims.y as usize * size_of::<f32>());
							ReadBackTexels::F32(unsafe { std::slice::from_raw_parts(
								bytes.as_ptr() as *const f32, bytes.len() / size_of::<f32>()
							)})
						},
						_ => unimplemented!(
							"readback for texture format {:?} not yet implemented", this.descriptor.format
						)
					};
					callback(readbackInfo, rowStride);
					drop(bufView);
					buf.unmap();
				}
				else {
					tracing::error!("readback buffer could not be mapped");
				}
			}
		);
	}
}



//////
//
// Functions
//

/// Convert a TextureDimension into its most immediate textureViewDimension equivalent (ignoring e.g. cube maps).
pub fn textureViewDimensionsEquiv (dimension: wgpu::TextureDimension) -> wgpu::TextureViewDimension {
	match dimension {
		wgpu::TextureDimension::D1 => wgpu::TextureViewDimension::D1,
		wgpu::TextureDimension::D2 => wgpu::TextureViewDimension::D2,
		wgpu::TextureDimension::D3 => wgpu::TextureViewDimension::D3
	}
}

/// Returns the number of bytes per texel for the given texture format.
pub fn numBytesFromFormat (format: wgpu::TextureFormat) -> usize
{
	use wgpu::TextureFormat::*;
	match format
	{
		Depth16Unorm => 2,

		Rgba8Unorm | Rgba8UnormSrgb | Bgra8Unorm | Bgra8UnormSrgb | Depth24PlusStencil8 | Depth32Float
		=> 4,

		_ => panic!("Unsupported or unimplemented texture format: {:?}", format)
	}
}

/// Returns whether the given texture format has an alpha channel.
pub fn hasAlpha (format: wgpu::TextureFormat) -> bool
{
	use wgpu::TextureFormat::*;
	match format
	{
		Depth16Unorm | Depth24Plus | Depth24PlusStencil8 | Depth32Float | Depth32FloatStencil8
		=> false,

		Rgba8Unorm | Rgba8UnormSrgb | Bgra8Unorm | Bgra8UnormSrgb
		=> true,

		_ => panic!("Unsupported or unimplemented texture format: {:?}", format)
	}
}

/// Determine the appropriate [`wgpu::TextureDimension`] for the given dimensions vector by checking for the first
/// component that is equal to 1. Malformed vectors (i.e. *x*-component being 1) will yield an undefined result.
pub fn textureDimensionsFromVec (dims: &glm::UVec3) -> wgpu::TextureDimension
{
	if dims.y == 1 {
		wgpu::TextureDimension::D1
	}
	else if dims.z == 1 {
		wgpu::TextureDimension::D2
	}
	else {
		wgpu::TextureDimension::D3
	}
}

/// References a reasonable, conservative default choice of `Some` [mipmap generator](gpu::mipmap::Generator) for use
/// with the corresponding `Option` of [`Texture::fromImage()`] and related methods. Currently, this is a compute
/// shader-based poly-phase box filter.
///
/// To indicate that no mipmapping should be done at all, use the `None` constant [`NO_MIPMAPS`] instead.
pub fn defaultMipmapping () -> Option<&'static gpu::mipmap::ComputeShaderGenerator<'static, gpu::mipmap::BoxFilter>> {
	Some(&DEFAULT_MIPMAP_GENERATOR)
}

/// Computes the number of mip levels in a full mip image chain for the given texture resolution.
pub fn numMipLevels (resolution: &glm::UVec3) -> u32 {
	u32::max(numMipLevels1D(resolution.x), u32::max(numMipLevels1D(resolution.y), numMipLevels1D(resolution.z)))
}

/// Computes the number of mip levels in a full mip image chain for the given 2D texture resolution.
#[allow(dead_code)]
#[inline(always)]
pub fn numMipLevels2D (resolution: &glm::UVec2) -> u32 {
	u32::max(numMipLevels1D(resolution.x), numMipLevels1D(resolution.y))
}

/// Computes the number of mip levels in a full mip image chain for the given 1D texture resolution.
#[allow(dead_code)]
#[inline(always)]
pub fn numMipLevels1D (resolution: u32) -> u32 {
	f32::floor(f32::log2(resolution as f32)) as u32 + 1
}
