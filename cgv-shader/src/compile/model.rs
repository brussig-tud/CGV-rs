
//////
//
// Imports
//

// Standard library
use std::path::Path;

// Local imports
use super::compile;



//////
//
// Errors
//

///
pub enum TranslateError {
	UnsupportedTarget,
	ImplementationSpecific(anyhow::Error)
}



//////
//
// Enums
//

///
pub enum ProgramCode {
	Text(String),
	Binary(Vec<u8>)
}



//////
//
// Traits
//

/// The trait of re-usable snippets of shader program code with granularity of at most (but potentially smaller than) a
/// single [`compile::Module`].
pub trait Component {
	type Id: Clone+PartialOrd;

	///
	fn id (&self) -> Self::Id;
}


/// The trait of entry points in a [`compile::Module`].
pub trait EntryPoint: Component {
	///
	fn name (&self) -> &str;
}


/// The trait of modules that contain shader program code managed by a [`compile::Context`].
pub trait Module<EntryPointType: EntryPoint>: Component
{
	///
	fn virtualFilepath (&self) -> &Path;

	///
	fn entryPoint (&self, name: &str) -> Option<&EntryPointType>;

	///
	fn entryPoints (&self) -> &[EntryPointType];
}


/// The trait of a combination of program snippets from whole [`compile::Module`]s and/or individual
/// [`EntryPoint`]s.
pub trait Composite: Component {}


/// The trait of a [`Composite`] that has been linked into a functional shader program.
pub trait LinkedComposite {
	///
	fn allEntryPointsCode (target: compile::Target) -> Result<ProgramCode, TranslateError>;

	///
	fn entryPointCode (target: compile::Target, entryPointIdx: u32) -> Option<Result<ProgramCode, TranslateError>>;
}



//////
//
// Structs
//

///
#[derive(Clone,Copy)]
pub enum ComponentRef<'c, ModuleType, EntryPointType, CompositeType>
where
	EntryPointType: EntryPoint, ModuleType: Module<EntryPointType>, CompositeType: Composite
{
	Module(&'c ModuleType),
	EntryPoint(&'c EntryPointType),
	Composite(&'c CompositeType)
}
