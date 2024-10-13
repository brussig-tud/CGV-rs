
//////
//
// Imports
//

// Anyhow
use anyhow::Result;

// WGPU
use wgpu;

// Image library
use image::GenericImageView;



//////
//
// Classes
//

/// Represents a texture object, its data and interface to that data.
#[allow(unused)]
pub struct Texture {
	/// The device texture object.
	pub texture: wgpu::Texture,

	/// The texture view for interfacing with the texture object.
	pub view: wgpu::TextureView,

	/// The sampler for the texture. TODO: Remove from texture object and establish a sampler library.
	pub sampler: wgpu::Sampler,
}

impl Texture
{
	/// Create the texture from the given blob, uploading using the given queue on the given device.
	///
	/// # Arguments
	///
	/// * `device` – The *WGPU* device to create the texture on.
	/// * `queue` – The queue on the given device to use for uploading.
	/// * `blob` – The memory slice containing the raw bytes making up the image.
	/// * `label` – The string to internally label the GPU-side texture object with.
	pub fn fromBlob (device: &wgpu::Device, queue: &wgpu::Queue, blob: &[u8],label: &str) -> Result<Self> {
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
		device: &wgpu::Device, queue: &wgpu::Queue, image: &image::DynamicImage, label: Option<&str>
	) -> Result<Self>
	{
		// Create texture object
		let size = {
			let dims = image.dimensions();
			wgpu::Extent3d {width: dims.0, height: dims.1, depth_or_array_layers: 1}
		};
		let texture = device.create_texture(&wgpu::TextureDescriptor {
			label, size, mip_level_count: 1, sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8Unorm,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			view_formats: &[],
		});

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
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		// Done!
		Ok(Self {view: texture.create_view(&wgpu::TextureViewDescriptor::default()), texture, sampler})
	}
}
