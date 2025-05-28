
//////
//
// Imports
//

// Standard library
use std::{fs, path::Path};

// Anyhow library
use anyhow::Result;



/// Recursively run a closure on an entire directory tree.
pub fn doRecursively<PathRef: AsRef<Path>, Action: FnMut(&Path, &Path, fs::FileType)->Result<()>> (
	baseDirectory: PathRef, mut action: Action
) -> Result<()>
{
	// The actual recursive worker
	#[inline(always)]
	fn recurse<Action: FnMut(&Path, &Path, fs::FileType)->Result<()>> (source: &Path, destStack: &Path, action: &mut Action)
	-> Result<()> {
		let roottype = fs::metadata(source)?.file_type();
		action(source, &destStack, roottype)?;
		if !roottype.is_dir() { return Ok(()) }
		for entry in fs::read_dir(source)?
		{
			let entry = entry?;
			let filetype = entry.file_type()?;
			if filetype.is_dir() {
				recurse(
					&entry.path(), &destStack.join(entry.file_name()), action
				)?;
			} else {
				action(&entry.path(), &destStack.join(entry.file_name()), filetype)?;
			}
		}
		Ok(())
	}

	// Dispatch
	recurse(baseDirectory.as_ref(), Path::new(""), &mut action)
}

/// Recursively copy an entire directory tree.
#[inline(always)]
pub fn copyRecursively<PathRef: AsRef<Path>> (source: PathRef, dest: PathRef) -> Result<()>
{
	doRecursively(source, |sourcePath, destStack, filetype|
	{
		let dest = dest.as_ref().join(destStack);
		Ok(
			if filetype.is_dir() { fs::create_dir_all(dest)? }
			else                 { fs::copy(sourcePath, dest).map(|_| ())? }
		)
	})
}
