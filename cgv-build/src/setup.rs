
//////
//
// Imports
//

// Standard library
use std::{fs, path::{Path, PathBuf}};

// Anyhow library
use anyhow::{Context, Result, anyhow};

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
	additionalLinkerFlags: Option<String>,
	shaderPath: Vec<PathBuf>,
}
impl Setup
{
	pub fn new() -> Self { Self {
		additionalLinkerFlags: None,
		shaderPath: Vec::new(),
	}}

	pub(crate) fn fromFile (filename: impl AsRef<std::path::Path>) -> Result<Self>
	{
		/*// The resulting setup
		let mut setup = Self::new();

		// Parse build setup file
		let contents = std::fs::read_to_string(path)?;
		contents.split('\n').for_each(|line|
		{
			// Skip empty line
			if line.split_ascii_whitespace().next().is_none() {
				return;
			}

			// Parse non-empty line
			if let Some((key, value)) = line.split_once('=')
			{
				let key = key.trim().to_owned();
				let value = value.trim().to_owned();
				match key.as_str()
				{
					"ADDITIONAL_LINKER_ARGS" => setup.addLinkerFlag(&value),

					_ => {
						// Warn of unrecognized key and ignore
						println!("cargo:warning=cgv_build::Setup::fromFile(): Unrecognized key: {line}");
					}
				}
			} else {
				println!("cargo:warning=cgv_build::Setup::fromFile(): Cannot interpret line: {line}");
			}
		});*/
		Ok(serde_json::from_reader(fs::File::open(filename)?)?)
	}

	pub(crate) fn writeToFile (&self, filename: impl AsRef<std::path::Path>) -> Result<()> {
		Ok(serde_json::to_writer(fs::File::create(filename)?, self)?)
	}

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
		if !util::setTimestampToBeforeBuildScriptTime(buildSetupFile) {
			println!(
				"cargo::warning=Build setup timestamp management failed! Cargo change detection will be affected."
			);
		}
		Ok(())
	}

	pub fn addLinkerFlag (&mut self, flag: impl AsRef<str>)
	{
		if let Some(flags) = self.additionalLinkerFlags.as_mut() {
			flags.push_str(flag.as_ref());
		}
		else {
			self.additionalLinkerFlags = Some(flag.as_ref().into());
		}
	}

	pub fn apply (&self) {
		if let Some(flags) = self.additionalLinkerFlags.as_ref() {
			println!("cargo:rustc-link-arg={}", flags);
		}
	}
}
