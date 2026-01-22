
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
pub struct EnvironmentBuilder<'this, CompileContext>
where
	CompileContext: shader::compile::Context + EnvironmentEnabled
{
	context: &'this mut CompileContext,
	addModulesRoot: PathBuf,
}
impl<'this, CompileContext> EnvironmentBuilder<'this, CompileContext>
where
	CompileContext: shader::compile::Context + EnvironmentEnabled
{
	///
	pub fn new(context: &'this mut CompileContext, addModulesRoot: impl AsRef<Path>) -> Self {
		Self { context, addModulesRoot: addModulesRoot.as_ref().to_owned() }
	}

	///
	pub fn addModule (
		&mut self, envStorage: CompileContext::EnvStorageHint,
		sourceFilePath: impl AsRef<Path>
	) -> Result<()> {
		let contents = std::fs::read_to_string(self.addModulesRoot.join(sourceFilePath.as_ref()))?;
		Ok(self.context.loadModuleFromSource(envStorage, sourceFilePath.as_ref(), &contents)?)
	}

	///
	pub fn addModuleAtPath (
		&mut self, environmentPath: impl AsRef<Path>, envStorage: CompileContext::EnvStorageHint,
		sourceFile: impl AsRef<Path>
	) -> Result<()> {
		let contents = std::fs::read_to_string(sourceFile.as_ref())?;
		Ok(self.context.loadModuleFromSource(envStorage, environmentPath.as_ref(), &contents)?)
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
	let slangContexts = shader::compile::createContextsForTargets(
		moduleSourceTargets.unwrap_or(&[shader::compile::mostSuitableTargetForPlatform(
			cargoBuildTargetPlatform()
		)]),
		buildSetup.shaderPath()
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
pub fn generateShaderEnvironment<CompileContextBuilder> (
	contextBuilder: CompileContextBuilder, environmentFilename: impl AsRef<Path>, addModulesRoot: impl AsRef<Path>,
	label: &str, addModules: impl FnOnce(
		EnvironmentBuilder<CompileContextBuilder::Context>, shader::slang::EnvironmentStorage
	)->Result<()>
) -> Result<()>
where
	CompileContextBuilder: shader::compile::ContextBuilder,
	CompileContextBuilder::Context: shader::compile::EnvironmentEnabled
{
	// Create Slang compilation context
	let mut context = contextBuilder.build()?;

	// Create and set target environment
	let env = shader::compile::Environment::forContextWithUuid(
		&context, util::unique::uuidFromUserString(label), label
	);
	context.replaceEnvironment(Some(env))?;

	// Dispatch the builder
	addModules(
		EnvironmentBuilder::new(&mut context, addModulesRoot),
		shader::slang::mostSuitableEnvironmentStorageForPlatform(cargoBuildTargetPlatform())
	)?;

	// Retrieve environment and generate
	let env = context.finishEnvironment().unwrap();
	let actualFilename = getCargoOutDir().join(environmentFilename);
	env.serializeToFile(&actualFilename)?;
	dependOnGeneratedFile(actualFilename)?;

	// Done!
	Ok(())
}
