
//////
//
// Module definitions
//

// Submodule defining the traits, structs and errors of the *CGV-rs* shader compilation model.
mod model;
pub use model::{
	Module, EntryPoint, Component, Composite, LinkedComposite, ProgramCode, ComponentRef, TranslateError
}; // re-export

// Submodule implementing compilation environments.
mod environment;
pub use environment::{Environment, AddModuleError}; // re-export

/// Submodule providing assorted facilities for working with [`compile::Environment`](super::Environment)s.
pub mod env {
	// Selected additional re-exports behind shorthand namespace.
	pub use super::environment::{Module, BytesModule};
}

/// The module prelude.
pub mod prelude {
	pub use super::{
		Context, ContextBuilder, WithFilesystemAccess, EnvironmentEnabled, Module, EntryPoint, Component, Composite
	};
}



//////
//
// Imports
//

// Standard library
use std::{error::Error, fmt::{Display, Formatter}, path::{PathBuf, Path}};

// GUID library
use cgv_util::uuid;

// Local imports
use crate::{compile, WgpuSourceType};
use cgv_util as util;



//////
//
// Errors
//

#[derive(Debug)]
pub enum CreateContextError {
	UnsupportedTarget(compile::Target),
	ImplementationDefined(anyhow::Error),
}
impl Display for CreateContextError {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::UnsupportedTarget(target) => format!("unsupported target: {target}"),
			Self::ImplementationDefined(err) => format!("implementation-specific: {err}")
		};
		write!(formatter, "CreateContextError[{desc}]")
	}
}
impl Error for CreateContextError {}

#[derive(Debug)]
pub enum LoadModuleError {
	CompilationError(String),
	InvalidModulePath(PathBuf),
	DuplicatePath(PathBuf)
}
impl Display for LoadModuleError {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::CompilationError(desc) => &format!("Compilation failed: {desc}"),
			Self::InvalidModulePath(path) => &format!("invalid module path: {}", path.display()),
			Self::DuplicatePath(path) => &format!("module already present at path: {}", path.display()),
		};
		write!(formatter, "LoadModuleError[{desc}]")
	}
}
impl Error for LoadModuleError {}

#[derive(Debug)]
pub enum CreateCompositeError {
	ImplementationSpecific(anyhow::Error)
}
impl Display for CreateCompositeError {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::ImplementationSpecific(st) => &format!("nested implementation error: {st}"),
		};
		write!(formatter, "CreateCompositeError[{desc}]")
	}
}
impl Error for CreateCompositeError {}

#[derive(Debug)]
pub enum LinkError {
	ImplementationSpecific(anyhow::Error)
}
impl Display for LinkError {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::ImplementationSpecific(st) => &format!("nested implementation error: {st}"),
		};
		write!(formatter, "LinkError[{desc}]")
	}
}
impl Error for LinkError {}

#[derive(Debug)]
pub enum SetEnvironmentError {
	IncompatibleEnvironment,
	ImplementationSpecific(anyhow::Error)
}
impl Display for SetEnvironmentError {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::IncompatibleEnvironment => "incompatible environment",
			Self::ImplementationSpecific(st) => &format!("nested implementation error: {st}"),
		};
		write!(formatter, "SetEnvironmentError[{desc}]")
	}
}
impl Error for SetEnvironmentError {}



//////
//
// Enums
//

/// Enum describing possible formats of a [`compile::Target`].
#[derive(Debug,Clone,Copy)]
pub enum TargetFormat {
	/// A sequence of arbitrary bytes.
	Binary,

	/// UTF-8 encoded text.
	Text
}
impl Display for TargetFormat {
	fn fmt (&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Binary => write!(f, "binary"),
			Self::Text => write!(f, "text")
		}
	}
}


/// Enum describing possible known shader compilation targets as well as a [`Custom`](Target::Custom) target for
/// supporting targets not currently known to *CGV-rs*.
#[repr(u8)]
#[derive(Debug,Clone,Copy)]
pub enum Target
{
	/// Compile shaders to *SPIR-V*, specifying whether they should be debuggable or not.
	SPIRV,

