
//////
//
// Imports
//

// Standard library
use std::env;

// CMake crate
use cmake;



//////
//
// Functions
//

/// Custom build steps – builds native dependencies and handles all additional steps that might be required to make them
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
			println!(
				"cargo::warning={}",
				"Building Slang compiler and reflection library. This can take several minutes the first time."
			);
			let _dst = cmake::Config::new("../3rd/slang")
				.profile(if cfg!(debug_assertions) { "Debug" } else { "Release" })
				.build_target("slangc")
				.build();
		}
	}
}
