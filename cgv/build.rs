
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

	// Launch VS Code LLDB debugger if it is installed and attach to the build script
	/*let url = format!(
		"vscode://vadimcn.vscode-lldb/launch/config?{{'request':'attach','pid':{}}}", std::process::id()
	);
	if let Ok(result) = std::process::Command::new("code").arg("--open-url").arg(url).output()
	    && result.status.success() {
		std::thread::sleep(std::time::Duration::from_secs(4)); // <- give debugger time to attach
	}*/

	// Get source directory
	let cgvSrcDir = std::env::var("CARGO_MANIFEST_DIR").map(std::path::PathBuf::from)?;

	// Find current out- and target directories
	let outDir = cgv_build::getCargoOutDir();
	let targetDir = cgv_build::getCargoTargetDirFromOutDir(outDir.as_path())?;


	////
	// Propagate build setup (should be applied by dependent crates via cgv_build::applyBuildSetup())

	// Linker flag in case we have the `copy_libs` feature
	if !cgv_build::isWasm()? && std::env::var("CARGO_FEATURE_COPY_LIBS").is_ok() {
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

	// Proof-of-concept: Manually compile the viewport compositor shader – TODO: add proper shader building facilities
	println!("cargo::rerun-if-changed={}", cgvSrcDir.join("shader/player/viewport.slang").to_str().unwrap());
	let (optLvlArg, debugLvlArg) = {
		let optLevel = std::env::var("OPT_LEVEL").expect(
			"Cargo did not provide the `OPT_LEVEL` environment variable"
		);
		let dbgLevel = std::env::var("DEBUG").map(|dbg|
			match dbg.as_str() {"none" | "false" | "0" => 0, _ => 3 }
		).expect(
			"Cargo did not provide the `DEBUG` environment variable"
		);
		/* evaluate to: */ (
			if optLevel == "0" { format!("-O0") } else { format!("-O3") },
			if dbgLevel ==  0  { format!("-g0") } else { format!("-g3") }
		)
	};
	let slangcOutput = std::process::Command::new("slangc")
		.current_dir(cgvSrcDir)
		.args([
			"./shader/player/viewport.slang", "-profile", "glsl_460", "-target", "spirv",
			"-o", outDir.join("viewport.spv").to_str().unwrap(), optLvlArg.as_str(), debugLvlArg.as_str()
		])
		.output()
		.expect("compileShaderProgram: `slangc` invocation failed");
	if let Err(err) = cgv_build::util::checkProcessOutput(slangcOutput, "slangc") {
		println!("cargo::error=Shader compilation produced errors!");
		panic!("{err}");
	}

	// Done!
	Ok(())
}
