
//////
//
// Module definitions
//

// Submodule implementing compilation environments
mod environment;
pub use environment::Environment; // re-export



//////
//
// Imports
//

// Standard library
/* nothing here yet */



//////
//
// Errors
//

pub enum SetEnvironmentError {
	IncompatibleEnvironment,
	ImplementationSpecific(anyhow::Error)
}

pub enum MergeEnvironmentError {
	IncompatibleEnvironment,
	MergeError(environment::MergeError)
}



//////
//
// Traits
//

///
pub trait Context<ModuleType: environment::Module>
{
	///
	fn replaceEnvironment (&mut self, environment: &Environment<ModuleType>) -> Result<(), SetEnvironmentError>;

	///
	fn mergeEnvironment (&mut self, environment: &Environment<ModuleType>) -> Result<(), MergeEnvironmentError>;
}
