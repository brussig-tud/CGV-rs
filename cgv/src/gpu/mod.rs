
//////
//
// Imports
//

// WGPU API
use wgpu;

// Local imports
use crate::*;



//////
//
// Module definitions
//

/// Submodule providing utilities for fast mip map computation
pub mod mipmap;



//////
//
// Structs and enums
//

/// Holds a reference to either a [render](wgpu::RenderPass) or [compute](wgpu::ComputePass) pass.
#[derive(Debug)]
pub enum Pass<'encoder> {
	/// This GPU pass represents a "classical" render pass.
	Render(wgpu::RenderPass<'encoder>),

	/// This GPU pass represents a compute pass.
	Compute(wgpu::ComputePass<'encoder>)
}
impl<'encoder> Pass<'encoder>
{
	/// Check if the GPU pass is a [render pass](Pass::Render).
	fn _isRender (&self) -> bool {
		if let Pass::Render(_) = self { true } else { false }
	}

	/// Check if the GPU pass is a [compute pass](Pass::Compute).
	fn _isCompute (&self) -> bool {
		if let Pass::Render(_) = self { true } else { false }
	}

	/// Retrieve a reference to the wrapped [`wgpu::RenderPass`], panicking if we are not actually holding one.
	fn _refRender (&mut self) -> &mut wgpu::RenderPass<'encoder> {
		if let Pass::Render(pass) = self { pass } else {
			panic!("Attempted to reference render pass from non-render pass!");
		}
	}

	/// Retrieve a reference to the wrapped [`wgpu::ComputePass`], panicking if we are not actually holding one.
	fn refCompute (&mut self) -> &mut wgpu::ComputePass<'encoder> {
		if let Pass::Compute(pass) = self { pass } else {
			panic!("Attempted to reference compute pass from non-compute pass!");
		}
	}
}
