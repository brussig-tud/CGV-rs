
//////
//
// Imports
//

use std::error::Error;
use std::fmt::{Display, Formatter};
// Standard library
use std::path::Path;

// Local imports
use super::compile;



//////
//
// Errors
//

///
#[derive(Debug)]
pub enum TranslateError {
	InvalidTarget(compile::Target),
	Backend(anyhow::Error)
}
impl Display for TranslateError {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::InvalidTarget(target) => format!("invalid target: {target}"),
			Self::Backend(err) => format!("backend error: {err}")
		};
		write!(formatter, "TranslateError[{desc}]")
	}
}
impl Error for TranslateError {}



//////
//
// Enums
//

///
#[derive(Clone,Debug)]
pub enum ProgramCode {
	///
	Text(String),

	///
	Binary(Vec<u8>)
}
impl ProgramCode
{
	///
	#[inline(always)]
	pub fn isText (&self) -> bool {
		matches!(self, Self::Text(_))
	}

	///
	#[inline(always)]
	pub fn isBinary (&self) -> bool {
		matches!(self, Self::Binary(_))
	}

	/// Obtain a new `Vec` containing the program code. Note that you lose the information whether this was `Text`
	/// or `Binary` code unless you store this information yourself.
	///
	/// # Returns
	///
	/// A `Vec` containing the raw bytes of the program code.
	#[inline(always)]
	pub fn toVec (&self) -> Vec<u8> {
		self.clone().into()
	}
}
impl From<String> for ProgramCode {
	fn from (text: String) -> Self {
		Self::Text(text)
	}
}
impl From<Vec<u8>> for ProgramCode {
	fn from (bin: Vec<u8>) -> Self {
		Self::Binary(bin)
	}
}
impl AsRef<[u8]> for ProgramCode {
	#[inline]
	fn as_ref(&self) -> &[u8] {
		match self {
			Self::Text(text) => text.as_bytes(),
			Self::Binary(bin) => bin.as_slice()
		}
	}
}
impl Into<Vec<u8>> for ProgramCode {
	fn into (self) -> Vec<u8> {
		match self {
			Self::Text(text) => text.as_bytes().to_owned(),
			Self::Binary(bin) => bin.to_owned()
		}
	}
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
	fn allEntryPointsCode (&self, target: compile::Target) -> Result<ProgramCode, TranslateError>;

	///
	fn entryPointCode (&self, target: compile::Target, entryPointIdx: usize) -> Option<Result<ProgramCode, TranslateError>>;
}



//////
//
// Structs
//

///
#[derive(Clone,Copy)]
pub enum ComponentRef<'this, 'ctx, ContextType>
where
	ContextType: compile::Context + ?Sized + 'ctx
{
	Module(&'this ContextType::ModuleType<'ctx>),
	EntryPoint(&'this ContextType::EntryPointType<'ctx>),
	Composite(&'this ContextType::CompositeType<'ctx>)
}
