
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
		let module = slangContext.session.load_module(
			filename.as_ref().to_str().context("invalid filename")?,
		).or_else(|err| Err(
			anyhow!("Compilation of `{}` failed:\n{}", filename.as_ref().display(), err)
		))?;

		let program = slangContext.session.create_composite_component_type(&[
			module.downcast().clone() //, entry_point.downcast().clone(),
		]).unwrap();
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
}