	/// Transpile shaders to *WebGPU Shading Language*.
	WGSL,

	/// Transpile shaders to *DirectX Intermediate Language*.
	DXIL,

	/// Transpile shaders to *GL Shading Language*.
	GLSL,

	/// Transpile shaders to *High-level Shading Language*.
	HLSL,

	/// Transpile shaders to *Cuda-C++*.
	CudaCpp,

	/// Transpile shaders to the *Metal* shading language.
	Metal,

	/// Compile to a custom target that the [`compile::Context`] supports. Implementations should make a best effort to
	/// select a globally unique [UUID](uuid) (first value) to identify their custom target. The second value indicates
	/// the format of the target. If implementations cannot guarantee [text targets](TargetFormat::Text) are UTF-8, they
	/// might want to choose marking it as a [binary format](TargetFormat::Binary) instead.
	///
	/// If a target comprises several formats (e.g. one text and one binary variant), implementations should introduce
	/// distinct `Custom` targets for them.
	Custom(uuid::Uuid, TargetFormat)
}
impl Target
{
	/// The highest slot any `compile::Target` corresponds to. *CGV-rs* shall adopt the convention that this will always
	/// be equal to the [discriminant](std::mem::Discriminant) of the [`Custom`](compile::Target::Custom) variant.
	///
	/// The type is intentionally kept as `u8` (thus requiring an explicit cast to `usize` for most practical purposes)
	/// to emphasize that this number will always be quite small. Its value will always be one less than
	/// [`compile::Target::NUM_SLOTS`].
	///
	/// # Examples
	///
	/// ```rust
	/// assert_eq!(compile::Target.slot(), compile::Target::MAX_SLOT);
	/// ```
	pub const MAX_SLOT: u8 = {
		// Ensure we stay informed about the primitive representation used for `compile::Target` in case it ever gets
		// changed
		util::assert_eq_size!(std::mem::Discriminant<Target>, u8);

		const DUMMY_CUSTOM: Target = Target::Custom(uuid::Uuid::from_u128(u128::MAX), TargetFormat::Text);
		const MAX: u8 = DUMMY_CUSTOM.slot() as u8;

		// Ensure we stay informed about the highest discriminant value whenever we change `compile::Target`
		util::const_assert_eq!(MAX, 7);
		MAX
	};

	/// The number of slots that an array storing one value per variant of the `compile::Target` enum would have. Will
	/// always be one more than the related [`compile::Target::MAX_SLOT`] constant.
	///
	/// The type is intentionally kept as `u8` (thus requiring an explicit cast to `usize` for most practical purposes)
	/// to emphasize that this number will always be quite small.
	///
	/// # Examples
	///
	/// ```rust
	/// // A map enabling lightning-fast $O(1)$ checks if a compilation target is active, and if yes, which index
	/// // it corresponds to.
	/// type ActiveTargetsMap = [Option<u32>; compile::Target::NUM_SLOTS as usize];
	/// ```
	pub const NUM_SLOTS: u8 = Target::MAX_SLOT + 1;

	///
	#[inline(always)]
	pub fn fromWgpuSourceType (wgpuSourceType: WgpuSourceType) -> Self {
		match wgpuSourceType {
			WgpuSourceType::SPIRV => Self::SPIRV,
			WgpuSourceType::WGSL => Self::WGSL,
			WgpuSourceType::GLSL => Self::GLSL
		}
	}

	/// The corresponding *slot* of a certain target. This will always be one less than [`compile::Target::NUM_SLOTS`]
	/// less-than-or-equal to [`compile::Target::MAX_SLOT`].
	#[inline(always)]
	pub const fn slot (&self) -> usize {
		unsafe {
			// SAFETY:
			// `compile::Target` is a `repr(u8)`, and the Rust specification states that the discriminants of enums with
			// primitive representation may be obtained via pointer casting even if the enum is complex:
			// https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
			*(self as *const Target as *const u8) as usize
		}
	}

