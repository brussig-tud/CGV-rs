
/// Custom build steps – apply the build setup as passed on from *CGV-rs* and support WASM deployment.
fn main() -> cgv_build::Result<()>
{
	// Apply CGV-rs build setup
	cgv_build::applyBuildSetup()?;

	// Deploy a web application if the target architecture is WASM
	cgv_build::webDeployIfWasm("../pkg", &["Cargo.toml"])
}
