
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
pub use texture::{Texture, TextureSize, ReadBackTexels, AlphaUsage, numBytesFromFormat, hasAlpha}; // re-export

/// Submodule providing the [`Framebuffer`](texture::Framebuffer) facilities
mod framebuffer;
// - re-exports
pub use framebuffer::{Framebuffer, FramebufferBuilder, DynamicFramebuffer, DepthStencilFormat};
pub use framebuffer::{decodeDepth, decodeDepthU16, decodeDepthU32};

/// Submodule providing the [`UniformGroup`](uniformgroup::UniformGroup) facilities
mod uniformgroup;
pub use uniformgroup::UniformGroup; // re-export