	///
	#[inline(always)]
	pub fn isSPIRV (&self) -> bool {
		matches!(self, Self::SPIRV)
	}

	///
	#[inline(always)]
	pub fn isWGSL (&self) -> bool {
		matches!(self, Self::WGSL)
	}

	///
	#[inline(always)]
	pub fn isGLSL (&self) -> bool {
		matches!(self, Self::GLSL)
	}

	///
	#[inline]
	pub fn isText (&self) -> bool {
		match self {
			Self::WGSL | Self::GLSL | Self::HLSL | Self::CudaCpp | Self::Metal => true,
			Self::SPIRV | Self::DXIL => false,
			Self::Custom(_, format) => matches!(format, TargetFormat::Text)
		}
	}

	///
	#[inline]
	pub fn isBinary (&self) -> bool {
		match self {
			Self::WGSL | Self::GLSL | Self::HLSL | Self::CudaCpp | Self::Metal => false,
			Self::SPIRV | Self::DXIL => true,
			Self::Custom(_, format) => matches!(format, TargetFormat::Binary)
		}
	}
}
impl Display for Target {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::SPIRV => write!(f, "SPIR-V"),
			Self::WGSL => write!(f, "WGSL"),
			Self::DXIL => write!(f, "DXIL"),
			Self::GLSL => write!(f, "GLSL"),
			Self::HLSL => write!(f, "HLSL"),
			Self::CudaCpp => write!(f, "Cuda-C++"),
			Self::Metal => write!(f, "Metal"),
			Self::Custom(uuid, fmt) => write!(f, "Custom[uuid={uuid},{fmt}]")
		}
	}
}
impl util::ds::UniqueVecElement for Target {
	type Key<'k> = u8;

	fn key (&self) -> Self::Key<'_> {
		util::assert_eq_size!(std::mem::Discriminant<Target>, u8);
		unsafe {
			// SAFETY: `Discriminant<Target>` has the same size as `u8` (statically asserted above), and all possible
			//          bit patterns form a valid `usize` value.
			std::mem::transmute(std::mem::discriminant(self))
		}
	}
}



//////
//
// Traits
//

///
pub trait ContextBuilder: Default
{
	////
	// Associated types

	///
	type Context: Context;


	////
	// Methods

	///
	fn defaultForPlatform (platform: &util::meta::SupportedPlatform) -> Self;

	///
	fn withTargets (targets: impl AsRef<[Target]>) -> Self;

	///
	#[inline(always)]
	fn withTarget (target: Target) -> Self {
		Self::withTargets(&[target])
	}

	///
	fn addTargets (self, targets: &[compile::Target]) -> Self;

	///
	#[inline(always)]
	fn addTarget (self, target: compile::Target) -> Self {
		self.addTargets(&[target])
	}

	///
	fn build (self) -> Result<Self::Context, compile::CreateContextError>;
}


///
pub trait WithFilesystemAccess
{
	#[inline(always)]
	fn withSearchPath (path: impl AsRef<Path>) -> Self
	where Self: ContextBuilder {
		Self::withSearchPaths(&[path])
	}
	fn withSearchPaths (paths: &[impl AsRef<Path>]) -> Self where Self: ContextBuilder;

	#[inline(always)]
	fn addSearchPath (self, path: impl AsRef<Path>) -> Self
	where Self: ContextBuilder {
		self.addSearchPaths(&[path.as_ref()])
	}
	fn addSearchPaths (self, paths: &[impl AsRef<Path>]) -> Self where Self: ContextBuilder ;
}


///
pub trait Context
{
	////
	// Associated types

	///
	type ModuleType<'module>: Module<Self::EntryPointType<'module>> where Self: 'module;

	///
	type EntryPointType<'ep>: EntryPoint where Self: 'ep;

	///
	type CompositeType<'cp>: Composite;

