
//////
//
// Language config
//

// Eff this convention.
#![allow(non_snake_case)]

// And this one... the macros are there for clients! Why should the library have to use every single one? WTF...
#![allow(unused_macros)]



//////
//
// Module definitions
//

/// The module implementing the build setup facilities.
mod setup;
pub use setup::Setup; // re-export

/// The module implementing the shader packaging facilities.
pub use cgv_shader as shader;

/// The module providing all kinds of additional assorted utilities specific to building (will get inserted verbatim
/// into the exported [`util`] module).
mod build_util;

/// The compound utilities module
pub mod util {
	// Integrate the regular CGV-rs utils
	pub use cgv_util::*;

	// Integrate our locally defined utils
	pub use crate::build_util::{
		downloadToFile, downloadAndExtract, dependOnDownloadedFile, dependOnDownloadedDirectory, checkProcessOutput
	};
}



//////
//
// Imports
//

// Standard library
use std::{env, fs, path::{Path, PathBuf}};

// Anyhow library
pub use anyhow::{Context, Result, anyhow};

// Cargo Metadata parsing library
use cargo_metadata::MetadataCommand;



//////
//
// Classes
//

////
// WebDeployment

pub struct WebDeployment {
	packageName: String,
	niceName: String,
	shortNiceName: String,
	faviconSourceDir: PathBuf
}


////
// WebDeploymentBuilder

/// A builder for setting the optional properties of the `index.html` template for web deployment.
#[derive(Default)]
pub struct WebDeploymentBuilder
{
	niceName: Option<String>,
	shortNiceName: Option<String>,
	faviconSourceDir: Option<PathBuf>
}

impl WebDeploymentBuilder
{
	pub fn niceName (&mut self, niceName: String) -> &mut Self {
		self.niceName = Some(niceName);
		self
	}
	pub fn shortNiceName (&mut self, shortNiceName: String) -> &mut Self {
		self.shortNiceName = Some(shortNiceName);
		self
	}
	pub fn faviconSourceDir (&mut self, faviconSourceDir: PathBuf) -> &mut Self {
		self.faviconSourceDir = Some(faviconSourceDir);
		self
	}

	pub fn build (&self, packageName: String) -> WebDeployment
	{
		WebDeployment {
			packageName,

			niceName: if let Some(niceName) = &self.niceName {
				niceName.clone()
			}
			else { "CGV Application".into() },

			shortNiceName: if let Some(shortNiceName) = &self.shortNiceName {
				shortNiceName.clone()
			}
			else {
				if let Some(niceName) = &self.niceName { niceName.clone() }
				else { "CGV App".into() }
			},

			faviconSourceDir: if let Some(faviconSourceDir) = &self.faviconSourceDir {
				faviconSourceDir.clone()
			}
			else { webResourcesDirFavicon().into() }
		}
	}
}



//////
//
// Functions
//

/// Report the filename used for storing [Setup](build setups)..
pub fn getSetupFilename () -> &'static str {
	"_CGV_BUILD_SETUP"
}

/// Apply the CGV build setup which will have been stored in the current *target* directory when building the *CGV-rs*
/// crate.
pub fn applyBuildSetup () -> Result<()>
{
	let setupFilename = getCargoTargetDir()?.join(getSetupFilename());
	println!("cargo::rerun-if-changed={}", setupFilename.display());
	if setupFilename.exists() {
		let setup = Setup::fromFile(setupFilename)?;
		setup.apply();
	}
	Ok(())
}

/// Internal utility for recursively copying entire directory trees
pub fn copyRecursively<PathRef: AsRef<Path>> (source: PathRef, dest: PathRef) -> Result<()>
{
	fs::create_dir_all(&dest)?;
	for entry in fs::read_dir(source)?
	{
		let entry = entry?;
		let filetype = entry.file_type()?;
		if filetype.is_dir() {
			copyRecursively(entry.path(), dest.as_ref().join(entry.file_name()))?;
		} else {
			fs::copy(entry.path(), dest.as_ref().join(entry.file_name()))?;
		}
	}
	Ok(())
}

/// Report the root path of source code for the crate that is currently building.
///
/// # Panics
///
/// This function cannot fail if *Cargo* works nominally, so it will panic if there are any problems extracting this
/// information.
pub fn getCargoSourceDir () -> &'static Path {
	static PATH: std::sync::LazyLock<PathBuf> = std::sync::LazyLock::new(||
		env::var("CARGO_MANIFEST_DIR").map(std::path::PathBuf::from)
			.expect("`CARGO_MANIFEST_DIR` should point to a valid path")
	);
	PATH.as_path()
}

