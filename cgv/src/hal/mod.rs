
//////
//
// Imports
//

// Anyhow library
use anyhow::Result;

// WGPU API
use wgpu;

// Image library
use image::GenericImageView;
use winit::dpi::PhysicalSize;
//////
//
// Enums
//

/// High-level enum encompassing all pure depth formats.
#[derive(Clone, Copy)]
pub enum DepthFormat
{
	/// 16-bits integer.
	D16,

	/// 24-bits integer.
	D24,

	/// 32-bits floating point.
	D32
}

/// High-level enum encompassing all pure depth formats.
#[derive(Clone, Copy)]
pub enum DepthStencilFormat
{
	/// 24-bits integer depth + 8-bits stencil.
	D24S8,

	/// 32-bits floating point depth + 8-bits stencil (requires feature support).
	D32S8
}



//////
//
// Classes
//

/// Represents a texture object, its data and interface to that data.
#[allow(unused)]
pub struct Texture<'a> {
	/// The device texture object.
	pub texture: wgpu::Texture,

	/// The descriptor used to create the [texture object](texture).
	pub descriptor: wgpu::TextureDescriptor<'a>,

	/// The texture view for interfacing with the texture object.
	pub view: wgpu::TextureView,

	/// The sampler for the texture. TODO: Remove from texture object and establish a sampler library.
	pub sampler: wgpu::Sampler,
}

impl<'a> Texture<'a>
{

	/// Create the texture from the given blob, uploading using the given queue on the given device.
	///
	/// # Arguments
	///
	/// * `device` – The *WGPU* device to create the texture on.
	/// * `queue` – The queue on the given device to use for uploading.
	/// * `blob` – The memory slice containing the raw bytes making up the image.
	/// * `label` – The string to internally label the GPU-side texture object with.
	pub fn fromBlob (device: &wgpu::Device, queue: &wgpu::Queue, blob: &[u8], label: &'a str) -> Result<Self> {
		let img = image::load_from_memory(blob)?;
		Self::fromImage(device, queue, &img, Some(label))
	}

	/// Create the texture from the given image, uploading using the given queue on the given device.
	///
	/// # Arguments
	///
	/// * `device` – The *WGPU* device to create the texture on.
	/// * `queue` – The queue on the given device to use for uploading.
	/// * `image` – The image the texture should contain.
	/// * `label` – The string to internally label the GPU-side texture object with.
	pub fn fromImage (
		device: &wgpu::Device, queue: &wgpu::Queue, image: &image::DynamicImage, label: Option<&'a str>
	) -> Result<Self>
	{
		// Create texture object
		let size = {
			let dims = image.dimensions();
			wgpu::Extent3d {width: dims.0, height: dims.1, depth_or_array_layers: 1}
		};
		let descriptor = wgpu::TextureDescriptor {
			label, size, mip_level_count: 1, sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8Unorm,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			view_formats: &[],
		};
		let texture = device.create_texture(&descriptor);

		// Upload to GPU
		queue.write_texture(
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
		queue.submit([]); // make sure the texture transfer starts immediately

		// Create interface
		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Linear,
			..Default::default()
		});

		// Done!
		Ok(Self {view: texture.create_view(&wgpu::TextureViewDescriptor::default()), texture, descriptor, sampler})

	}

	pub fn createDepthTexture(
		device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, format: DepthFormat, label: Option<&'a str>
	) -> Self
	{
		let size = wgpu::Extent3d {
			width: config.width.max(1), height: config.height.max(1),
			depth_or_array_layers: 1,
		};
		let descriptor = wgpu::TextureDescriptor {
			label,
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: match format {
				DepthFormat::D16 => wgpu::TextureFormat::Depth16Unorm,
				DepthFormat::D24 => wgpu::TextureFormat::Depth24Plus,
				DepthFormat::D32 => wgpu::TextureFormat::Depth32Float
			},
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[],
		};
		let texture = device.create_texture(&descriptor);

		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		let sampler = device.create_sampler(
			&wgpu::SamplerDescriptor { // 4.
				address_mode_u: wgpu::AddressMode::ClampToEdge,
				address_mode_v: wgpu::AddressMode::ClampToEdge,
				address_mode_w: wgpu::AddressMode::ClampToEdge,
				mag_filter: wgpu::FilterMode::Linear,
				min_filter: wgpu::FilterMode::Linear,
				mipmap_filter: wgpu::FilterMode::Nearest,
				compare: Some(wgpu::CompareFunction::LessEqual), // 5.
				lod_min_clamp: 0.0,
				lod_max_clamp: 100.0,
				..Default::default()
			}
		);

		Self { texture, descriptor, view, sampler }
	}

	pub fn createDepthStencilTexture(
		device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, format: DepthStencilFormat, label: Option<&'a str>
	) -> Self
	{
		let size = wgpu::Extent3d {
			width: config.width.max(1), height: config.height.max(1),
			depth_or_array_layers: 1,
		};
		let descriptor = wgpu::TextureDescriptor {
			label,
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: match format {
				DepthStencilFormat::D24S8 => wgpu::TextureFormat::Depth24PlusStencil8,
				DepthStencilFormat::D32S8 => wgpu::TextureFormat::Depth32FloatStencil8
			},
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[],
		};
		let texture = device.create_texture(&descriptor);

		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		let sampler = device.create_sampler(
			&wgpu::SamplerDescriptor { // 4.
				address_mode_u: wgpu::AddressMode::ClampToEdge,
				address_mode_v: wgpu::AddressMode::ClampToEdge,
				address_mode_w: wgpu::AddressMode::ClampToEdge,
				mag_filter: wgpu::FilterMode::Linear,
				min_filter: wgpu::FilterMode::Linear,
				mipmap_filter: wgpu::FilterMode::Nearest,
				compare: Some(wgpu::CompareFunction::LessEqual), // 5.
				lod_min_clamp: 0.0,
				lod_max_clamp: 100.0,
				..Default::default()
			}
		);

		Self { texture, descriptor, view, sampler }
	}

	pub fn size(&self) -> glm::UVec3 {
		let size = &self.descriptor.size;
		glm::vec3(size.width, size.height, size.depth_or_array_layers)
	}

	pub fn size2WH(&self) -> glm::UVec2 {
		let size = &self.descriptor.size;
		glm::vec2(size.width, size.height)
	}

	pub fn size2WD(&self) -> glm::UVec2 {
		let size = &self.descriptor.size;
		glm::vec2(size.width, size.depth_or_array_layers)
	}

	pub fn size2HD(&self) -> glm::UVec2 {
		let size = &self.descriptor.size;
		glm::vec2(size.height, size.depth_or_array_layers)
	}

	pub fn physicalSizeWH(&self) -> PhysicalSize<u32> {
		let size = &self.descriptor.size;
		PhysicalSize::new(size.width, size.height)
	}
}
