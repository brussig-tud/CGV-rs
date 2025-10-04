
//////
//
// Module definitions
//

// Submodule implementing compilation environments
mod environment;
pub use environment::{Module, Environment}; // re-export



//////
//
// Imports
//

// Standard library
use std::{error::Error, fmt::{Display, Formatter}, path::Path};



//////
//
// Errors
//

#[derive(Debug)]
pub enum SetEnvironmentError {
	IncompatibleEnvironment,
	ImplementationSpecific(anyhow::Error)
}
impl Display for SetEnvironmentError {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::IncompatibleEnvironment => "incompatible environment",
			Self::ImplementationSpecific(st) => &format!("nested implementation error: {st}"),
		};
		write!(formatter, "SetEnvironmentError[{desc}]")
	}
}
impl Error for SetEnvironmentError {}

#[derive(Debug)]
pub enum LoadModuleError {
	ImplementationSpecific(anyhow::Error)
}
impl Display for LoadModuleError {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::ImplementationSpecific(st) => &format!("nested implementation error: {st}"),
		};
		write!(formatter, "LoadModuleError[{desc}]")
	}
}
impl Error for LoadModuleError {}



//////
//
// Traits
//

///
pub trait Context<ModuleType: environment::Module>
{
	///
	fn replaceEnvironment (&mut self, environment: Option<&Environment<ModuleType>>) -> Result<(), SetEnvironmentError>;

	///
	fn environmentCompatHash (&self) -> u64;

	///
	fn loadModuleFromDisk (&mut self, filepath: impl AsRef<Path>) -> Result<ModuleType, LoadModuleError>;

	///
	fn loadModuleFromMemory (&mut self, blob: &[u8]) -> Result<ModuleType, LoadModuleError>;
}
