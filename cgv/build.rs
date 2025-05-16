
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

/// Custom build steps – currently just collects known build variables from dependencies to propagate to dependent
/// crates via the `BuildSetup` mechanism of CGV-rs.
fn main() -> cgv_build::Result<()>
{
	////
	// Preamble

	// Get source directory
	let cgvSrcDir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

	// Find current out- and target directories
	let outDir = cgv_build::getCargoOutDir();
	let targetDir = cgv_build::getCargoTargetDirFromOutDir(outDir.as_path())?;


	////
	// Propagate build setup (should be applied by dependent crates via cgv_build::applyBuildSetup())

	// Linker flag in case we have the `copy_libs` feature
	if std::env::var("CARGO_FEATURE_COPY_LIBS").is_ok() {
		let data = "ADDITIONAL_LINKER_ARGS=-Wl,-rpath=$ORIGIN";
		std::fs::write(targetDir.join("_CGV_BUILD_SETUP"), data).or(
			Err(cgv_build::anyhow!("Could not write build setup file"))
		)?;
	}
	else {
		// Currently, the `copy_libs` feature is the only thing giving us any build setup at all, so we don't need the
		// build setup file
		std::fs::remove_file(targetDir.join("_CGV_BUILD_SETUP")).ok();
	}

	////
	// Compile our shaders

	// slangc hello-world.slang -profile glsl_450 -target spirv -o hello-world.spv -entry computeMain
	/*std::process::Command::new("slangc")
		.current_dir(cgvSrcDir)
		.args([
			"./shader/player/viewport.slang", "-profile", "glsl_460", "-target", "spirv",
			"-o", outDir.join("viewport.spv").to_str().unwrap(),
			"-entry", "computeMain"
		])
		.output()
		.expect("compileShaderProgram: `slangc` invocation failed");*/

	// Done!
	Ok(())
}
