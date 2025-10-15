
//////
//
// Imports
//

// Standard library
use std::{path::PathBuf, error::Error, fmt::{Display, Formatter}};

// UUID library
use uuid;
use uuid::Uuid;

// Local imports
use crate::compile;
use crate::util;



//////
//
// Errors
//

#[derive(Debug)]
pub enum MergeError {
	Incompatible,
	DuplicateModulePaths(PathBuf)
}
impl Display for MergeError
{
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::Incompatible => "incompatible environments",
			Self::DuplicateModulePaths(path) => &format!("duplicate module names: {}", path.display())
		};
		write!(formatter, "MergeError[{desc}]")
	}
}
impl Error for MergeError {}



//////
//
// Traits
//

pub trait Module: Sized+Clone {}

#[derive(Clone)]
pub struct _DummyModule;
impl Module for _DummyModule {}



//////
//
// Structs
//

#[derive(Clone)]
struct ModuleEntry<ModuleType: Module> {
	path: PathBuf,
	module: ModuleType,
	sourceEnv: Option<Uuid>
}
impl<ModuleType: Module> util::ds::UniqueArrayElement<PathBuf> for ModuleEntry<ModuleType> {
	fn key (&self) -> &PathBuf {
		&self.path
	}
}

/// Represents a shader *environment* that, in essence, collects shader modules that should be accessible in conjunction
/// with each other to shader programs that might want to use them.
///
/// Environments should be utilized the following way: *CGV-rs* crates that want to provide shaders should collect all
/// their shaders either during compile-time (preferred) or at initialization, potentially already compiling them to
/// some intermediate representation, and put each one into a [`Module`]. The modules collected this way should then go
/// into an instance of [`Environment`] that the crate will make available through some means of its choosing. Since
/// the modules the crate provides will typically also depend on modules from other crates (the most notable exception
/// being the core `cgv` crate, which only provides self-contained shader functionality), **this environment must be
/// [merged](Environment::mergeWith) with the environments that the other crates provide their modules in** in order to
/// make it usable in a self-contained manner\* (akin to static linking).
///
/// If the collected modules depend on some sort of compiler or preprocessor settings, some hash that uniquely
/// identifies the required configuration should be set which will be checked when [merging](Environment::merge) and
/// which crates using the environment can check to make sure that they are compatible. The crate exporting the
/// environment should naturally provide a way for clients to find and use compatible settings.
///
/// ##### Footnotes
///
/// \*This is merely *CGV-rs* **convention**, in principle crates could require clients to merge in a list of stated
/// dependencies on their side.
#[derive(Clone)]
pub struct Environment<ModuleType: Module> {
	uuid: Uuid,
	label: &'static str,
	compatHash: u64,
	modules: util::ds::UniqueArray<PathBuf, ModuleEntry<ModuleType>>
}
impl<ModuleType: Module> Environment<ModuleType>
{
	/////
	// Helper functions

	/// Return a clone of `moduleEntry` where the source UUID is set to `self` _**iff**_ `moduleEntry`
	/// does not have a source (i.e. it is an internal module).
	#[inline(always)]
	fn externalizeModule (&self, moduleEntry: &ModuleEntry<ModuleType>) -> ModuleEntry<ModuleType> {
		ModuleEntry::<ModuleType> {
			path: moduleEntry.path.clone(),
			module: moduleEntry.module.clone(),
			sourceEnv: if moduleEntry.sourceEnv.is_some() { moduleEntry.sourceEnv } else { Some(self.uuid) }
		}
	}

	#[inline]
	fn externalizeModules (&self) -> util::ds::UniqueArray<PathBuf, ModuleEntry<ModuleType>> {
		let mut newModules = util::ds::UniqueArray::withCapacity(
			self.modules.len()
		);
		for entry in self.modules.iter() {
			newModules.push(self.externalizeModule(&entry)).unwrap()
		}
		newModules
	}


	////
	// Constructors

