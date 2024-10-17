
// Language config
#![allow(non_snake_case)]

// Imports
use std::env;
use std::fs;
use cargo_metadata::MetadataCommand;

// Custom build steps - in this example, it is used only for WASM support, by invoking the proper wasm-bindgen command,
// auto-generating an appropriate index.html, and packaging everything (including resources like the favicon and app
// manifest) for hosting.
fn main() -> Result<(), Box<dyn std::error::Error>>
{
	// Get general cargo properties
	let pkgName = env::var("CARGO_PKG_NAME").unwrap();
	let outDir = env::var("OUT_DIR").unwrap();

	// Setup cargo-metadata to retrieve our custom fields
	let meta = MetadataCommand::new()
		.manifest_path("./Cargo.toml")
		.current_dir(env::var("CARGO_MANIFEST_DIR").unwrap())
		.exec()
		.unwrap();

	// Extract fields required to generate index.html
	let pkg = meta.root_package().unwrap();
	let niceName =
		if let Some(niceName) = pkg.metadata["nice-name"].as_str() {
			niceName
		}
		else {
			pkgName.as_str()
		};

	// Done!
	Ok(())
}
