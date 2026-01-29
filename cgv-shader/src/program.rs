
//////
//
// Imports
//

// Standard library
use std::{collections::BTreeMap, path::Path};

// Anyhow library
use anyhow::anyhow;

// Local imports
use crate::compile::{self, prelude::*};



//////
//
// Classes
//

///
pub struct Program<'this> {
	target: compile::Target,
	allEntryPointsProg: compile::ProgramCode,
	entryPointProgs: Vec<(&'this str, compile::ProgramCode)>,
	entryPointsMap: BTreeMap<String, usize>,
}
impl Program<'_>
{
	///
	pub fn fromSingleModule<'outer, Context: compile::Context> (
		context: &'outer Context, module: &Context::ModuleType<'outer>, target: compile::Target
	) -> anyhow::Result<Self>
	{
		// Build
		let linkedProg = compile::buildModule(context, module).map_err(
			|err| anyhow!("error building '{}':\n{err}", module.virtualFilepath().display())
		)?;

		// Translate and return
		Self::fromLinkedComposite(context, target, &linkedProg).map_err(
			|err| anyhow!("error building '{}':\n{err}", module.virtualFilepath().display())
		)
	}

	///
	pub fn fromLinkedComposite<Context: compile::Context> (
		context: &Context, target: compile::Target, linkedComposite: &Context::LinkedCompositeType<'_>
	) -> anyhow::Result<Self>
	{
		// Eliminate [unused_variables] warning, we're using `context` for type inference of the `Context` generic.
		#[allow(path_statements)] context;

		// Generate code
		// - variant including all entry points
		let allEntryPointsProg = linkedComposite.allEntryPointsCode(target).or_else(
			|err| Err(anyhow!("Code generation failed: {err}"))
		)?;
		// - specialized to each entry point
		let mut entryPointProgs = Vec::with_capacity(linkedComposite.numEntryPoints());
		let mut entryPointsMap = BTreeMap::new();
		for index in linkedComposite.entryPointsIndices()
		{
			let result = linkedComposite.entryPointCode(target, index).expect(
				"requesting code for an entry point that is known to exist should always yield `Some` result \
				even in case of a compilation error"
			);
			let progBytecode = result.map_err(
				|err| anyhow!("Code generation for entry point {index} failed: {err}")
			)?;
			let name = linkedComposite.entryPointName(index).to_owned();
			entryPointProgs.push((
				unsafe {
					// SAFETY:
					// Strings always live on the heap in Rust, and we know that we won't change the entry point
					// name for the lifetime of this `Program`. Thus, the address of the string contents will
					// remain stable. To the outside, these references will be communicated with the same lifetime
					// as this `Program`, so safe code cannot possibly end up with dangling references.
					&*(name.as_str() as *const str)
				},
				progBytecode
			));
			entryPointsMap.insert(name, index);
		}

		// Done!
		Ok(Self { target, allEntryPointsProg, entryPointProgs, entryPointsMap })
	}

	///
	pub fn fromSource<Context: compile::Context> (
		context: &Context, target: compile::Target, virtualFilename: impl AsRef<Path>, sourceCode: &str
	) -> anyhow::Result<Self>
	{
		// Compile Slang module
		let module = context.compileFromNamedSource(&virtualFilename, sourceCode)?;

		// Common initialization code
		Self::fromSingleModule(context, &module, target)
	}

	///
	pub fn fromSourceFile<Context> (context: &Context, target: compile::Target, filename: impl AsRef<Path>)
		-> anyhow::Result<Self>
	where Context: compile::HasFileSystemAccess
	{
		// Compile Slang module
		let module = context.compile(filename.as_ref())?;

		// Common initialization code
		Self::fromSingleModule(context, &module, target)
	}

	///
	#[inline(always)]
	pub fn target (&self) -> compile::Target {
		self.target
	}

	///
	#[inline(always)]
	pub fn entryPointProgs (&self) -> &[(&str, compile::ProgramCode)] {
		&self.entryPointProgs
	}

	///
	#[inline]
	#[allow(dead_code)]
	fn entryPointProgsWithName (&self) -> impl Iterator<Item = (&str, &compile::ProgramCode)> {
		self.entryPointsMap.iter().map(|(name, &idx)| {
			(name.as_str(), &self.entryPointProgs[idx].1)
		})
	}

	///
	#[inline(always)]
	pub fn entryPointProg (&self, name: &str) -> Option<&compile::ProgramCode> {
		if let Some(&index) = self.entryPointsMap.get(name) {
			Some(&self.entryPointProgs[index].1)
		}
		else { None }
	}

	///
	#[inline(always)]
	pub fn allEntryPointsProg (&self) -> &compile::ProgramCode {
		&self.allEntryPointsProg
	}
}
