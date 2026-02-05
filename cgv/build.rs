
//////
//
// Language config
//

// Eff this convention. Probably the worst aspect of Rust after the lack of a standardized ABI
#![allow(non_snake_case)]



//////
//
// Imports
//

// CGV-rs
use cgv_build::shader::compile::prelude::*;



// ////
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
	// - propagate
	buildSetup.injectIntoCargoBuild()?;

	// Generate the runtime shader compilation environment exposing the core CGV shader library
	cgv_build::generateShaderEnvironment(
		cgv_build::shader::slang::ContextBuilder::withPlatformDefaults(cgv_build::cargoBuildTargetPlatform())
			.addSearchPaths(buildSetup.shaderPath()),
		"coreshaderlib.env", "shader/lib",
		"CgvCoreShaderLib", |mut env, recommendedStorage| {
			env.addModule(recommendedStorage, "cgv/common.slang")?;
			env.addModule(recommendedStorage, "cgv/math/misc.slang")?;
			env.addModule(recommendedStorage, "cgv/geom/common.slang")?;
			env.addModule(recommendedStorage, "cgv/api/uniforms.slang")?;
			env.addModule(recommendedStorage, "cgv/lin/operators.slang")?;
			env.addModule(recommendedStorage, "cgv/lin/transform.slang")?;
			env.addModule(recommendedStorage, "cgv/gpu/filter.slang")?;
			env.addModule(recommendedStorage, "cgv/gpu/filter/box-polyphase.slang")
		}
	)?;

	// Compile our internally used shaders
	cgv_build::prepareShaders(
		&buildSetup, None, "shader", /* exclude: */Some(&["common", "gpu", "lib"])
	)?;

	// Done!
	Ok(())
}
