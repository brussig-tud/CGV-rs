
//////
//
// Language config
//

// Eff this convention.
#![allow(non_snake_case)]



//////
//
// Imports
//

// Standard library
use std::{fs, path::{Path, PathBuf}};

// Serde framework
use serde;
use serde_yaml_ng;



//////
//
// Structs
//

/// A struct storing runtime environment information (most notably, the shader search path) for a *CGV-rs* application.
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Environment {
	/// Array of directory paths to search for whenever shader files are given by a relative path.
	pub shaderPath: Vec<PathBuf>
}
impl Environment
{
	///
	pub fn serialize (&self) -> Vec<u8> {
		let mut bytes = Vec::new();
		serde_yaml_ng::to_writer(&mut bytes, self).expect(
			"INTERNAL LOGIC ERROR: failed to serialize in instance of cgv_runenv::Environment"
		);
		bytes
	}

	///
	pub fn serializeToFile (&self, filename: impl AsRef<Path>) -> anyhow::Result<()> {
		Ok(fs::write(filename, self.serialize())?)
	}

	///
	pub fn deserialize (bytes: impl AsRef<[u8]>) -> Result<Self, serde_yaml_ng::Error> {
		serde_yaml_ng::from_slice(bytes.as_ref())
	}
}
