
//////
//
// Module definitions
//

/// Submodule implementing the native version of the *Slang* compilation context.
#[cfg(not(target_arch="wasm32"))]
mod native;
#[cfg(not(target_arch="wasm32"))]
pub use native::{
	Context, ContextBuilder, Module, EntryPoint, Composite, LinkedComposite, obtainGlobalSession
}; // re-export

/// Submodule implementing the WASM version of the *Slang* compilation context.
#[cfg(target_arch="wasm32")]
mod wasm;
#[cfg(target_arch="wasm32")]
pub use wasm::{
	Context, ContextBuilder, Module, EntryPoint, Composite, LinkedComposite, obtainGlobalSession
}; // re-export



//////
//
// Imports
//

// Standard library
use std::{error::Error, path::Path, fmt::{Display, Formatter}};

// Serde library
use serde;

// Slang library
#[cfg(not(target_arch="wasm32"))]
use shader_slang as slang;

// CRC64-fast library
#[cfg(not(target_arch="wasm32"))]
use crc64fast_nvme as crc64;

// Local imports
use crate::*;
use crate::compile::AddModuleError;



//////
//
// Errors
//

#[derive(Debug)]
pub enum CreateSessionError {
	Generic,
}
impl Display for CreateSessionError {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::Generic => "generic/unknown"
		};
		write!(formatter, "CreateSessionError[{desc}]")
	}
}
impl Error for CreateSessionError {}



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
pub enum EnvModule {
	/// The module is provided as source code.
	SourceCode(String),

	/// The module is provided in *Slang*-IR form.
	IR(Vec<u8>)
}
impl EnvModule
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
impl compile::env::Module for EnvModule {}



//////
//
// Structs
//

/// Helper struct for encapsulating [compatibility-relevant](Context::environmentCompatHash) Slang session options
#[cfg(not(target_arch="wasm32"))]
struct CompatOptions {
	matrixLayoutColumn: bool
}
#[cfg(not(target_arch="wasm32"))]
impl Default for CompatOptions {
	fn default() -> Self { Self { matrixLayoutColumn: false } }
}
#[cfg(not(target_arch="wasm32"))]
impl CompatOptions {
	///
	#[allow(dead_code)]
	#[inline(always)]
	pub fn matrixLayoutColumn (&mut self) {
		self.matrixLayoutColumn = true;
	}

	///
	#[allow(dead_code)]
	#[inline(always)]
	pub fn matrixLayoutRow (&mut self) {
		self.matrixLayoutColumn = false;
	}

	///
	pub fn toCompilerOptions (&self) -> slang::CompilerOptions {
		let options = slang::CompilerOptions::default();
		if self.matrixLayoutColumn {
			options.matrix_layout_row(false).matrix_layout_column(true)
		} else { options }
	}

	///
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
fn validateModulePath (targetPath: &Path) -> Result<&str, compile::LoadModuleError>
{
	targetPath.parent().ok_or(
		compile::LoadModuleError::InvalidModulePath(targetPath.to_owned())
	)?;
	targetPath.file_stem().ok_or(
		compile::LoadModuleError::InvalidModulePath(targetPath.to_owned())
	)?;

	Ok(targetPath.as_os_str().to_str().ok_or(
		compile::LoadModuleError::InvalidModulePath(targetPath.to_owned())
	)?)
}

///
#[inline]
fn storeInEnvironment (
	environment: Option<&mut compile::Environment<EnvModule>>, atPath: impl AsRef<Path>, module: EnvModule
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
