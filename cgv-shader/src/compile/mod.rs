
//////
//
// Module definitions
//

// Submodule defining the compilation model.
mod model;
pub use model::{Module, EntryPoint, Component, Composite, LinkedComposite, ComponentRef}; // re-export

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
	pub use super::{Context, EnvironmentEnabled, Module, EntryPoint, Component, Composite};
}



//////
//
// Imports
//

// Standard library
use std::{error::Error, fmt::{Display, Formatter}};



//////
//
// Errors
//

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
// Traits
//

///
pub trait Context<'this, ModuleType, EntryPointType, CompositeType, LinkedCompositeType>
where
	EntryPointType: EntryPoint, ModuleType: Module<EntryPointType>, CompositeType: Composite,
	LinkedCompositeType: LinkedComposite
{
	///
	fn createComposite (&'this self, components: &[ComponentRef<'this, ModuleType, EntryPointType, CompositeType>])
		-> Result<CompositeType, CreateCompositeError>;

	///
	fn linkComposite (&'this self, composite: &CompositeType) -> Result<LinkedCompositeType, LinkError>;
}


/// The trait of a [`compile::Context`] that is capable of working with [`compile::Environment`](Environment)s.
pub trait EnvironmentEnabled<ModuleType: env::Module>
{
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
	fn replaceEnvironment (&mut self, environment: Option<Environment<ModuleType>>)
		-> Result<Option<Environment<ModuleType>>, SetEnvironmentError>;

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
	fn takeEnvironment (&mut self) -> Option<Environment<ModuleType>> {
		self.replaceEnvironment(None).expect(
			"Context::takeEnvironment: replacing the current environment with `None` should never fail"
		)
	}

	/// Close the context and return its environment if it had one. As the context is consumed by this method,
	/// implementations can skip any and all reinitialization work that might be required to keep the context usable.
	fn finishEnvironment (self) -> Option<Environment<ModuleType>>;

	///
	fn environmentCompatHash (&self) -> u64;
}