	///
	type LinkedCompositeType<'lct>: LinkedComposite where Self: 'lct;

	///
	type Builder: ContextBuilder<Context = Self>;


	////
	// Methods

	///
	fn compileFromSource (&self, sourceCode: &str) -> Result<Self::ModuleType<'_>, LoadModuleError>;

	///
	fn compileFromNamedSource (&self, targetPath: impl AsRef<Path>, sourceCode: &str)
		-> Result<Self::ModuleType<'_>, LoadModuleError>;

	///
	fn createComposite<'this> (
		&'this self, components: &[
			ComponentRef<'this, Self::ModuleType<'this>, Self::EntryPointType<'this>, Self::CompositeType<'this>>
		]
	) -> Result<Self::CompositeType<'this>, CreateCompositeError>;

	///
	fn linkComposite<'this> (&'this self, composite: &Self::CompositeType<'_>)
		-> Result<Self::LinkedCompositeType<'this>, LinkError>;
}


/// The trait of a [`compile::Context`] that is capable of working with [`compile::Environment`](Environment)s.
pub trait EnvironmentEnabled
{
	////
	// Associated types

	///
	type ModuleType: env::Module + serde::Serialize+(for<'de> serde::Deserialize<'de>);

	///
	type EnvStorageHint;


	////
	// Methods

	fn loadModule (&mut self, filename: impl AsRef<Path>) -> Result<(), LoadModuleError>;

	///
	fn loadModuleFromSource (
		&mut self, envStorage: Self::EnvStorageHint, virtualFilepath: impl AsRef<Path>, sourceCode: &str
	) -> Result<(), compile::LoadModuleError>;

	///
	fn loadModuleFromIR (&mut self, targetPath: impl AsRef<Path>, bytes: &[u8]) -> Result<(), compile::LoadModuleError>;

	/// Replace the currently active [`compile::Environment`](Environment) with the given one (or `None`).
	///
	/// Note that the environment is *moved* in. The caller *must* lose ownership of the environment because of the
	/// complex uniqueness semantics of compile environments (which are required for sane [merge](Environment::merge)
	/// operations): compiling shader code with an active environment will typically *alter* said environment. It would
	/// be problematic if the caller then retains a copy of the environment that claims to be identical to the one that
	/// underwent unknown changes inside the `compile::Context`.
	///
	/// If the caller wants to do other things with the environment after they plugged it into a `compile::Context`, it
	/// can retrieve it again by passing `None` to this method (or more expressively, via the shorthand
	/// [`EnvironmentEnabled::takeEnvironment`]). This will retake ownership and leave the `compile::Context` without an
	/// active environment.
	///
	/// If the context's sole purpose was to work on an environment, then clients can also reclaim it via
	/// [`EnvironmentEnabled::finishEnvironment`]. This will consume and thus "end" the context, avoiding the potentially
	/// expensive re-initialization for a new/blank environment that some implementations might need to perform.
	///
	/// **Note**: setting a [`compile::Environment`] could involve potentially expensive re-initialization as well as
	/// (re-)compiling the contained [`env::Module`]s for the new context, which can be a very expensive operation, so
	/// clients should try to minimize moving environments in and out of contexts.
	///
	/// # Arguments
	///
	/// * `environment` – The compile environment to replace the current one with.
	///
	/// # Returns
	///
	/// The previous environment if `Ok`, otherwise a [`SetEnvironmentError`] describing the problem.
	fn replaceEnvironment (&mut self, environment: Option<Environment<Self::ModuleType>>)
		-> Result<Option<Environment<Self::ModuleType>>, SetEnvironmentError>;

