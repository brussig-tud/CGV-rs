
//////
//
// Language config
//

// Eff this convention. Probably the worst aspect of Rust after the lack of a standardized ABI
#![allow(non_snake_case)]

// And this one... the macros are there for clients! Why should the library have to use every single one? WTF...
#![allow(unused_macros)]

// We're pasting whole modules from the cgv crate in here, including many things we don't actually need, due to the
// utterly stupid way Rust build scripts work, so we don't have a choice about this one
#![allow(dead_code)]



//////
//
// Includes
//

// CGV-rs util modules
pub mod util {
	include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../cgv/src/util/mod.rs"));
}



//////
//
// Imports
//

// Standard library
use std::{env, fs};

// Anyhow library
pub use anyhow::{Context, Result};

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
	faviconSourceDir: std::path::PathBuf
}


////
// WebDeploymentBuilder

/// A builder for setting the optional properties of the `index.html` template for web deployment.
#[derive(Default)]
pub struct WebDeploymentBuilder
{
	niceName: Option<String>,
	shortNiceName: Option<String>,
	faviconSourceDir: Option<std::path::PathBuf>
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
	pub fn faviconSourceDir (&mut self, faviconSourceDir: std::path::PathBuf) -> &mut Self {
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

/// Internal utility for recursively copying entire directory trees
pub fn copyRecursively<PathRef: AsRef<std::path::Path>> (source: PathRef, dest: PathRef) -> Result<()>
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

/// Retrieve the base path of the CGV crate.
pub fn cgvBuildCrateDirectory() -> &'static std::path::Path {
	static PATH: std::sync::LazyLock<std::path::PathBuf> = std::sync::LazyLock::new(
		|| env!("CARGO_MANIFEST_DIR").parse::<std::path::PathBuf>().unwrap()
	);
	PATH.as_path()
}

/// Retrieve the path to the favicon resources
pub fn webResourcesDirFavicon () -> &'static std::path::Path {
	static PATH: std::sync::LazyLock<std::path::PathBuf> = std::sync::LazyLock::new(
		|| cgvBuildCrateDirectory().join("web/favicon")
	);
	PATH.as_path()
}

/// Retrieve the path to the `index.html` template for web deployments.
pub fn templateFileIndexHtml () -> &'static std::path::Path {
	static PATH: std::sync::LazyLock<std::path::PathBuf> = std::sync::LazyLock::new(
		|| cgvBuildCrateDirectory().join("web/index.html")
	);
	PATH.as_path()
}

/// Retrieve the path to the `site.webmanifest` template for web deployments.
pub fn templateFileSiteWebmanifest () -> &'static std::path::Path {
	static PATH: std::sync::LazyLock<std::path::PathBuf> = std::sync::LazyLock::new(
		|| cgvBuildCrateDirectory().join("web/site.webmanifest")
	);
	PATH.as_path()
}

/// Process the given template file for the provided [`WebDeployment`].
pub fn instantiateTemplate (filepath: &std::path::Path, webDeployment: &WebDeployment) -> Result<String>
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
pub fn deployCgvApplication (outputPath: &std::path::Path, webDeployment: WebDeployment)
	-> Result<()>
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
///                                 changes into the Cargo `build.rs` re-run decision logic. Therefore, as per cargo
///                                 monitoring rules, the calling build script must make explicit any locations it would
///                                 normally depend on being monitored automatically by Cargo. It can do so here if it
///                                 doesn't already do it elsewhere (passing in an empty slice is perfectly OK).
pub fn webDeployIfWasm (outputPath: &str, changeCheckedFilesOrDirs: &[&str]) -> Result<()>
{
	////
	// Preamble

	// Don't do anything if we're not building for WASM
	let targetArch = env::var("CARGO_CFG_TARGET_ARCH")?;
	if targetArch != "wasm32" {
		return Ok(());
	}

	// Get other relevant general cargo properties
	let manifestPath = env::var("CARGO_MANIFEST_DIR").unwrap().parse::<std::path::PathBuf>()?;
	let pkgName = env::var("CARGO_PKG_NAME")?;


	////
	// Gather metadata

	// First, parse output path
	let outputPath = outputPath.parse::<std::path::PathBuf>()?;

	// Inject re-run decision dependencies
	for dep in changeCheckedFilesOrDirs {
		let dep_absPath = util::path::normalizeToAnchor(
			&manifestPath, &dep.parse::<std::path::PathBuf>()?
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
