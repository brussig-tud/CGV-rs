

//////
//
// Module definitions
//

/// Deprecated submodule providing linear data processing functionality, ignore and don't use.
pub mod DEPRECATED_data;

/// Submodule providing utilities for fast mip map computation
pub mod mipmap;



//////
//
// Imports
//

// Standard library
/* nothing here yet */

// WGPU API
use wgpu;

// Local imports
use crate::*;



//////
//
// Structs and enums
//

/// Holds a reference to either a [render](wgpu::RenderPass) or [compute](wgpu::ComputePass) pass.
#[derive(Debug)]
pub enum Pass<'this> {
	/// This GPU pass represents a "classical" render pass.
	Render(wgpu::RenderPass<'this>),

	/// This GPU pass represents a compute pass.
	Compute(wgpu::ComputePass<'this>)
}
impl<'this> Pass<'this>
{
	/// Check if the GPU pass is a [render pass](Pass::Render).
	#[expect(unused)]
	fn isRender (&self) -> bool {
		if let Pass::Render(_) = self { true } else { false }
	}

	/// Check if the GPU pass is a [compute pass](Pass::Compute).
	#[expect(unused)]
	fn isCompute (&self) -> bool {
		if let Pass::Render(_) = self { true } else { false }
	}

	/// Retrieve a reference to the wrapped [`wgpu::RenderPass`], panicking if we are not actually holding one.
	#[expect(unused)]
	fn refRender<'outer> (&'outer mut self) -> &'outer mut wgpu::RenderPass<'this> {
		if let Pass::Render(pass) = self { pass } else {
			panic!("Attempted to reference render pass from non-render pass!");
		}
	}

	/// Retrieve a reference to the wrapped [`wgpu::ComputePass`], panicking if we are not actually holding one.
	fn refCompute<'outer> (&'outer mut self) -> &'outer mut wgpu::ComputePass<'this> {
		if let Pass::Compute(pass) = self { pass } else {
			panic!("Attempted to reference compute pass from non-compute pass!");
		}
	}
}
