
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
	cgv_build::applyBuildSetup()?;

	// Deploy a web application if the target architecture is WASM
	cgv_build::webDeployIfWasm("../pkg", &["Cargo.toml"])?;


	////
	// Compile our shaders – TODO: add proper shader building facilities

	// Set up paths
	let srcDir = cgv_build::getCargoSourceDir();
	let cgvShaderDir = cgv_build::cgvCrateDirectory().join("shader/lib");
	let shaderPath = &[
		std::fs::canonicalize(cgvShaderDir.join("lin"))?, std::fs::canonicalize(cgvShaderDir.join("api"))?
	];

	// Manually compile the example shader
	// - set up filenames
	let shaderSrc_example = srcDir.join("shader/example.slang");
	let shaderPak_example = cgv_build::getCargoOutDir().join("example.spk");
	cgv_build::dependOnFile(&shaderSrc_example);
	// - set up compilation targets to include
	let slang2SPIRV = cgv_build::shader::slang::Context::forTarget(
		cgv_build::shader::slang::CompilationTarget::SPIRV(cgv_build::getCargoDebugBuild()?), shaderPath
	)?;
	let slang2WGSL = cgv_build::shader::slang::Context::forTarget(
		cgv_build::shader::slang::CompilationTarget::WGSL, shaderPath
	)?;
	// - compile
	let viewportCompositorPak = cgv_build::shader::Package::fromSlangMultipleContexts(
		&[&slang2SPIRV, &slang2WGSL], shaderSrc_example, None
	)?;
	// - write shader package
	viewportCompositorPak.writeToFile(&shaderPak_example)?;
	cgv_build::dependOnGeneratedFile(shaderPak_example)?;

	// Done!
	Ok(())
}
