
/// Custom build steps – set up build script debugging, apply the build setup as passed on from *CGV-rs*, compile
/// shaders and support WASM deployment.
fn main() -> cgv_build::Result<()>
{
	// Support debugging this build script (currently hard-wired to VS Code until IDEs start providing proper means of
	// build script debugging)
	if std::env::var("CARGO_FEATURE_BUILD_SCRIPT_DEBUG").is_ok() {
		cgv_build::debugWithVsCode(true)?;
	}

	// Apply CGV-rs build setup
	cgv_build::applyBuildSetup()?;

	// Deploy a web application if the target architecture is WASM
	cgv_build::webDeployIfWasm("../pkg", &["Cargo.toml"])
}
