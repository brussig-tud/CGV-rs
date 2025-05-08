
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
	// Propagate build setup (should be applied by dependent crates via cgv_build::applyBuildSetup())

	// Find current target directory
	let targetDir = cgv_build::getCargoTargetDir()?;

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

	// Done!
	Ok(())
}