/// Report the path of the `OUT_DIR` of the current *Cargo* invocation.
///
/// # Panics
///
/// This function cannot fail if *Cargo* works nominally, so it will panic if there are any problems extracting this
/// information.
pub fn getCargoOutDir () -> &'static Path {
	static PATH: std::sync::LazyLock<PathBuf> = std::sync::LazyLock::new(||
		env::var("OUT_DIR").map(std::path::PathBuf::from)
			.expect("`OUT_DIR` should point to a valid path")
	);
	PATH.as_path()
}

/// Find the path to the *target* directory of the current *Cargo* invocation, based on the provided `outDir` path.
/// Adapted from the following issue: https://github.com/rust-lang/cargo/issues/9661#issuecomment-1722358176
///
/// # Arguments
///
/// * `outDir` – A directory, typically the *Cargo*-assigned `OUT_DIR` of a running build script, that is known to be
///              inside the directory structure of the *target* directory.
pub fn getCargoTargetDirFromOutDir (outDir: &Path) -> Result<PathBuf>
{
	let profile = env::var("PROFILE")?;
	let mut targetDir = None;
	let mut subPath = outDir;
	while let Some(parent) = subPath.parent() {
		if parent.ends_with(&profile) {
			targetDir = Some(parent);
			break;
		}
		subPath = parent;
	}
	targetDir.map(|p| p.to_path_buf()).ok_or_else(
		|| anyhow!("Could not find target directory for profile '{}'", profile)
	)
}

/// Find the path to the *target* directory of the current *Cargo* invocation.
/// Adapted from the following issue: https://github.com/rust-lang/cargo/issues/9661#issuecomment-1722358176
pub fn getCargoTargetDir () -> Result<PathBuf> {
	getCargoTargetDirFromOutDir(getCargoOutDir())
}

/// Retrieve the base path of the `cgv-build` crate.
pub fn cgvBuildCrateDirectory () -> &'static Path {
	static PATH: std::sync::LazyLock<PathBuf> = std::sync::LazyLock::new(
		|| env!("CARGO_MANIFEST_DIR").parse::<PathBuf>().unwrap()
	);
	PATH.as_path()
}

/// Detect whether a build is targeting WASM.
pub fn isWasm () -> Result<bool> {
	Ok(env::var("CARGO_CFG_TARGET_ARCH")? == "wasm32")
}

/// Detect whether a build is targeting Windows.
pub fn isWindows () -> Result<bool> {
	Ok(env::var("CARGO_CFG_TARGET_OS")? == "windows")
}

/// Detect whether a build is targeting Windows.
pub fn isMac () -> Result<bool> {
	Ok(env::var("CARGO_CFG_TARGET_OS")? == "macos")
}

/// Report the debug level set for the current *Cargo* build process.
pub fn getCargoDebugBuild () -> Result<bool>
{
	let dbgLvl = std::env::var("DEBUG").context(
		"Cargo did not provide the `DEBUG` environment variable"
	)?;
	match dbgLvl.as_str() {
		"none" | "false" | "0" => Ok(false), "1" | "true" => Ok(true),
		_ => Err(anyhow!("Invalid value for `DEBUG` environment variable received from Cargo: '{}'", dbgLvl))
	}
}

/// Report the optimization level set for the current *Cargo* build process.
pub fn getCargoOptLevel () -> Result<u32>
{
	let optLevel = std::env::var("OPT_LEVEL").context(
		"Cargo did not provide the `OPT_LEVEL` environment variable"
	)?;
	optLevel.parse::<u32>().context(
		format!("Invalid value for `OPT_LEVEL` environment variable received from Cargo: '{}'", optLevel)
	)
}

/// Report both debug (1st value) and optimization (2nd value) levels set for the current *Cargo* build process.
pub fn getCargoDebugAndOptLevel () -> Result<(bool, u32)> {
	Ok((getCargoDebugBuild()?, getCargoOptLevel()?))
}

/// Retrieve the path to the favicon resources
pub fn webResourcesDirFavicon () -> &'static Path {
	static PATH: std::sync::LazyLock<PathBuf> = std::sync::LazyLock::new(
		|| cgvBuildCrateDirectory().join("web/favicon")
	);
	PATH.as_path()
}

/// Retrieve the path to the `index.html` template for web deployments.
pub fn templateFileIndexHtml () -> &'static Path {
	static PATH: std::sync::LazyLock<PathBuf> = std::sync::LazyLock::new(
		|| cgvBuildCrateDirectory().join("web/index.html")
	);
	PATH.as_path()
}

/// Retrieve the path to the `site.webmanifest` template for web deployments.
pub fn templateFileSiteWebmanifest () -> &'static Path {
	static PATH: std::sync::LazyLock<PathBuf> = std::sync::LazyLock::new(
		|| cgvBuildCrateDirectory().join("web/site.webmanifest")
	);
	PATH.as_path()
}

