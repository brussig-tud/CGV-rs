
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
	allEntryPointsProg: compile::ProgramCode,
	entryPointProgs: Vec<(&'this str, compile::ProgramCode)>,
	entryPointsMap: BTreeMap<String, usize>,
}
impl Program<'_>
{
	fn finishCreation<'outer, Context: compile::Context> (
		context: &'outer Context, module: &Context::ModuleType<'outer>, target: compile::Target
	) -> anyhow::Result<Self>
	{
		// XXX: This is ugly. We need to rephrase the way we communicate lifetimes in the `compile::model` to dodge this
		// issue. Could potentially be solved by having the context own every component and hand out references only.
		let module = unsafe { &*(module as *const Context::ModuleType<'_>) };

		// Gather components to specialize program for each entry point
		use compile::ComponentRef;
		let mut components = vec![ComponentRef::Module(module)];
		for ep in module.entryPoints() {
			components.push(ComponentRef::EntryPoint(ep));
		}
		let composite = context.createComposite(&components).map_err(
			|err| anyhow!("Pre-linking of `{}` failed:\n{}", module.virtualFilepath().display(), err)
		)?;

		// Link
		let linkedProg = context.linkComposite(&composite).map_err(
			|err| anyhow!("Linking of `{}` failed:\n{}", module.virtualFilepath().display(), err)
		)?;

		// Generate code
		// - variant including all entry points
		let allEntryPointsProg = linkedProg.allEntryPointsCode(target).or_else(|err| Err(
			anyhow!("Code generation for `{}` failed:\n{}", module.virtualFilepath().display(), err)
		))?;
		// - specialized to each entry point
		let entryPointsInfo = module.entryPoints();
		let mut entryPointProgs = Vec::with_capacity(module.entryPoints().len());
		let mut entryPointsMap = BTreeMap::new();
		for index in 0..module.entryPoints().len()
		{
			if let Some(result) = linkedProg.entryPointCode(target, index) {
				let progBytecode = result.map_err(|err| anyhow!(
					"Code generation for entry point {index} of `{}` failed: {err}", module.virtualFilepath().display()
				))?;
				let name = entryPointsInfo[index].name().to_owned();
				entryPointProgs.push((
					unsafe {
						// SAFETY:
						// Strings always live on the heap in Rust, and we know that we won't change the entry point
						// name for the lifetime of this `Program`. Thus, the address of the string contents will
						// remain stable. To the outside, these references will be communicated with the same lifetime
						// as this `Program`, so safe code cannot possibly end up with dangling references.
						&*(name.as_str() as *const str)
					},
					progBytecode));
				entryPointsMap.insert(name, index);
			}
			else {
				return Err(anyhow!("linked Slang program for selected entry point did not receive compiled code"));
			}
		}

		// Done!
		Ok(Self { allEntryPointsProg, entryPointProgs, entryPointsMap })
	}

	pub fn fromSource<Context: compile::Context> (
		context: &Context, target: compile::Target, virtualFilename: impl AsRef<Path>, sourceCode: &str
	) -> anyhow::Result<Self>
	{
		// Compile Slang module
		let module = context.compileFromNamedSource(&virtualFilename, sourceCode)?;

		// Common initialization code
		Self::finishCreation(context, &module, target)
	}

	pub fn fromSourceFile<Context> (context: &Context, target: compile::Target, filename: impl AsRef<Path>)
		-> anyhow::Result<Self>
	where Context: compile::HasFileSystemAccess
	{
		// Compile Slang module
		let module = context.compile(filename.as_ref())?;

		// Common initialization code
		Self::finishCreation(context, &module, target)
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
