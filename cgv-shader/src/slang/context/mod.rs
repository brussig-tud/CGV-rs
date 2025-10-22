
//////
//
// Module definitions
//

/// Submodule implementing the native version of the *Slang* compilation context.
#[cfg(not(target_arch="wasm32"))]
mod native;
#[cfg(not(target_arch="wasm32"))]
pub use native::Context; // re-export

/// Submodule implementing the WASM version of the *Slang* compilation context.
#[cfg(target_arch="wasm32")]
mod wasm;
#[cfg(target_arch="wasm32")]
pub use wasm::{Context, testJsInterop}; // re-export



//////
//
// Imports
//

// Standard library
use std::{error::Error, borrow::Cow, path::{PathBuf, Path}, fmt::{Display, Formatter}};

// Serde library
use serde;

// Slang library
#[cfg(not(target_arch="wasm32"))]
use shader_slang as slang;

// CRC64-fast library
use crc64fast_nvme as crc64;

// Local imports
use crate::*;
use crate::compile::AddModuleError;



//////
//
// Errors
//

#[derive(Debug)]
pub enum LoadModuleError {
	CompilationError(String),
	InvalidModulePath(PathBuf),
	DuplicatePath(PathBuf)
}
impl Display for LoadModuleError {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::CompilationError(desc) => &format!("Compilation failed: {desc}"),
			Self::InvalidModulePath(path) => &format!("invalid module path: {}", path.display()),
			Self::DuplicatePath(path) => &format!("module already present at path: {}", path.display()),
		};
		write!(formatter, "LoadModuleError[{desc}]")
	}
}
impl Error for LoadModuleError {}



//////
//
// Enums
//

/// Indicates in what form a [`slang::Context`](Context) should enter modules into the active [`compile::Environment`]:
///
/// * `SourceCode` – The module should be stored as source code.
/// * `IR` – The module should be stored in *Slang*-IR form.
#[derive(Clone,Copy,serde::Serialize,serde::Deserialize)]
pub enum EnvironmentStorage {
	/// The module should be stored as source code.
	SourceCode,

	/// The module should be stored in *Slang*-IR form.
	IR
}

///
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub enum Module {
	/// The module is provided as source code.
	SourceCode(String),

	/// The module is provided in *Slang*-IR form.
	IR(Vec<u8>)
}
impl Module
{
	///
	#[cfg(not(target_arch="wasm32"))]
	#[inline(always)]
	fn fromSlangModule (slangModule: slang::Module) -> anyhow::Result<Self> {
		Ok(Self::IR(slangModule.serialize()?.as_slice().to_owned()))
	}

	#[inline(always)]
	#[allow(dead_code)] // CGV-rs itself only uses this for WASM builds, but it could still be useful for clients
	fn fromSlangIRBytes (bytes: &[u8]) -> Self {
		Self::IR(bytes.to_owned())
	}

	///
	#[inline(always)]
	fn fromSlangSourceCode (sourceCode: &str) -> Self {
		Self::SourceCode(sourceCode.to_owned())
	}
}
impl compile::Module for Module {}



//////
//
// Structs
//

/// Helper struct for encapsulating [compatibility-relevant](Context::environmentCompatHash) Slang session options
#[derive(Default)]
#[cfg(not(target_arch="wasm32"))]
struct CompatOptions {
	matrixLayoutColumn: bool,
	matrixLayoutRow: bool,
	optimize: bool
}
#[cfg(not(target_arch="wasm32"))]
impl CompatOptions {
	pub fn matrixLayoutColumn(&mut self, enable: bool) -> bool {
		self.matrixLayoutColumn = enable;
		enable
	}

	pub fn matrixLayoutRow(&mut self, enable: bool) -> bool {
		self.matrixLayoutRow = enable;
		enable
	}

	pub fn optimize(&mut self, enable: bool) -> slang::OptimizationLevel {
		self.optimize = enable;
		if enable { slang::OptimizationLevel::Maximal } else { slang::OptimizationLevel::None }
	}

	pub fn digest (self) -> u64 {
		let mut digest = crc64::Digest::new();
		digest.write(util::slicify(&self));
		digest.sum64()
	}
}



//////
//
// Functions
//

///
fn validateModulePath (targetPath: &Path) -> Result<&str, LoadModuleError>
{
	targetPath.parent().ok_or(
		LoadModuleError::InvalidModulePath(targetPath.to_owned())
	)?;
	targetPath.file_stem().ok_or(
		LoadModuleError::InvalidModulePath(targetPath.to_owned())
	)?;

	Ok(targetPath.as_os_str().to_str().ok_or(
		LoadModuleError::InvalidModulePath(targetPath.to_owned())
	)?)
}

///
#[inline]
fn /*decompose*/encodeValidModulePath (targetPath: &Path) -> /*(*/Cow<'_, str>//, Cow<'_, str>)
{
	targetPath.parent().ok_or(
		LoadModuleError::InvalidModulePath(targetPath.to_owned())
	).unwrap();
	targetPath.file_stem().ok_or(
		LoadModuleError::InvalidModulePath(targetPath.to_owned())
	).unwrap();

	targetPath.as_os_str().to_string_lossy()
}

///
#[inline]
fn storeInEnvironment (
	environment: Option<&mut compile::Environment<Module>>, atPath: impl AsRef<Path>, module: Module
) -> Result<(), AddModuleError>
{
	if let Some(env) = environment {
		// If we got an environment, put the module in it
		env.addModule(atPath, module)
	}
	else {
		// No environment, nothing to do
		Ok(())
	}
}
