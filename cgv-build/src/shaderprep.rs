
/// All facilities in this module assume the default Slang shading language. If crates want to provide different compile
/// contexts and environments, they should also provide their own build-time facilities similar to this one.



//////
//
// Imports
//

// Standard library
use std::{fs, path::{Path}};

// Anyhow library
pub use anyhow::{Result, anyhow};

// CGV-rs
use cgv_shader as shader;
use cgv_util as util;
use shader::compile::prelude::*;

// Local imports
use crate::*;



//////
//
// Structs
//

///
pub struct EnvironmentBuilder<'this, 'ctx> {
	slangContext: &'this mut shader::slang::Context<'ctx>,
	addModulesRoot: PathBuf,
}
impl<'this, 'ctx> EnvironmentBuilder<'this, 'ctx>
{
	///
	pub fn new(slangContext: &'this mut shader::slang::Context<'ctx>, addModulesRoot: impl AsRef<Path>) -> Self {
		Self { slangContext, addModulesRoot: addModulesRoot.as_ref().to_owned() }
	}

	///
	pub fn addModule (
		&mut self, envStorage: shader::slang::EnvironmentStorage,
		sourceFilePath: impl AsRef<Path>
	) -> Result<()> {
		let contents = std::fs::read_to_string(self.addModulesRoot.join(sourceFilePath.as_ref()))?;
		Ok(self.slangContext.loadModuleFromSource(envStorage, sourceFilePath.as_ref(), &contents)?)
	}

	///
	pub fn addModuleAtPath (
		&mut self, environmentPath: impl AsRef<Path>, envStorage: shader::slang::EnvironmentStorage,
		sourceFile: impl AsRef<Path>
	) -> Result<()> {
		let contents = std::fs::read_to_string(sourceFile.as_ref())?;
		Ok(self.slangContext.loadModuleFromSource(envStorage, environmentPath.as_ref(), &contents)?)
	}
}



//////
//
// Functions
//

/// Prepare the shaders in the given directory, skipping indicated directory sub-trees.
pub fn prepareShaders (
	buildSetup: &Setup, moduleSourceTargets: Option<&[shader::compile::Target]>,
	shaderDirectory: impl AsRef<Path>, skipSubDirs: Option<&[impl AsRef<Path>]>
) -> Result<()>
{
	// Sanity checks
	// - syntactic
	if shaderDirectory.as_ref().is_absolute() {
		return Err(anyhow!(
			"`shaderDirectory` is not relative: '{}'\nMust be a directory relative to the path of the crate manifest!",
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
	if !absoluteShaderDir.starts_with(fs::canonicalize(crateSrcDir)?) {
		return Err(anyhow!(
			"`shaderDirectory` is not a subdir of the crate source root:\ncrate root: {}\nshader dir: {}",
			crateSrcDir.display(), absoluteShaderDir.display()
		))
	}

	// Determine target root directory for packaged shaders
	let targetDir = std::path::absolute(getCargoOutDir().join(shaderDirectory))?;

	// Determine module source types if none were specified
	let slangContexts = shader::slang::createContextsForTargets(
		moduleSourceTargets.unwrap_or(&[shader::mostSuitableCompilationTargetForPlatform(
			cargoBuildTargetPlatform()
		)]),
		buildSetup.shaderPath().as_slice()
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
			fs::create_dir_all(tgtParent)?;
			let package = shader::Package::fromSlangSourceFileMultiple(&slangContexts, srcPath, None)?;
			package.writeToFile(&tgtPath)?;
			dependOnGeneratedFile(tgtPath)?;
			Ok(())
		}
		else {
			Ok(())
		}
	})?;

	// Mirror the shader structure of the given directory
	Ok(())
}

///
pub fn generateShaderEnvironment (
	environmentFilename: impl AsRef<Path>, addModulesRoot: impl AsRef<Path>, shaderPath: Option<&[impl AsRef<Path>]>,
	label: &str, addModules: impl FnOnce(&mut EnvironmentBuilder, shader::slang::EnvironmentStorage)->Result<()>
) -> Result<()> {
	// Create Slang compilation context
	let mut slangContext = shader::slang::Context::forTarget(
		shader::mostSuitableCompilationTargetForPlatform(cargoBuildTargetPlatform()),
		shaderPath.unwrap_or_default(),
	)?;

	// Create and set target environment
	let env = shader::compile::Environment::forContextWithUuid(
		&slangContext, util::unique::uuidFromUserString(label), label
	);
	slangContext.replaceEnvironment(Some(env))?;

	// Dispatch the builder
	addModules(
		&mut EnvironmentBuilder::new(&mut slangContext, addModulesRoot),
		shader::slang::mostSuitableEnvironmentStorageForPlatform(cargoBuildTargetPlatform())
	)?;

	// Retrieve environment and generate
	let env = slangContext.finishEnvironment().unwrap();
	let actualFilename = getCargoOutDir().join(environmentFilename);
	env.serializeToFile(&actualFilename)?;
	dependOnGeneratedFile(actualFilename)?;

	// Done!
	Ok(())
}
