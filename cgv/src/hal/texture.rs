
//////
//
// Imports
//

// Standard library
/* nothing here yet */

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
pub struct TextureSize {
	pub logical: usize,
	pub actual: usize
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



//////
//
// Classes
//

/// Represents a texture object, its data and interface to that data.
#[allow(unused)]
pub struct Texture {
	/// The name (if any) of the texture object.
	pub name: Option<String>,

	/// How to interpret the alpha channel (if any) when blending.
	pub alphaUsage: AlphaUsage,

	/// The device texture object.
	pub texture: Box<wgpu::Texture>,

	/// The descriptor used to create the [texture object](texture).
	pub descriptor: wgpu::TextureDescriptor<'static>,

	/// The texture view for interfacing with the texture object.
	pub view: wgpu::TextureView,

	/// The sampler for the texture. TODO: Remove from texture object and make context have a sampler library instead
	pub sampler: wgpu::Sampler,

	/// The buffer object for readback operations in case the texture usage allows for that
	pub readbackBuffer: Option<Box<wgpu::Buffer>>,

	/// The ImageCopyTexture-compatible view on the texture in case readback is enabled
	pub readbackView_tex: Option<wgpu::ImageCopyTexture<'static>>,

	/// The ImageCopyTexture-compatible view on the texture in case readback is enabled
	pub readbackView_buf: Option<wgpu::ImageCopyBuffer<'static>>,

	// Cached size (wihtout mipmap levels) in bytes.
	pub size: TextureSize
}

impl Texture
{
	/// Create the texture from the given blob.
	///
	/// # Arguments
	///
	/// * `context` – The *CGV-rs* context under which to create the texture.
	/// * `blob` – The memory slice containing the raw bytes making up the image.
	/// * `specialUsageFlags` – An optional set of [texture usage flags](wgpu::TextureUsages) to add on to the minimum
	///    required usages for creating a texture from host-data.
	/// * `label` – An optional name to internally label the GPU-side texture object with.
	pub fn fromBlob (
		context: &Context, blob: &[u8], specialUsageFlags: Option<wgpu::TextureUsages>, label: Option<&str>
	) -> Result<Self> {
		let img = image::load_from_memory(blob)?;
		Self::fromImage(context, &img, specialUsageFlags, label)
	}

	/// Create the texture from the given image.
	///
	/// # Arguments
	///
	/// * `context` – The *CGV-rs* context under which to create the texture.
	/// * `image` – The image the texture should contain.
	/// * `specialUsageFlags` – An optional set of [texture usage flags](wgpu::TextureUsages) to add on to the minimum
	///    required usages for creating a texture from host-data (currently, only [`wgpu::TextureUsages::COPY_DST`]).
	/// * `label` – The string to internally label the GPU-side texture object with.
	pub fn fromImage (
		context: &Context, image: &image::DynamicImage, specialUsageFlags: Option<wgpu::TextureUsages>,
		label: Option<&str>
	) -> Result<Self>
	{
		// Store name in owned memory
		let name = label.map(String::from);
		let label = if let Some(name) = &name {
			Some(util::statify(name.as_str()))
		} else {
			None
		};

		// Create texture object
		let size = {
			let dims = image.dimensions();
			wgpu::Extent3d {width: dims.0, height: dims.1, depth_or_array_layers: 1}
		};
		let descriptor = wgpu::TextureDescriptor {
			label, size, mip_level_count: 1, sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8Unorm,
			usage: if let Some(usages) = specialUsageFlags {
				wgpu::TextureUsages::COPY_DST | usages
			} else {
				wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
			},
			view_formats: &[],
		};
		let texture = Box::new(context.device().create_texture(&descriptor));

		// Upload to GPU
		context.queue().write_texture(
			wgpu::ImageCopyTexture {
				aspect: wgpu::TextureAspect::All,
				texture: &texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
			},
			&image.to_rgba8(),
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * size.width),
				rows_per_image: Some(size.height),
			},
			size,
		);
		context.queue().submit([]); // make sure the texture transfer starts immediately

