
//////
//
// Imports
//

// Standard library
use std::collections::BTreeMap;

// Dashmap library
use dashmap::DashMap;

// WGPU API
use wgpu;

// Local imports
use crate::*;



//////
//
// Structs and enums
//

/* nothing here yet */



//////
//
// Classes
//

/// An abstraction over a shader program with changeable symbolic constants and internally managed re-compilation.
pub struct ShaderProgram
{
	/// The desired user debug label to attach to built shader modules.
	label: Option<String>,

	/// The original shader source code.
	code: String,

	/// The cache of different instances of the shader program (for different sets of constants).
	_moduleCache: DashMap<Vec<String>, Box<wgpu::ShaderModule>>
}
impl ShaderProgram
{
	/// Create the program from the given source code.
	///
	/// # Arguments
	///
	/// * `context` – The *CGV-rs* context under which to create resources.
	/// * `code` – A string containing the (possibly augmented) WGSL source code of the shader program.
	/// * `label` – The debug label to attach to built shader modules, if desired.
	///
	/// # Returns
	///
	/// The fully constructed shader program with one pre-built module for the initial set of constant values defined in
	/// the shader source.
	pub fn fromSource (_context: &Context, code: impl AsRef<str>, label: Option<impl ToString>) -> Self {
		Self {
			label: label.map(|v| v.to_string()),
			code: String::from(code.as_ref()),
			_moduleCache: DashMap::with_capacity(4)
		}
	}

	/// Obtain a *WGPU* shader module for the given set of constants.
	///
	/// # Arguments
	///
	/// * `context` – The *CGV-rs* context under which to create the module if it doesn't yet exist.
	/// * `constants` – The map of constant values that should be applied in the module. When this is empty, the module
	///                 where all values have been taken from the definitions in the original source will be returned.
	///
	/// # Returns
	///
	/// A reference to a compiled and linked shader module with the constants set to the specified values.
	pub fn refModule (&mut self, context: &Context, _constants: &BTreeMap<impl AsRef<str>, impl ToString>)
	-> &wgpu::ShaderModule
	{
		// TODO: This is just here to prevent built errors while the functionality is missing
		let module = util::notsafe::UncheckedRef::new(
			&context.device().create_shader_module(wgpu::ShaderModuleDescriptor {
				label: self.label.as_ref().map(|l| l.as_str()),
				source: wgpu::ShaderSource::Wgsl(self.code.as_str().into())
			})
		);
		unsafe { module.as_ref() } // Safety: this will crash for sure
	}
}
