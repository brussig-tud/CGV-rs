
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
use ::slang;
use slang::Downcast;

// Local imports
use crate::*;



//////
//
// Structs & enums
//

/// Enum describing the compilation target of a [`slang::Context`](Context).
#[derive(Debug)]
pub enum CompilationTarget {
	/// Compile shaders to *SPIR-V*, specifying whether they should be debuggable or not.
	SPIRV(/* debug: */bool),

	/// Transpile shaders to *WGSL*.
	WGSL
}

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
pub struct Context {
	#[allow(dead_code)] // we need to keep this around as it dictates the lifetime of `session`
	globalSession: slang::GlobalSession,

	pub(crate) session: slang::Session,

	pub compilationTarget: SourceType
}
impl Context
{
	/// Create a new Slang context for the given compilation target using the given module search path.
	///
	/// # Arguments
	///
	/// * `target` – The target representation this `Context` will compile/transpile to.
	/// * `searchPath` – The module search path for the *Slang* compiler.
	pub fn forTarget (target: CompilationTarget, searchPath: &[impl AsRef<Path>]) -> anyhow::Result<Self>
	{
		// Convert search path for FFI
		// - create owned storage for the CStrings
		let searchPaths = searchPath.iter().map(|p| unsafe {
			std::ffi::CString::from_vec_unchecked(p.as_ref().to_string_lossy().as_bytes().to_vec())
		}).collect::<Vec<std::ffi::CString>>();
		// - build array of raw pointers required by the FFI
		let searchPaths = searchPaths.iter().map(|p|
			p.as_ptr()
		).collect::<Vec<*const i8>>();

		// Start a Slang global session
		let globalSession = slang::GlobalSession::new();
		let globalSession = if globalSession.is_some() {
			globalSession.unwrap()
		}
		else {
			return Err(anyhow!("Failed to create Slang global session"));
		};

		// Finalize the slang context with our CGV-rs specific options
		// - compile flags
		let sessionOptions = slang::CompilerOptions::default()
			.matrix_layout_row(false)
			.matrix_layout_column(true)
			.language(slang::SourceLanguage::Glsl);
		let sessionOptions = match target
		{
			CompilationTarget::SPIRV(debug) => sessionOptions
				.emit_spirv_directly(true)
				.optimization(
					if debug { slang::OptimizationLevel::None } else { slang::OptimizationLevel::Maximal }
				)
				.debug_information(
					if debug { slang::DebugInfoLevel::Maximal } else { slang::DebugInfoLevel::None }
				),

			CompilationTarget::WGSL => sessionOptions
				.optimization(slang::OptimizationLevel::Maximal)
				.debug_information(slang::DebugInfoLevel::None)
		};
		// - output profile
		let compilationTarget;
		let targetDesc = slang::TargetDesc::default()
			.profile(globalSession.find_profile("glsl_460"));
		let targetDesc = match target {
			CompilationTarget::SPIRV(_) => {
				compilationTarget = SourceType::SPIRV;
				targetDesc.format(slang::CompileTarget::Spirv)
			},
			CompilationTarget::WGSL => {
				compilationTarget = SourceType::WGSL;
				targetDesc.format(slang::CompileTarget::Wgsl)
			}
		};

		let targets = &[targetDesc];
		// - the reusable compiler session
		let session = globalSession.create_session(&slang::SessionDesc::default()
			.targets(targets)
			.search_paths(searchPaths.as_slice())
			.options(&sessionOptions)
		);
		let session = if session.is_some() {
			session.unwrap()
		}
		else {
			return Err(anyhow!("Failed to create Slang context"));
		};

		// Done!
		Ok(Self {	globalSession, session, compilationTarget })
	}

	/// Create a new Slang context with the given module search path. The target platform is automatically detected
	/// before delegating to [`Self::forPlatform`].
	///
	/// # Arguments
	///
	/// * `searchPath` – The module search path for the *Slang* compiler.
	pub fn new (searchPath: &[impl AsRef<Path>]) -> anyhow::Result<Self>
	{
		#[cfg(not(target_arch="wasm32"))] {
			Self::forTarget(CompilationTarget::SPIRV(cfg!(debug_assertions)), searchPath)
		}
		#[cfg(target_arch="wasm32")] {
			Self::forTarget(CompilationTarget::WGSL, searchPath)
		}
	}

	/// Build a shader program from the given *Slang* source file.
	///
	/// # Arguments
	///
	/// * `sourceFile` – The `.slang` file containing the shader source code.
	pub fn buildProgram (&self, sourceFile: impl AsRef<Path>) -> Result<Program> {
		Program::new(self, sourceFile)
	}
}

///
pub struct Program {
	linkedProg: slang::ComponentType,
	genericBytecode: slang::Blob,
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
		let genericBytecode = linkedProg.target_code(0).or_else(|err| Err(
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
