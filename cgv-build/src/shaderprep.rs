
//////
//
// Imports
//

// Standard library
use std::{env, fs, path::{Path, PathBuf}};

// Anyhow library
pub use anyhow::{Context, Result, anyhow};



//////
//
// Classes
//

/// Type representing the shader path in which modules are being looked for.
pub struct ShaderPath {
	pub directories: Vec<PathBuf>,
}



//////
//
// Functions
//

///
pub fn prepShaders(directory: impl AsRef<Path>, shaderPath: Option<&ShaderPath>) {
	//let root = crate::obtainCrateLocalBuildFS(Some(directory));
}