	/// Construct an empty environment identified by the given
	/// [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier).
	pub fn withUuid (uuid: Uuid, label: &'static str) -> Self { Self {
		uuid, label, compatHash: 0, modules: util::ds::UniqueArray::new()
	}}

	/// Construct an empty environment identified by the given
	/// [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier) and setting compatibility with the provided
	/// [`compile::Context`]
	pub fn forContextWithUuid (context: &impl compile::Context<ModuleType>, uuid: Uuid, label: &'static str) -> Self { Self {
		uuid, label, compatHash: context.environmentCompatHash(), modules: util::ds::UniqueArray::new()
	}}

	/// Construct an empty environment identified by the given
	/// [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier) and an initial compatibility hash.
	pub fn withUuidAndCompatHash (uuid: Uuid, label: &'static str, compatHash: u64) -> Self { Self {
		uuid, label, compatHash, modules: util::ds::UniqueArray::new()
	}}


	////
	// Methods

	/// Clones the current environment with a new UUID and label.
	///
	/// This function creates a new instance of the environment, retaining the compatibility hash and modules of the
	/// current instance. However, **it will mark modules introduced by `self`** (i.e. those that
	/// don't have a source environment) as **externally sourced** from the old UUID.
	///
	/// # Arguments
	///
	/// * `newUuid` – A `Uuid` representing the unique identifier for the new environment.
	/// * `newLabel` – A static string slice representing the label for the new environment.
	///
	/// # Returns
	///
	/// A new instance of the environment with the provided UUID and label, with all modules and the same compatibility
	/// hash as the original.
	pub fn cloneWithNewUuid (&self, newUuid: Uuid, newLabel: &'static str) -> Self
	{
		// Commit all modules, marking the ones owned by `self` as being sourced from `self.uuid`
		let newModules = self.externalizeModules();

		// Build cloned environment
		Self {
			uuid: newUuid, label: newLabel, compatHash: self.compatHash, modules: newModules
		}
	}

	/// Report the [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier) of this environment.
	#[inline(always)]
	pub fn uuid (&self) -> Uuid {
		self.uuid
	}

	/// Reference the label string of this environment.
	#[inline(always)]
	pub fn label (&self) -> &'static str {
		self.label
	}

	/// Report the compatibility hash of this environment.
	#[inline(always)]
	pub fn compatHash (&self) -> u64 {
		self.compatHash
	}

	/// Produce a new `Environment` that is the result of merging `other` to a copy of `self`.
	///
	/// The language in the above sentence is chosen very deliberately – this operation is, technically, **not
	/// commutative**! Merging env 'b' to env 'a' will mean that 'a' will reference 'b'. This will make little
	/// difference in practice, as the modules contained in the merger will be the same no matter the order. But the
	/// *references* semantics will be reverses of each other.
	///
	/// # Arguments
	///
	/// * `other` – The other environment to merge.
	/// * `uuid` – A UUID for the merged environment.
	/// * `label` – The label for the merged environment.
	///
	/// # Returns
	///
	/// The merged environment, or a [`MergeError`] indicating what went wrong.
	pub fn merge (&self, other: &Environment<ModuleType>, uuid: Uuid, label: &'static str) -> Result<Self, MergeError> {
		if self.compatHash != other.compatHash {
			return Err(MergeError::Incompatible);
		}
		let mut newEnv = self.cloneWithNewUuid(uuid, label);
		newEnv.mergeWith(other).map(move |_| newEnv)
	}

	/// Merges another `Environment` into `self`.
	///
	/// # Arguments
	///
	/// * `other` – The other environment to merge with.
	///
	/// # Returns
	///
	/// Nothing in case of success, or a [`MergeError`] indicating what went wrong. In case of any error, `self`
	/// **remains unchanged**!
	pub fn mergeWith (&mut self, other: &Environment<ModuleType>) -> Result<(), MergeError>
	{
		// Initial sanity checks
		// - environments are incompatible
		if self.compatHash != other.compatHash {
			return Err(MergeError::Incompatible);
		}
		// - trying to merge with an identical environment (a no-op)
		if self.uuid == other.uuid {
			return Ok(());
		}

		// Compile list of foreign modules to incorporate
		let mut newModules = util::ds::UniqueArray::withCapacity(other.modules.len());
		for otherEntry in other.modules.iter()
		{
			// Decide what to do with this module
			match self.modules.get(&otherEntry.path)
			{
				// Case 1: this is an unseen module, include
				None => newModules.push(other.externalizeModule(otherEntry)).unwrap(),

				Some(ownEntry) => {
					// Case 2: one of the envs has introduced this module, and the other imported it - don't (re-)import
					// - scenario 2.1: `other` is the original source of the module
					if ownEntry.sourceEnv == Some(other.uuid) {
						assert!(otherEntry.sourceEnv.is_none()); // this assert could only trigger if there was a logic
						continue;                                // bug elsewhere
					}
					// - scenario 2.2: `self` is the original source of the module
					if otherEntry.sourceEnv == Some(self.uuid) {
						assert!(ownEntry.sourceEnv.is_none()); // this assert could only trigger if there was a logic
						continue;                              // bug elsewhere
					}

					// Case 3: we already have a module at this path, and it has the same source: don't import
					if    ownEntry.sourceEnv.is_some() && otherEntry.sourceEnv.is_some()
					   && ownEntry.sourceEnv == otherEntry.sourceEnv {
						continue;
					};

					// Case 4: we already have a module at this path, and it has a different source: ERROR!
					return Err(MergeError::DuplicateModulePaths(otherEntry.path.clone()));
				}
			};
		}

		// Incorporate the new modules
		unsafe {
			// SAFETY: we guaranteed above that the new modules will be unique after merging
			self.modules.join_unchecked(&newModules);
		}

		// Done!
		Ok(())
	}
}
