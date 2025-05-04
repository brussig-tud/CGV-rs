
//////
//
// Imports
//

// Standard library
use std::env;

// CMake crate
use cmake;

// Local imports
use cgv_build;
use cgv_build::util;



//////
//
// Functions
//

/// Custom build steps – build native dependencies and handle all additional steps that might be required to make them
/// work for WASM builds.
fn main ()
{
	// Configure and build Slang
	match env::var("CARGO_CFG_TARGET_ARCH").expect("Unable to determine target architecture").as_ref()
	{
		// WASM is not yet supported
		"wasm32" => println!(
			"cargo::warning={}", "WASM target - Slang will be unavailable."
		),

		// Native Slang build
		_ => {
			let slang_install_path = util::path::normalizeToAnchor(
				std::path::Path::new(env::var("CARGO_MANIFEST_DIR").unwrap().as_str()),
				std::path::Path::new("../target/slang-install")
			);
			let _dst = cmake::Config::new("../3rd/slang")
				.profile(/*if cfg!(debug_assertions) { "Debug" } else { */"Release"/* }*/)
				.define("CMAKE_INSTALL_PREFIX", slang_install_path)
				.build();
		}
	}
}
