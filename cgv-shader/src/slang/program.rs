
//////
//
// Imports
//

// Standard library
use std::path::Path;

// Anyhow library
use anyhow::*;

// Slang library
use shader_slang as slang;

// Local imports
use crate::slang::{Context, EntryPoint};



//////
//
// Classes
//

///
pub struct Program {
	_linkedProg: slang::ComponentType,
	allEntryPointsProg: slang::Blob,
	entryPointProgs: Vec<EntryPoint>
}
impl Program
{
	#[cfg(not(target_arch="wasm32"))]
	pub(crate) fn finishCreation (
		slangContext: &Context, module: &slang::Module, virtualFilename: impl AsRef<Path>
	) -> Result<Self>
	{
		// Specialize program instances for each entry point
		// - gather components
		let components = {
			let mut components = vec![module.clone().into()];
			for ep in module.entry_points() {
				components.push(ep.clone().into());
			}
			components
		};
		let program = slangContext.session.create_composite_component_type(
			components.as_slice()
		).or_else(|err| Err(
			anyhow!("Instantiating `{}` failed:\n{}", virtualFilename.as_ref().display(), err)
		))?;
		// - link
		let linkedProg = program.link().or_else(|err| Err(
			anyhow!("Linking of `{}` failed:\n{}", virtualFilename.as_ref().display(), err)
		))?;
		// - variant including all entry points
		let allEntryPointsProg = linkedProg.target_code(0).or_else(|err| Err(
			anyhow!("Building of `{}` failed:\n{}", virtualFilename.as_ref().display(), err)
		))?;
		// - bytecode specialized to each entry point
		let entryPointProgs = {
			let mut index = 0;
			module.entry_points().map(|ep| {
				let progBytecode = linkedProg.entry_point_code(index, 0).expect(
					"linked Slang program for selected entry point did not receive compiled code"
				);
				index += 1;
				EntryPoint { slang: ep, progBytecode }
			}).collect::<Vec<_>>()
		};

		// Done!
		Ok(Self { _linkedProg: linkedProg, allEntryPointsProg, entryPointProgs })
	}

	#[cfg(not(target_arch="wasm32"))]
	pub(crate) fn fromSource (slangContext: &Context, virtualFilename: impl AsRef<Path>, sourceCode: &str)
		-> Result<Self>
	{
		// Compile Slang module
		let module = slangContext.compileFromNamedSource(&virtualFilename, sourceCode)?;

		// Common initialization code
		Self::finishCreation(slangContext, &module, virtualFilename)
	}

	#[cfg(not(target_arch="wasm32"))]
	pub(crate) fn fromSourceFile (slangContext: &Context, filename: impl AsRef<Path>) -> Result<Self>
	{
		// Compile Slang module
		let module = slangContext.compile(filename.as_ref())?;

		// Common initialization code
		Self::finishCreation(slangContext, &module, filename)
	}

	///
	#[inline]
	pub fn entryPointProgs (&self) -> &[EntryPoint] {
		&self.entryPointProgs
	}

	///
	#[inline]
	pub fn allEntryPointsProg (&self) -> &[u8] {
		self.allEntryPointsProg.as_slice()
	}
}
