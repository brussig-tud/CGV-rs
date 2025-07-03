
//////
//
// Language config
//

// Eff this convention. Probably the worst aspect of Rust after the lack of a standardized ABI
#![allow(non_snake_case)]



//////
//
// Functions
//

/// Custom build steps – collect transitive build properties to propagate to dependent crates via the `BuildSetup`
/// mechanism of CGV-rs and compile and package all internal shaders.
fn main() -> cgv_build::Result<()>
{
	////
	// Preamble

	// Launch VS Code LLDB debugger if it is installed and attach to the build script
	if std::env::var("CARGO_FEATURE_BUILD_SCRIPT_DEBUG").is_ok() {
		cgv_build::debugWithVsCode(true)?;
	}

	// Get source directory
	let cgvSrcDir = cgv_build::getCargoSourceDir();


	////
	// Perform build operations

	// Propagate our build setup (should be applied by dependent crates via cgv_build::applyBuildSetup())
	let mut buildSetup = cgv_build::Setup::default();
	// active features
	buildSetup.cgvFeatures.slang_runtime = std::env::var("CARGO_FEATURE_SLANG_RUNTIME").is_ok();
	buildSetup.cgvFeatures.wayland = std::env::var("CARGO_FEATURE_COPY_LIBS").is_ok();
	buildSetup.cgvFeatures.x11 = std::env::var("CARGO_FEATURE_COPY_LIBS").is_ok();
	// - linker flag in case we have the `copy_libs` feature
	if !cgv_build::isWindows()? && !cgv_build::isWasm()? && buildSetup.cgvFeatures.slang_runtime {
		buildSetup.addLinkerFlag("-Wl,-rpath=$ORIGIN");
	}
	// - our shader path
	buildSetup.addShaderPath(cgvSrcDir.join("shader/lib"));
	buildSetup.addShaderPath(cgvSrcDir.join("shader/lib/api"));
	buildSetup.addShaderPath(cgvSrcDir.join("shader/lib/lin"));
	// - propagate
	buildSetup.injectIntoCargoBuild()?;

	// Compile our shaders
	cgv_build::prepareShaders(&buildSetup, None, "shader", Some(&["common", "gpu", "lib"]))?;

	// Done!
	Ok(())
}