/// Process the given template file for the provided [`WebDeployment`].
pub fn instantiateTemplate (filepath: &Path, webDeployment: &WebDeployment) -> Result<String>
{
	let template = fs::read_to_string(filepath)?;
	let instantiated =
		template.replace("@PACKAGE_NAME@", webDeployment.packageName.as_str())
		        .replace("@NICE_NAME@", webDeployment.niceName.as_str())
		        .replace("@SHORT_NICE_NAME@", webDeployment.shortNiceName.as_str())
		        .replace("@FAVICON_SOURCE_DIR@", webDeployment.faviconSourceDir.to_str()
		         	.context("property 'faviconSourceDir' is not UTF-8")?
		         );
	Ok(instantiated)
}

/// Deploy a CGV-rs WASM application to the given directory.
pub fn deployCgvApplication (outputPath: &Path, webDeployment: WebDeployment) -> Result<()>
{
	// Instantiate templates
	let indexHtml = instantiateTemplate(templateFileIndexHtml(), &webDeployment)?;
	let siteWebmanifest = instantiateTemplate(templateFileSiteWebmanifest(), &webDeployment)?;

	// Copy resources (also creates the deployment output folder)
	copyRecursively(webDeployment.faviconSourceDir, outputPath.join("res/favicon"))?;

	// Write instantiated templates
	fs::write(outputPath.join("index.html"), indexHtml)?;
	fs::write(outputPath.join("site.webmanifest"), siteWebmanifest)?;

	// Done!
	Ok(())
}

/// Performs a full web deployment of the *CGV-rs* WASM application that the calling build script belongs to.
///
/// # Arguments
///
/// * `outputPath` – The path to deploy to.
/// * `changeCheckedFilesOrPaths` – This function is injecting a new file system location that will be monitored for
///                                 changes into the *Cargo* `build.rs` re-run decision logic. Therefore, as per *Cargo*
///                                 monitoring rules, the calling build script must make explicit any locations it would
///                                 normally depend on being monitored automatically by *Cargo*. It can do so here if it
///                                 doesn't already do it elsewhere (passing in an empty slice is perfectly OK).
pub fn webDeployIfWasm (outputPath: &str, changeCheckedFilesOrDirs: &[&str]) -> Result<()>
{
	////
	// Preamble

	// Don't do anything if we're not building for WASM
	if !isWasm()? {
		return Ok(());
	}

	// Get other relevant general Cargo properties
	let manifestPath = env::var("CARGO_MANIFEST_DIR").unwrap().parse::<PathBuf>()?;
	let pkgName = env::var("CARGO_PKG_NAME")?;


	////
	// Gather metadata

	// First, parse output path
	let outputPath = outputPath.parse::<PathBuf>()?;

	// Inject re-run decision dependencies
	for dep in changeCheckedFilesOrDirs {
		let dep_absPath = util::path::normalizeToAnchor(
			&manifestPath, &dep.parse::<PathBuf>()?
		);
		println!("cargo::rerun-if-changed={}", dep_absPath.as_os_str().to_str().unwrap());
	}
	println!(
		"cargo::rerun-if-changed={}",
		&cgvBuildCrateDirectory().to_str().context("CGV-rs seems to appears to reside at a non-UTF-8 path")?
	);
	let outputPath = util::path::normalizeToAnchor(&manifestPath, &outputPath);
	println!("cargo::rerun-if-changed={}",
		outputPath.as_os_str().to_str().context("`outputPath` contains non-UTF-8 characters")?
	);

	// Setup cargo-metadata to retrieve our custom fields
	let meta = MetadataCommand::new()
		.manifest_path("./Cargo.toml")
		.current_dir(env::var("CARGO_MANIFEST_DIR")?)
		.exec()?;

	// Extract custom metadata fields we might be using
	let pkg = meta.root_package().unwrap();


	////
	// Web deployment

	// Package
	let webDeployment = {
		// Fill all optional properties from the Cargo package metadata
		// - create the builder
		let mut builder = WebDeploymentBuilder::default();
		// - assign properties
		if let Some(niceName) = pkg.metadata["nice-name"].as_str() {
			builder.niceName(niceName.into());
		}
		if let Some(shortNiceName) = pkg.metadata["short-nice-name"].as_str() {
			builder.shortNiceName(shortNiceName.into());
		}
		if let Some(faviconSourceDir) = pkg.metadata["web-favicon-srcdir"].as_str() {
			builder.faviconSourceDir(faviconSourceDir.into());
		}
		// - build
		builder.build(pkgName)
	};
	deployCgvApplication(&outputPath, webDeployment)?;

	// Done!
	Ok(())
}
