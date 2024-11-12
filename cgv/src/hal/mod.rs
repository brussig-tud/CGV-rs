
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
pub use texture::Texture;
pub use texture::TextureSize;
pub use texture::ReadBackTexels;

/// Submodule providing the [`Framebuffer`](texture::Framebuffer) facilities
mod framebuffer;
// - re-exports
pub use framebuffer::Framebuffer;
pub use framebuffer::FramebufferBuilder;
pub use framebuffer::DepthStencilFormat;

/// Submodule providing the [`UniformGroup`](uniformgroup::UniformGroup) facilities
mod uniformgroup;
pub use uniformgroup::UniformGroup; // re-export