	/// Take the current [`compile::Environment`] out of the context, leaving `None` in its place.
	///
	/// # Returns
	///
	/// `Some` previous environment if there was one, `None` otherwise.
	///
	/// # Panics
	///
	/// Not under normal circumstances, but a faulty implementation of `compile::Context` might return an error when
	/// [replacing]() the current environment with `None` (which should be guaranteed to succeed) and cause this method
	/// to panic.
	#[inline(always)]
	fn takeEnvironment (&mut self) -> Option<Environment<Self::ModuleType>> {
		self.replaceEnvironment(None).expect(
			"Context::takeEnvironment: replacing the current environment with `None` should never fail"
		)
	}

	/// Close the context and return its environment if it had one. As the context is consumed by this method,
	/// implementations can skip any and all reinitialization work that might be required to keep the context usable.
	fn finishEnvironment (self) -> Option<Environment<Self::ModuleType>>;

	///
	fn environmentCompatHash (&self) -> u64;
}



//////
//
// Functions
//

/// Turn a list of [compilation targets](CompilationTarget) into a list of [contexts](Context) that compile to these
/// targets.
pub fn createContextsForTargets<'a, ContextType> (targets: &[Target], shaderPath: &[impl AsRef<Path>])
	-> anyhow::Result<util::ds::RefVec<'a, ContextType>>
where
	ContextType: compile::Context, ContextType::Builder: WithFilesystemAccess
{
	let mut contexts = Vec::<ContextType>::with_capacity(targets.len());
	for &target in targets {
		contexts.push(ContextType::Builder::withTarget(target).addSearchPaths(shaderPath).build()?);
	}
	Ok(contexts.into())
}

/// Determine the most suitable shader compilation target for the platform the module was built for.
#[inline(always)]
pub fn mostSuitableTarget() -> compile::Target
{
	// WebGPU/WASM
	#[cfg(target_arch="wasm32")] {
		compile::Target::WGSL
	}
	// All native backends (currently always considers SPIR-V preferable even on non-Vulkan backends)
	#[cfg(not(target_arch="wasm32"))] {
		compile::Target::SPIRV
	}
}

/// Determine the most suitable shader compilation target for the given platform.
pub fn mostSuitableTargetForPlatform(platform: &util::meta::SupportedPlatform) -> compile::Target
{
	// WebGPU/WASM
	if platform.isWasm() {
		compile::Target::WGSL
	}
	// All native backends
	else {
		// Currently always considers SPIR-V preferable even on non-Vulkan backends
		// TODO: somehow incorporate notion of WGPU backend into this decision
		compile::Target::SPIRV
	}
}

/// Return a list of feasible shader compilation target for the platform the module was built for, from most to least
/// suitable.
#[inline(always)]
pub fn feasibleTargets() -> &'static [compile::Target]
{
	// WebGPU/WASM
	#[cfg(target_arch="wasm32")]
	const COMPILATION_TARGETS: [compile::Target; 2] = [compile::Target::WGSL, compile::Target::SPIRV];

	// All native backends (currently always considers SPIR-V preferable even on non-Vulkan backends)
	#[cfg(not(target_arch="wasm32"))]
	const COMPILATION_TARGETS: [compile::Target; 2] = [compile::Target::SPIRV, compile::Target::WGSL];

	&COMPILATION_TARGETS
}

/// Return a list of feasible shader compilation target for the given platform, from most to least suitable.
pub fn feasibleTargetsForPlatform(platform: &util::meta::SupportedPlatform) -> &'static [compile::Target]
{
	// WebGPU/WASM
	if platform.isWasm() {
		const COMPILATION_TARGETS: [compile::Target; 2] = [compile::Target::WGSL, compile::Target::SPIRV];
		&COMPILATION_TARGETS
	}
	// All native backends
	else {
		// Currently always considers SPIR-V preferable even on non-Vulkan backends
		if !platform.isDebug() {
			const COMPILATION_TARGETS: [compile::Target; 2] = [compile::Target::SPIRV, compile::Target::WGSL];
			&COMPILATION_TARGETS
		}
		else {
			const COMPILATION_TARGETS: [compile::Target; 2] = [compile::Target::SPIRV, compile::Target::WGSL];
			&COMPILATION_TARGETS
		}
	}
}