		// Create sampler
		let sampler = context.device().create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Linear,
			..Default::default()
		});

		let size = {
			let logicalBytesPerRow = numBytesFromFormat(descriptor.format) * descriptor.size.width as usize;
			let heightTimesDepth = (descriptor.size.height * descriptor.size.depth_or_array_layers) as usize;
			TextureSize {
				logical: logicalBytesPerRow * heightTimesDepth,
				actual: heightTimesDepth * alignToFactor(
					logicalBytesPerRow, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize
				)
			}
		};
		let readbackBuffer = match &specialUsageFlags {
			Some(wgpu::TextureUsages::COPY_SRC) => Some(Box::new(context.device().create_buffer(
				&wgpu::BufferDescriptor {
					label: util::concatIfSome(&label, "_readbackBuf").as_deref(),
					size: size.actual as u64,
					usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
					mapped_at_creation: false
				}
			))),
			_ => None
		};
		let readbackView_tex = match &readbackBuffer {
			Some(_) => Some(wgpu::ImageCopyTexture {
				texture: util::statify(texture.as_ref()),
				mip_level: 0,
				origin: Default::default(),
				aspect: wgpu::TextureAspect::DepthOnly,
			}),
			_ => None
		};
		let readbackView_buf = match &readbackBuffer {
			Some(buffer) => Some(wgpu::ImageCopyBuffer {
				buffer: util::statify(buffer.as_ref()),
				layout: wgpu::ImageDataLayout {
					bytes_per_row: Some(util::math::alignToFactor(
						descriptor.size.width * numBytesFromFormat(descriptor.format) as u32,
						wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
					)),
					..Default::default()
				}
			}),
			_ => None
		};

		// Done!
		Ok(Self {
			alphaUsage: AlphaUsage::DontCare, view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
			texture, name, size, readbackBuffer, readbackView_tex, readbackView_buf, descriptor, sampler
		})
	}

	/// Create a generic uninitialized texture of arbitrary format.
	///
	/// # Arguments
	///
	/// * `context` – The *CGV-rs* context under which to create the texture.
	/// * `dims` – The desired dimensions in terms of width and height.
	/// * `format` – The desired format of the texture.
	/// * `alphaUsage` – How the alpha channel of the texture (if any) should be used when blending.
	/// * `usageFlags` – The set of [texture usages](wgpu::TextureUsages) the texture is intended for.
	/// * `label` – The string to internally label the GPU-side texture object with.
	pub fn createEmptyTexture(
		context: &Context, dims: glm::UVec2, format: wgpu::TextureFormat, alphaUsage: AlphaUsage,
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

		let descriptor = wgpu::TextureDescriptor {
			format,	label, size: wgpu::Extent3d {width: dims.x, height: dims.y, depth_or_array_layers: 1},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			usage: usageFlags,
			view_formats: &[],
		};
		let texture = Box::new(context.device().create_texture(&descriptor));

		let sampler = context.device().create_sampler(
			&wgpu::SamplerDescriptor { // 4.
				address_mode_u: wgpu::AddressMode::ClampToEdge,
				address_mode_v: wgpu::AddressMode::ClampToEdge,
				address_mode_w: wgpu::AddressMode::ClampToEdge,
				mag_filter: wgpu::FilterMode::Nearest,
				min_filter: wgpu::FilterMode::Nearest,
				mipmap_filter: wgpu::FilterMode::Nearest,
				compare: Some(wgpu::CompareFunction::LessEqual), // 5.
				lod_min_clamp: 0.0,
				lod_max_clamp: 100.0,
				..Default::default()
			}
		);

		let size = {
			let logicalBytesPerRow = numBytesFromFormat(descriptor.format) * descriptor.size.width as usize;
			let heightTimesDepth = (descriptor.size.height * descriptor.size.depth_or_array_layers) as usize;
			TextureSize {
				logical: logicalBytesPerRow * heightTimesDepth,
				actual: heightTimesDepth * alignToFactor(
					logicalBytesPerRow, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize
				)
			}
		};
		let readbackBuffer = usageFlags.contains(wgpu::TextureUsages::COPY_SRC).then(||
			Box::new(context.device().create_buffer( &wgpu::BufferDescriptor {
				label: util::concatIfSome(&label, "_readbackBuf").as_deref(),
				size: size.actual as u64,
				usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
				mapped_at_creation: false
			}))
		);
		let readbackView_tex = match &readbackBuffer {
			Some(_) => Some(wgpu::ImageCopyTexture {
				texture: util::statify(texture.as_ref()),
				mip_level: 0,
				origin: Default::default(),
				aspect: wgpu::TextureAspect::DepthOnly,
			}),
			_ => None
		};
		let readbackView_buf = match &readbackBuffer {
			Some(buffer) => Some(wgpu::ImageCopyBuffer {
				buffer: util::statify(buffer.as_ref()),
				layout: wgpu::ImageDataLayout {
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
			view: texture.create_view(&wgpu::TextureViewDescriptor::default()), alphaUsage,
			texture, name, size, readbackBuffer, readbackView_tex, readbackView_buf, descriptor, sampler
		}
	}

	/// Create an uninitialized texture suitable for use as a depth/stencil attachment.
	///
	/// # Arguments
	///
	/// * `context` – The *CGV-rs* context under which to create the texture.
	/// * `dims` – The desired dimensions in terms of width and height.
	/// * `format` – The desired depth/stencil format of the texture.
	/// * `specialUsageFlags` – An optional set of [texture usage flags](wgpu::TextureUsages) to add on to the minimum
	///    required usages for creating a texture from host-data (currently, only [`wgpu::TextureUsages::RENDER_ATTACHMENT`]).
	/// * `label` – The string to internally label the GPU-side texture object with.
	pub fn createDepthStencilTexture(
		context: &Context, dims: glm::UVec2, format: hal::DepthStencilFormat,
		specialUsageFlags: Option<wgpu::TextureUsages>, label: Option<&str>
	) -> Self
	{
		// Store name in owned memory
		let name = label.map(String::from);
		let label = if let Some(name) = &name {
			Some(util::statify(name.as_str()))
		} else {
			None
		};

		let descriptor = wgpu::TextureDescriptor {
			label, size: wgpu::Extent3d {width: dims.x, height: dims.y, depth_or_array_layers: 1},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: format.into(),
			usage: if let Some(usages) = specialUsageFlags {
				wgpu::TextureUsages::RENDER_ATTACHMENT | usages
			} else {
				wgpu::TextureUsages::RENDER_ATTACHMENT
			},
			view_formats: &[],
		};
		let texture = Box::new(context.device().create_texture(&descriptor));

		let sampler = context.device().create_sampler(
			&wgpu::SamplerDescriptor { // 4.
				address_mode_u: wgpu::AddressMode::ClampToEdge,
				address_mode_v: wgpu::AddressMode::ClampToEdge,
				address_mode_w: wgpu::AddressMode::ClampToEdge,
				mag_filter: wgpu::FilterMode::Nearest,
				min_filter: wgpu::FilterMode::Nearest,
				mipmap_filter: wgpu::FilterMode::Nearest,
				compare: Some(wgpu::CompareFunction::LessEqual), // 5.
				lod_min_clamp: 0.0,
				lod_max_clamp: 100.0,
				..Default::default()
			}
		);

		let size = {
			let logicalBytesPerRow = numBytesFromFormat(descriptor.format) * descriptor.size.width as usize;
			let heightTimesDepth = (descriptor.size.height * descriptor.size.depth_or_array_layers) as usize;
			TextureSize {
				logical: logicalBytesPerRow * heightTimesDepth,
				actual: heightTimesDepth * alignToFactor(
					logicalBytesPerRow, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize
				)
			}
		};
		let readbackBuffer = match specialUsageFlags {
			Some(wgpu::TextureUsages::COPY_SRC) => Some(Box::new(context.device().create_buffer(
				&wgpu::BufferDescriptor {
					label: util::concatIfSome(&label, "_readbackBuf").as_deref(),
					size: size.actual as u64,
					usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
					mapped_at_creation: false
				}
			))),
			_ => None
		};
		let readbackView_tex = match &readbackBuffer {
			Some(_) => Some(wgpu::ImageCopyTexture {
				texture: util::statify(texture.as_ref()),
				mip_level: 0,
				origin: Default::default(),
				aspect: wgpu::TextureAspect::DepthOnly,
			}),
			_ => None
		};
		let readbackView_buf = match &readbackBuffer {
			Some(buffer) => Some(wgpu::ImageCopyBuffer {
				buffer: util::statify(buffer.as_ref()),
				layout: wgpu::ImageDataLayout {
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
			alphaUsage: AlphaUsage::DontCare, view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
			texture, name, size, readbackBuffer, readbackView_tex, readbackView_buf, descriptor, sampler
		}
	}

	/// The logical size of the uncompressed texture (excluding mipmap levels) in bytes.
	pub fn size (&self) -> usize { self.size.logical }

	pub fn dims (&self) -> glm::UVec3 {
		let size = &self.descriptor.size;
		glm::vec3(size.width, size.height, size.depth_or_array_layers)
	}

	pub fn dimsWH (&self) -> glm::UVec2 {
		let size = &self.descriptor.size;
		glm::vec2(size.width, size.height)
	}

	pub fn dimsWD (&self) -> glm::UVec2 {
		let size = &self.descriptor.size;
		glm::vec2(size.width, size.depth_or_array_layers)
	}

	pub fn dimsHD (&self) -> glm::UVec2 {
		let size = &self.descriptor.size;
		glm::vec2(size.height, size.depth_or_array_layers)
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
		buf.slice(0..self.size.actual as u64).map_async(
			wgpu::MapMode::Read, move |result| {
				if result.is_ok()
				{
					let bufView = buf.slice(..).get_mapped_range();
					let bytes = bufView.iter().as_slice();
					let rowStride;
					let readbackInfo = match this.descriptor.format {
						wgpu::TextureFormat::Depth16Unorm => {
							rowStride = this.size.actual / (dims.y as usize * size_of::<u16>());
							ReadBackTexels::U16(unsafe { std::slice::from_raw_parts(
								bytes.as_ptr() as *const u16, bytes.len() / size_of::<u16>()
							)})
						},
						wgpu::TextureFormat::Depth24PlusStencil8 => {
							rowStride = this.size.actual / (dims.y as usize * size_of::<u32>());
							ReadBackTexels::U32(unsafe { std::slice::from_raw_parts(
								bytes.as_ptr() as *const u32, bytes.len() / size_of::<u32>()
							)})
						},
						wgpu::TextureFormat::Depth32Float => {
							rowStride = this.size.actual / (dims.y as usize * size_of::<f32>());
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
