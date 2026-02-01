
/// Custom build steps – set up build script debugging, apply the build setup as passed on from *CGV-rs*, compile
/// shaders and support WASM deployment.
#[allow(non_snake_case)]
fn main() -> cgv_build::Result<()>
{
	// Support debugging this build script (currently hard-wired to VS Code until IDEs start providing proper means of
	// build script debugging)
	if std::env::var("CARGO_FEATURE_BUILD_SCRIPT_DEBUG").is_ok() {
		cgv_build::debugWithVsCode(true)?;
	}

	// Apply CGV-rs build setup
	let buildSetup = cgv_build::applyBuildSetup()?;
	if !cgv_build::isWasm()? {
		// also get an "ENVIRONMENT.yaml" file for non-WASM builds
		cgv_build::generateRuntimeEnvironmentFile(&buildSetup)?;
	}

	// Deploy a web application if the target architecture is WASM
	cgv_build::webDeployIfWasm(
		cgv_build::getCargoWorkspaceRootDir().join("./pkg/ex-shaders"), &buildSetup,
		&["Cargo.toml"]
	)?;

	// Done!
	Ok(())
}
