
//////
//
// Imports
//

// Standard library
/* nothing here yet */



//////
//
// Module definitions
//

/// Submodule providing the [`Texture`](texture::Texture) facilities
mod texture;
// - re-exports
pub use texture::{Texture, TextureSize, ReadBackTexels, AlphaUsage, NO_MIPMAPS};
pub use texture::{
	numBytesFromFormat, hasAlpha, textureDimensionsFromVec, defaultMipmapping, numMipLevels, numMipLevels2D,
	numMipLevels1D
};

/// Submodule providing the [`Framebuffer`](texture::Framebuffer) facilities
mod framebuffer;
// - re-exports
pub use framebuffer::{Framebuffer, FramebufferBuilder, DynamicFramebuffer, DepthStencilFormat};
pub use framebuffer::{decodeDepth, decodeDepthU16, decodeDepthU32};

/// Submodule providing the [`UniformGroup`](uniformgroup::UniformGroup) facilities
mod uniformgroup;
pub use uniformgroup::UniformGroup; // re-export
