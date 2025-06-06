
//////
//
// Imports
//

// Standard library
use std::{fs, path::{Path}};

// Anyhow library
pub use anyhow::{Result, anyhow};

// Local imports
use crate::*;


//////
//
// Functions
//

/// Prepare the shaders in the given directory, skipping indicated directory sub-trees.
pub fn prepareShaders (
	buildSetup: &Setup, moduleSourceTargets: Option<&[cgv_shader::CompilationTarget]>,
	shaderDirectory: impl AsRef<Path>, skipSubDirs: Option<&[impl AsRef<Path>]>
) -> Result<()>
{
	// Sanity checks
	// - syntactic
	if shaderDirectory.as_ref().is_absolute() {
		return Err(anyhow!(
			"`shaderDirectory` is not relative: '{}'\nMust be a directory relative to the path of the Crate manifest!",
			shaderDirectory.as_ref().display()
		))
	}
	if let Some(skipSubDirs) = skipSubDirs {
		for skipSubDir in skipSubDirs {
			if skipSubDir.as_ref().is_absolute() {
				return Err(anyhow!("Skip subdir is not relative: '{}'", skipSubDir.as_ref().display()))
			}
		}
	}
	// - semantic
	let crateSrcDir = getCargoSourceDir();
	let absoluteShaderDir = crateSrcDir.join(shaderDirectory.as_ref()).canonicalize()?;
	if !absoluteShaderDir.starts_with(crateSrcDir) {
		return Err(anyhow!(
			"`shaderDirectory` is not a subdir of the Crate source root:\ncrate root: {}\nshader dir: {}",
			crateSrcDir.display(), absoluteShaderDir.display()
		))
	}

	// Determine target root directory for packaged shaders
	let targetDir = std::path::absolute(getCargoOutDir().join(shaderDirectory))?;

	// Determine module source types if none were specified
	let slangContexts = shader::slang::createContextsForTargets(
		moduleSourceTargets.unwrap_or_else(|| shader::feasibleCompilationTargets()), buildSetup.shaderPath().as_slice()
	)?;

	// Recurse through provided shader directory and package each .slang shader encountered that is not in a skipped
	// subdirectory
	util::fs::doRecursively(absoluteShaderDir, |srcPath, destStack, fileType|
		{
			if !fileType.is_dir() {
				let destStackParent = destStack.parent().ok_or_else(|| anyhow!("INTERNAL LOGIC ERROR"))?;
				let tgtPath = targetDir.join(destStack).with_extension("spk");
				let tgtParent = targetDir.join(destStackParent);
				if let Some(skipSubDirs) = skipSubDirs {
					for skipDir in skipSubDirs {
						if destStackParent.starts_with(skipDir.as_ref()) {
							// We're in a skipped directory
							return Ok(())
						}
					}
				}
				dependOnFile(srcPath);
				/*println!("cargo::warning=PREP_SHADER:");
				println!("cargo::warning=source:  {}", srcPath.display());
				println!("cargo::warning=tgtPath: {}", tgtPath.display());
				println!("cargo::warning=tgtPrnt: {}", tgtParent.display());*/
				fs::create_dir_all(tgtParent)?;
				let package = shader::Package::fromSlangMultipleContexts(&slangContexts, srcPath, None)?;
				package.writeToFile(&tgtPath)?;
				dependOnGeneratedFile(tgtPath)
			}
			else {
				Ok(())
			}
		})?;

	// Mirror the shader structure of the given directory
	Ok(())
}
