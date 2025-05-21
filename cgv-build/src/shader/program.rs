
//////
//
// Imports
//

// Standard library
use std::path::{PathBuf, Path};

// Slang library
use slang;
use slang::Downcast;



//////
//
// Classes
//

///
pub struct Program {
}
impl Program
{
	pub fn fromFile (filename: impl AsRef<Path>) -> Self
	{
		let search_path = std::ffi::CString::new("shaders/directory").unwrap();

		let slangSession = slang::GlobalSession::new().unwrap();

		// All compiler options are available through this builder.
		let session_options = slang::CompilerOptions::default()
			.optimization(slang::OptimizationLevel::High)
			.matrix_layout_row(true);

		let target_desc = slang::TargetDesc::default()
			.format(slang::CompileTarget::Dxil)
			.profile(slangSession.find_profile("sm_6_5"));

		let targets = [target_desc];
		let search_paths = [search_path.as_ptr()];

		let session_desc = slang::SessionDesc::default()
			.targets(&targets)
			.search_paths(&search_paths)
			.options(&session_options);

		let session = slangSession.create_session(&session_desc).unwrap();
		let module = session.load_module("filename.slang").unwrap();
		let entry_point = module.find_entry_point_by_name("main").unwrap();

		let program = session.create_composite_component_type(&[
			module.downcast().clone(), entry_point.downcast().clone(),
		]).unwrap();

		let linked_program = program.link().unwrap();

		// Entry point to the reflection API.
		let reflection = linked_program.layout(0).unwrap();

		let shader_bytecode = linked_program.entry_point_code(0, 0).unwrap();
		Self {
		}
	}
}
