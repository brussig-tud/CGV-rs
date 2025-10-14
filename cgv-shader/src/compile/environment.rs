
//////
//
// Imports
//

// Standard library
use std::{
	collections::BTreeMap, path::PathBuf, error::Error, fmt::{Display, Formatter}
};

// Tracing library
//#[cfg(feature="tracing_output")]
//use tracing;

// UUID library
use uuid;
use uuid::Uuid;

// Local imports
use crate::compile;



//////
//
// Errors
//

#[derive(Debug)]
pub enum MergeError {
	Incompatible,
	DuplicateModuleNames(String)
}
impl Display for MergeError
{
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::Incompatible => "incompatible environments",
			Self::DuplicateModuleNames(st) => &format!("duplicate module names: {st}")
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

/*#[derive(Clone)]
pub struct Environment<ModuleType: Module> {
	uuid: Uuid,
	compatHash: u64,
	ownedModules: BTreeSet<String>,
	linkedEnvs: BTreeMap<Uuid, BTreeSet<String>>,
	modules: BTreeMap<String, ModuleType>
}
impl<ModuleType: Module> Environment<ModuleType>
{
	/// Construct an empty environment identified by the given
	/// [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier).
	pub fn withUuid (uuid: Uuid) -> Self { Self {
		uuid, compatHash: 0, ownedModules: BTreeSet::new(), linkedEnvs: BTreeMap::new(), modules: BTreeMap::new()
	}}

	/// Construct an empty environment identified by the given
	/// [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier) and setting compatibility with the provided
	/// [cgv_shader::]()
	pub fn forContextWithUuid (context: &impl compile::Context<ModuleType>, uuid: Uuid) -> Self { Self {
		uuid, compatHash: context.environmentCompatHash(), ownedModules: BTreeSet::new(), linkedEnvs: BTreeMap::new(), modules: BTreeMap::new()
	}}

	/// Construct an empty environment identified by the given
	/// [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier) and an initial compatibility hash.
	pub fn withUuidAndCompatHash (uuid: Uuid, compatHash: u64) -> Self { Self {
		uuid, compatHash, ownedModules: BTreeSet::new(), linkedEnvs: BTreeMap::new(), modules: BTreeMap::new()
	}}

	///
	pub fn cloneWithNewUuid (&self, uuid: Uuid) -> Self {
		let mut newEnv = self.clone();
		newEnv.uuid = uuid;
		newEnv
	}

	/// Reports the [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier) of this environment.
	#[inline(always)]
	pub fn uuid (&self) -> Uuid {
		self.uuid
	}

	/// Reports the compatibility hash of this environment.
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
	///
	/// # Returns
	///
	/// The merged environment, or a [`MergeError`] indicating what went wrong.
	pub fn merge (&self, other: &Environment<ModuleType>, uuid: Uuid) -> Result<Self, MergeError> {
		if self.compatHash != other.compatHash {
			return Err(MergeError::Incompatible);
		}
		let mut newEnv = self.cloneWithNewUuid(uuid);
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
		// Initial sanity check
		if self.compatHash != other.compatHash {
			return Err(MergeError::Incompatible);
		}

		// Step 1 - Compile list of linked environments in the `other` environment that we need to import (basically all
		//          envs that we're not already referencing ourselves)
		let mut skippedExistingEnvs: Vec<&Uuid> = Vec::with_capacity(self.linkedEnvs.len());
		let mut moduleOrigins: BTreeMap<&str, Uuid> = BTreeMap::new();
		let mut subenvsToLink: Vec<(Uuid, &BTreeSet<String>)> = Vec::with_capacity(other.linkedEnvs.len());
		for (uuid, moduleNames) in &other.linkedEnvs
		{
			if let Some(alreadyReferenced) = self.linkedEnvs.iter().find(
				|&(existingUuid, _)| existingUuid == uuid
			){
				skippedExistingEnvs.push(alreadyReferenced.0)
			}
			else
			{
				subenvsToLink.push((*uuid, moduleNames));
				for moduleName in moduleNames {
					if let Some(existingUuid) = moduleOrigins.insert(moduleName, *uuid) {
						panic!(
							"Internal state violation: to-be-merged shader environment [{}], claims to reference at \
							 least two other environments that define modules with conflicting names: module \
							 '{moduleName}' is reportedly defined in both [{existingUuid}] and [{uuid}]", other.uuid
						);
					}
				}
			}
		}

		// Step 2 - Flatten list of new modules to be merged in, checking for duplicates (this check is what in theory
		//          should prevent the panic above from ever being triggered)
		let mut newModules: BTreeMap<&str, &ModuleType> = BTreeMap::new();
		for &(_, subEnvModules) in &subenvsToLink
		{
			for moduleName in subEnvModules
			{
				if let Some((alreadyDefined, _)) = self.modules.iter().find(
					|&(name, _)| name == moduleName
				){
					assert_eq!(alreadyDefined, moduleName);
					return Err(MergeError::DuplicateModuleNames(moduleName.to_owned()));
				}
				else
				{
					let module = other.modules.get(moduleName).unwrap();
					if newModules.insert(moduleName, module).is_some() {
						panic!(
							"Internal state violation: to-be-merged shader environment [{}] somehow defines module \
							 '{moduleName}' twice! (this should be impossible as the modules live in a `Map` that is \
							 keyed by name)", other.uuid
						);
					}
				}
			}
		}
		#[cfg(feature="tracing_output")] {
			tracing::info!("Merging shader environments: [{}] <- [{}]", self.uuid, other.uuid);
			tracing::debug!(
				"Skipped already referenced sub-environments: [{}]",
				skippedExistingEnvs.iter().map(|k| k.to_string()).intersperse(",".to_string()).collect::<String>()
			);
		}

		// Step 3 - The merge has been validated, now commit the changes
		// - commit new modules
		for (name, module) in newModules
		{
			if self.modules.insert(name.to_owned(), module.clone()).is_some() {
				panic!(
					"Internal logic error: a shader module with the name '{name}' is already present in the target \
					 environment! This should have been detected when validating the merger. There is now no way to \
					 recover from this condition."
				);
			};
		}
		// - reference new sub-environments
		for (uuid, moduleNames) in subenvsToLink
		{
			if self.linkedEnvs.insert(uuid, moduleNames.to_owned()).is_some() {
				panic!(
					"Internal logic error: the new sub-environment [{uuid}] was already referenced by the target! This \
					 should have been detected when validating the merger. There is now no way to recover from this \
					 condition."
				);
			};
		}
		// - reference the merged-in environment itself
		self.linkedEnvs.insert(other.uuid, other.ownedModules.clone());

		// Done!
		Ok(())
	}
}*/


#[derive(Clone)]
struct ModuleEntry<ModuleType: Module> {
	module: ModuleType,
	sourceEnv: Option<Uuid>
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
	modules: BTreeMap<PathBuf, ModuleEntry<ModuleType>>,
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
			module: moduleEntry.module.clone(),
			sourceEnv: if moduleEntry.sourceEnv.is_some() { moduleEntry.sourceEnv } else { Some(self.uuid) }
		}
	}

	#[inline(always)]
	fn externalizeModules (&self) -> BTreeMap<PathBuf, ModuleEntry<ModuleType>> {
		self.modules.iter().map(
			|(name, entry)| (
				name.to_owned(), self.externalizeModule(entry)
			)
		).collect::<BTreeMap<_, _>>()
	}

	/// Construct an empty environment identified by the given
	/// [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier).
	pub fn withUuid (uuid: Uuid, label: &'static str) -> Self { Self {
		uuid, label, compatHash: 0, modules: BTreeMap::new()
	}}

	/// Construct an empty environment identified by the given
	/// [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier) and setting compatibility with the provided
	/// [cgv_shader::]()
	pub fn forContextWithUuid (context: &impl compile::Context<ModuleType>, uuid: Uuid, label: &'static str) -> Self { Self {
		uuid, label, compatHash: context.environmentCompatHash(), modules: BTreeMap::new()
	}}

	/// Construct an empty environment identified by the given
	/// [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier) and an initial compatibility hash.
	pub fn withUuidAndCompatHash (uuid: Uuid, label: &'static str, compatHash: u64) -> Self { Self {
		uuid, label, compatHash, modules: BTreeMap::new()
	}}

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
		// Initial sanity check
		if self.compatHash != other.compatHash {
			return Err(MergeError::Incompatible);
		}

		// Compile list of foreign modules to incorperate
		let mut newModules = BTreeMap::<PathBuf, ModuleEntry<ModuleType>>::new();
		for (path, moduleEntry) in &other.modules
		{
			// Decide what to do with this module
			match self.modules.get(path)
			{
				None => {
					// Case 1: this is a new module, include
					let check = newModules.insert(
						path.to_owned(), other.externalizeModule(moduleEntry)
					);
					assert!(check.is_none());
				},

				Some(ownEntry) => {
					// Case 2: we already have a module at this path, and it has the same source: do not include
					if ownEntry.sourceEnv != moduleEntry.sourceEnv {}
					// Case 2: we already have a module at this path, and it has the same source: do not include
				}
			};
		}

		// Done!
		Ok(())
	}
}
