
//////
//
// Imports
//

// CGV-rs utility library
use cgv_util as util;

// Local imports
use crate::{*, compile::{BuildsContextWithFilesystemAccess, ContextBuilder, HasFileSystemAccess, Module}};



//////
//
// Functions
//

/// Create a *Slang* compilation context.
fn createContext<'outer> (globalSession: &'outer slang::GlobalSession) -> slang::Context<'outer> {
	slang::ContextBuilder::withTargets(&[compile::Target::SPIRV, compile::Target::WGSL])
		.addSearchPath(util::pathInsideCrate!("/shader/tests"))
		.buildWithGlobalSession(globalSession).expect("failed to create Slang compilation context")
}



//////
//
// Tests
//

#[test]
fn test_compilation_fromFile ()
{
	// Compile test shader
	let gs = slang::GlobalSession::new();
	let module =  createContext(&gs).compile("multiple_entrypoints.slang")
		.expect("failed to compile shader");

	// Check entry points
	assert_eq!(module.entryPoints().len(), 4);
	assert!(module.entryPoint("vertexMain").is_some());
	assert!(module.entryPoint("fragmentMain").is_some());
	assert!(module.entryPoint("computeMain1").is_some());
	assert!(module.entryPoint("computeMain2").is_some());
}

#[test]
fn test_Program_fromFile_binaryTarget ()
{
	// Build test shader
	let gs = slang::GlobalSession::new();
	let program = match Program::fromSourceFile(
		&createContext(&gs), compile::Target::SPIRV,
		util::pathInsideCrate!("/shader/tests/multiple_entrypoints.slang")
	){
		Ok(prog) => prog,
		Err(err) => panic!("failed to create program: {}", err)
	};

	// Check entry points in program
	assert_eq!(program.entryPointProgs().len(), 4);
	assert!(program.entryPointProg("vertexMain").expect("missing entry point").isBinary());
	assert!(program.entryPointProg("fragmentMain").expect("missing entry point").isBinary());
	assert!(program.entryPointProg("computeMain1").expect("missing entry point").isBinary());
	assert!(program.entryPointProg("computeMain2").expect("missing entry point").isBinary());
}

#[test]
fn test_Program_fromFile_textTarget ()
{
	// Build test shader
	let gs = slang::GlobalSession::new();
	let program = match Program::fromSourceFile(
		&createContext(&gs), compile::Target::WGSL,
		util::pathInsideCrate!("/shader/tests/multiple_entrypoints.slang")
	){
		Ok(program) => program,
		Err(err) => panic!("failed to create program: {}", err)
	};

	// Check entry points in program
	assert_eq!(program.entryPointProgs().len(), 4);
	assert!(program.entryPointProg("vertexMain").expect("missing entry point").isText());
	assert!(program.entryPointProg("fragmentMain").expect("missing entry point").isText());
	assert!(program.entryPointProg("computeMain1").expect("missing entry point").isText());
	assert!(program.entryPointProg("computeMain2").expect("missing entry point").isText());
}
