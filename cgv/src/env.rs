
//////
//
// Imports
//

// Standard library
use std::path::PathBuf;

// Serde framework
use serde;
use serde_yaml_ng;

// Local imports
/* nothing here yet */



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
impl Environment {
	pub fn fromBytes (bytes: impl AsRef<[u8]>) -> Result<Self, serde_yaml_ng::Error> {
		serde_yaml_ng::from_slice(bytes.as_ref())
	}
}
