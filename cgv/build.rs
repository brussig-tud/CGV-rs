
//////
//
// Language config
//

// Eff this convention. Probably the worst aspect of Rust after the lack of a standardized ABI
#![allow(non_snake_case)]



//////
//
// Errors
//

/// An error indicating that an external command invoked via [`std::process::Command`] failed, holding the complete
/// [output](std::process::Output) that the command produced.
#[derive(Debug)]
pub struct CommandFailedError {
	/// A short descriptive name for the command that failed.
	pub command_name: String,
	pub output: std::process::Output
}
impl CommandFailedError
{
	pub fn format_stdstream (formatter: &mut std::fmt::Formatter<'_>, prefix: &str, stream_buf: &[u8])
	                         -> std::fmt::Result {
		for line in String::from_utf8_lossy(stream_buf).lines() {
			writeln!(formatter, "{prefix}{line}")?;
		}
		Ok(())
	}
}
impl std::fmt::Display for CommandFailedError {
	fn fmt (&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(formatter, "CommandFailedError[`{}` -> {}]", self.command_name, self.output.status)?;
		Self::format_stdstream(formatter, " stdout: ", &self.output.stdout)?;
		Self::format_stdstream(formatter, " stderr: ", &self.output.stderr)
	}
}
impl std::error::Error for CommandFailedError {}



//////
//
// Functions
//

/// Check the given [std::process::Output](process output) for errors, emitting *Cargo* output detailing the problem if
/// the output does not indicate success.
fn check_process_output (output: std::process::Output, command_name: impl AsRef<str>) -> Result<(), CommandFailedError>
{
	if !output.status.success() {
		Err(CommandFailedError{ command_name: String::from(command_name.as_ref()), output })
	}
	else {
		Ok(())
	}
}

/// Custom build steps – currently just collects known build variables from dependencies to propagate to dependent
/// crates via the `BuildSetup` mechanism of CGV-rs.
fn main() -> cgv_build::Result<()>
{
	////
	// Preamble

	// Get source directory
	let cgvSrcDir = std::env::var("CARGO_MANIFEST_DIR").map(std::path::PathBuf::from)?;

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

	println!("cargo::rerun-if-changed={}", cgvSrcDir.join("shader/player/viewport.slang").to_str().unwrap());
	let slangcOutput = std::process::Command::new("slangc")
		.current_dir(cgvSrcDir)
		.args([
			"./shader/player/viewport.slang", "-profile", "glsl_460", "-target", "spirv",
			"-o", outDir.join("viewport.spv").to_str().unwrap()//, "-entry", "vertexMain", "-entry", "fragmentMain"
		])
		.output()
		.expect("compileShaderProgram: `slangc` invocation failed");
	if let Err(err) = check_process_output(slangcOutput, "slangc") {
		println!("cargo::error=Shader compilation produced errors!");
		panic!("{err}");
	}

	// Done!
	Ok(())
}
