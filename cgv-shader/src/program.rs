
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
// Classes
//

///
pub struct Program {
	linkedProg: slang::ComponentType,
	bytecode: slang::Blob
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
		//let entry_point = module.find_entry_point_by_name("main").unwrap();

		let program = slangContext.session.create_composite_component_type(&[
			module.downcast().clone() //, entry_point.downcast().clone(),
		]).unwrap();
		let linkedProg = program.link().or_else(|err| Err(
			anyhow!("Linking of `{}` failed:\n{}", filename.as_ref().display(), err)
		))?;
		let bytecode = linkedProg.target_code(0).or_else(|err| Err(
			anyhow!("Building of `{}` failed:\n{}", filename.as_ref().display(), err)
		))?;

		Ok(Self { linkedProg, bytecode })
	}
}
