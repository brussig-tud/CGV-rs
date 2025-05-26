
//////
//
// Imports
//

// Standard library
use std::path::Path;

// Slang library
use slang;

// Local imports
use crate::*;



//////
//
// Structs
//

///
pub struct EntryPoint {
	pub slang: slang::EntryPoint,
	bytecode: slang::Blob,
}
impl EntryPoint {
	#[inline]
	pub fn buildArtifact (&self) -> &[u8] {
		self.bytecode.as_slice()
	}
}



//////
//
// Classes
//

///
pub struct Program {
	linkedProg: slang::ComponentType,
	genericBytecode: slang::Blob,
	entryPoints: Vec<EntryPoint>
}
impl Program
{
	pub(crate) fn new (slangContext: &SlangContext, filename: impl AsRef<Path>) -> Result<Self>
	{
		// Compile Slang module
		let module = slangContext.session.load_module(
			filename.as_ref().to_str().context("invalid filename")?,
		).or_else(|err| Err(
			anyhow!("Compilation of `{}` failed:\n{}", filename.as_ref().display(), err)
		))?;
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
		let linkedProg = program.link().or_else(|err| Err(
			anyhow!("Linking of `{}` failed:\n{}", filename.as_ref().display(), err)
		))?;
		let genericBytecode = linkedProg.target_code(0).or_else(|err| Err(
			anyhow!("Building of `{}` failed:\n{}", filename.as_ref().display(), err)
		))?;

		let entryPoints = {
			let mut index = 0;
			module.entry_points().map(|ep| {
				let bytecode = linkedProg.entry_point_code(index, 0).expect("entry point bytecode");
				index += 1;
				EntryPoint { slang: ep, bytecode }
			}).collect::<Vec<_>>()
		};

		Ok(Self { linkedProg, genericBytecode, entryPoints })
	}

	#[inline]
	pub fn entryPoints (&self) -> &[EntryPoint] {
		&self.entryPoints
	}

	#[inline]
	pub fn genericBuildArtifact (&self) -> &[u8] {
		self.genericBytecode.as_slice()
	}
}
