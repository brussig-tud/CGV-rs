
//////
//
// Imports
//

// Standard library
use std::path::*;

// Normpalize-path library
use normalize_path::NormalizePath;



//////
//
// Functions
//

/// Normalizes the given path, i.e. resolves/collapses any and all `..` and `.` contained within.
///
/// # Arguments
///
/// * `path` – The path to normalize.
///
/// # Returns
///
/// The normalized path.
pub fn normalize<PathRef: AsRef<Path>> (path: PathRef) -> PathBuf {
	path.as_ref().normalize()
}

/// Normalizes (i.e. returns an absolute path) the given *path* relative to the given *anchor*, **iff** *path* is
/// relative. If it is not, it will just be returned verbatim.
///
/// # Arguments
///
/// * `anchor` – The path to normalize relative to.
/// * `path` – The path to normalize in case it is relative.
///
/// # Returns
///
/// The absolute path resulting from normalizing *path*.
pub fn normalizeToAnchor<PathRef1: AsRef<Path>, PathRef2: AsRef<Path>> (anchor: PathRef1, path: PathRef2) -> PathBuf
{
	if path.as_ref().is_relative() {
		anchor.as_ref().join(path).normalize()
	}
	else {
		path.as_ref().into()
	}
}
