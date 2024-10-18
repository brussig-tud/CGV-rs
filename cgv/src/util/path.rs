
//////
//
// Imports
//

use std::path::*;



//////
//
// Functions
//

/// Normalizes (i.e. returns an absolute path) the given *path* relative to the given *anchor*, **iff** *path* is
/// relative. If it is not, it will just be returned verbatim.
///
/// # Arguments
///
/// * `anchor` – The path to normalize relative to.
/// * `path` – The path to normalize in case it is relative.
///
/// # Return
///
/// The absolute path resulting from normalizing *path*.
pub fn normalizeToAnchor<PathRef: AsRef<Path>> (anchor: PathRef, path: PathRef) -> PathBuf
{
	let path: &Path = path.as_ref();
	if path.is_relative() {
		absolute(anchor.as_ref().join(path)).unwrap()
	}
	else {
		path.into()
	}
}
