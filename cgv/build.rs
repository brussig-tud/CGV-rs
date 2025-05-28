
//////
//
// Language config
//

// Allow debugging the build script
#![allow(internal_features)]
#![feature(core_intrinsics)]

// Eff this convention. Probably the worst aspect of Rust after the lack of a standardized ABI
#![allow(non_snake_case)]



//////
//
// Imports
//

// Anyhow library
use cgv_build::anyhow;



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

	// Launch VS Code LLDB debugger if it is installed and attach to the build script
	if std::env::var("CARGO_FEATURE_BUILD_SCRIPT_DEBUG").is_ok() {
		cgv_build::debugWithVsCode(true)?;
	}

	// Get source directory
	let cgvSrcDir = cgv_build::getCargoSourceDir();

	// Find current out- and target directories
	let outDir = cgv_build::getCargoOutDir();
	let targetDir = cgv_build::getCargoTargetDirFromOutDir(outDir)?;


	////
	// Propagate build setup (should be applied by dependent crates via cgv_build::applyBuildSetup())

	// The path
	let buildSetupPath = targetDir.join("_CGV_BUILD_SETUP");
	// Linker flag in case we have the `copy_libs` feature
	if !cgv_build::isWindows()? && !cgv_build::isWasm()? && std::env::var("CARGO_FEATURE_COPY_LIBS").is_ok() {
		let data = "ADDITIONAL_LINKER_ARGS=-Wl,-rpath=$ORIGIN";
		std::fs::write(&buildSetupPath, data).or(Err(cgv_build::anyhow!("Could not write build setup file")))?;
	}
	else {
		// Currently, the `copy_libs` feature is the only thing giving us any build setup at all, so we'll just end up
		// with an empty build file.
		std::fs::write(&buildSetupPath, "").or(Err(cgv_build::anyhow!("Could not write build setup file")))?;
	}
	cgv_build::util::setTimestampToBeforeBuildScriptTime(buildSetupPath);


	////
	// Compile our shaders

	// Proof-of-concept: Manually compile the viewport compositor shader – TODO: add proper shader building facilities
	let shaderSrc_viewport_slang = cgvSrcDir.join("shader/player/viewport.slang");
	let shaderPak_viewport_pak = outDir.join("viewport.spv");
	println!("cargo::rerun-if-changed={}", shaderSrc_viewport_slang.to_str().ok_or(anyhow!("Invalid path"))?);

	// Test proper shader compilation
	let slang = cgv_build::shader::SlangContext::new(
		&[cgv_build::getCargoSourceDir().join("shader/lib")]
	)?;
	let viewportCompositorProg = slang.buildProgram(cgvSrcDir.join("shader/player/viewport.slang"))?;
	std::fs::write(shaderPak_viewport_pak.as_path(), viewportCompositorProg.genericBuildArtifact())?;
	cgv_build::util::setTimestampToBeforeBuildScriptTime(&shaderPak_viewport_pak);
	println!("cargo::rerun-if-changed={}", shaderPak_viewport_pak.to_str().ok_or(anyhow!("Invalid path"))?);

	// Done!
	Ok(())
}
