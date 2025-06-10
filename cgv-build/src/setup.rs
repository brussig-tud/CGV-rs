
//////
//
// Imports
//

// Standard library
use std::{fs, collections::BTreeSet, path::{Path, PathBuf}};

// Anyhow library
use anyhow::Result;

// Serde library
use serde;
use serde_json;

// Local imports
use crate::*;



//////
//
// Classes
//

/// Accumulates transitive build properties for crates participating in a *CGV-rs*-based build.
#[derive(serde::Serialize,serde::Deserialize)]
pub struct Setup {
	additionalLinkerFlags: BTreeSet<String>,
	shaderPath: BTreeSet<PathBuf>,
}
impl Setup
{
	///
	pub fn new() -> Self { Self {
		additionalLinkerFlags: BTreeSet::new(),
		shaderPath: BTreeSet::new(),
	}}

	///
	pub(crate) fn fromFile (filename: impl AsRef<Path>) -> Result<Self> {
		let setup = serde_json::from_reader(fs::File::open(&filename)?)?;
		dependOnFile(filename);
		Ok(setup)
	}

	///
	pub(crate) fn fromDirectory (dirpath: impl AsRef<Path>) -> Result<Self>
	{
		let mut setup = Self::new();
		for entry in fs::read_dir(dirpath)?
		{
			let entry = entry?;
			if !entry.file_type()?.is_dir() {
				if let Some(extension) =  entry.path().extension() && extension == "json" {
					let newSetup = Self::fromFile(entry.path())?;
					setup.merge(newSetup);
				}
			}
		}
		Ok(setup)
	}

	///
	pub(crate) fn writeToFile (&self, filename: impl AsRef<Path>) -> Result<()> {
		Ok(serde_json::to_writer(fs::File::create(filename)?, self)?)
	}

	///
	pub fn injectIntoCargoBuild (&self) -> Result<()>
	{
		// Preamble
		let packageName = getCargoCrateName();
		let targetDir = getCargoTargetDir()?;
		let buildSetupDir = targetDir.join("_CGV_BUILD_SETUP");
		let buildSetupFile = buildSetupDir.join(format!("{packageName}.json"));

		// Make sure the build setup directory is there
		fs::create_dir_all(&buildSetupDir)?;

		// Serialize
		self.writeToFile(&buildSetupFile)?;

		// Rerun change detection
		dependOnGeneratedFile(buildSetupFile)?;

		// Done!
		Ok(())
	}

	///
	pub fn merge (&mut self, other: Self) {
		self.additionalLinkerFlags.extend(other.additionalLinkerFlags);
		self.shaderPath.extend(other.shaderPath);
	}

	///
	pub fn addLinkerFlag (&mut self, flagString: impl AsRef<str>) {
		flagString.as_ref().split_ascii_whitespace().for_each(|flag| {
			self.additionalLinkerFlags.insert(flag.to_owned());
		});
	}

	///
	pub fn addShaderPath (&mut self, path: impl AsRef<Path>)
	{
		if let Ok(path) = path.as_ref().canonicalize() {
			self.shaderPath.insert(path);
		}
		else {
			println!("cargo::warning=Unaccessible shader path directory: '{}'", path.as_ref().display());
		}
	}

	///
	pub fn apply (&self)
	{
		// Accumulate linker flags into whitespace-separated string
		let additionalLinkerFlags = self.additionalLinkerFlags.iter().fold(
			String::new(), |flags, flag| format!("{flags} {flag}")
		);

		// Emit flags string to Cargo
		if !self.additionalLinkerFlags.is_empty() {
			println!("cargo:rustc-link-arg={additionalLinkerFlags}");
		}
	}

	///
	pub fn shaderPath (&self) -> Vec<&Path> {
		self.shaderPath.iter().map(|path| path.as_path()).collect()
	}
}
