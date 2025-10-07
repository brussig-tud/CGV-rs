
//////
//
// Imports
//

// Standard library
use std::path::Path;

// Anyhow library
use anyhow::*;
use anyhow::Context as AnyhowContext;

// Slang library
use shader_slang as slang;
use slang::Downcast;

// Local imports
use crate::slang::{Context, EntryPoint};



//////
//
// Classes
//

///
pub struct Program {
	_linkedProg: slang::ComponentType,
	_genericIRBytecode: slang::Blob,
	_genericIRBytecode_bytes: Vec<u8>,
	allEntryPointsBytecode: slang::Blob,
	entryPoints: Vec<EntryPoint>
}
impl Program
{
	pub(crate) fn new (slangContext: &Context, filename: impl AsRef<Path>) -> Result<Self>
	{
		// Compile Slang module
		let module = slangContext.session.load_module(
			filename.as_ref().to_str().context("invalid filename")?,
		).or_else(|err| Err(
			anyhow!("Compilation of `{}` failed:\n{}", filename.as_ref().display(), err)
		))?;
		let _genericIRBytecode = module.serialize()?;
		let _genericIRBytecode_bytes = _genericIRBytecode.as_slice().to_vec();
		let entryPoints = module.entry_points();

		// Link program instances resulting from each entry point
		// - gather components
		let components = {
			let mut components = vec![module.downcast().clone()];
			for ep in entryPoints {
				components.push(ep.downcast().clone());
			}
			components
		};
		let program = slangContext.session.create_composite_component_type(
			components.as_slice()
		).or_else(|err| Err(
			anyhow!("Instantiating `{}` failed:\n{}", filename.as_ref().display(), err)
		))?;
		// - link
		let linkedProg = program.link().or_else(|err| Err(
			anyhow!("Linking of `{}` failed:\n{}", filename.as_ref().display(), err)
		))?;
		// - generic bytecode including all entry points
		let allEntryPointsBytecode = linkedProg.target_code(0).or_else(|err| Err(
			anyhow!("Building of `{}` failed:\n{}", filename.as_ref().display(), err)
		))?;
		// - bytecode specialized to each entry point
		let entryPoints = {
			let mut index = 0;
			module.entry_points().map(|ep| {
				let bytecode = linkedProg.entry_point_code(index, 0).expect("entry point bytecode");
				index += 1;
				EntryPoint { slang: ep, bytecode }
			}).collect::<Vec<_>>()
		};

		// Done!
		Ok(Self { _linkedProg: linkedProg, _genericIRBytecode, _genericIRBytecode_bytes, allEntryPointsBytecode, entryPoints })
	}

	#[inline]
	pub fn entryPoints (&self) -> &[EntryPoint] {
		&self.entryPoints
	}

	#[inline]
	pub fn genericBuildArtifact (&self) -> &[u8] {
		self.allEntryPointsBytecode.as_slice()
	}
}
