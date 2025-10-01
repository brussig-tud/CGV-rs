
//////
//
// Imports
//

// Standard library
use std::{vec::Vec, collections::BTreeSet, collections::BTreeMap, error::Error, fmt::{Display, Formatter}};

// Tracing library
use tracing;

// UUID library
use uuid;
use uuid::Uuid;



//////
//
// Errors
//

#[derive(Debug)]
pub enum MergeError {
	DuplicateModuleNames(String)
}
impl Display for MergeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.to_string())
	}
}
impl Error for MergeError {}



//////
//
// Traits
//

pub trait Module where Self: Sized+Clone {}

#[derive(Clone)]
pub struct _DummyModule;
impl Module for _DummyModule {}



//////
//
// Structs
//

/// Represents a shader *environment* that, in essence, collects shader modules that should be accessible in conjunction
/// with each other to shader programs that might want to use them.
///
/// Environments should be utilized the following way: *CGV-rs* crates that want to provide shaders should collect all
/// their shaders either during compile-time (preferred) or at initialization, potentially already compiling them to
/// some intermediate representation, and put each one into a [`Module`]. The modules collected this way should then go
/// into an instance of [`Environment`] that the crate will make available through some means of its choosing. Since
/// the modules the crate provides will typically also depend on modules from other crates (the most notable exception
/// being the core `cgv` create, which only provides self-contained shader functionality), **this environment must be
/// [merged](Environment::mergeWith) with the environments that the other crates provide their modules in** in order to
/// make it usable in a self-contained manner\* (akin to static linking).
///
/// \*Note that this is *CGV-rs* **convention**, in principle crates could require clients to merge in a list of stated
/// dependencies on their side.
pub struct Environment<ModuleType: Module> {
	uuid: Uuid,
	ownedModules: BTreeSet<String>,
	linkedEnvs: BTreeMap<Uuid, BTreeSet<String>>,
	modules: BTreeMap<String, ModuleType>
}
impl<ModuleType: Module> Environment<ModuleType>
{
	/// Construct an empty environment identified by the given
	/// [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier).
	pub fn withUuid (uuid: Uuid) -> Self { Self {
		uuid, ownedModules: BTreeSet::new(), linkedEnvs: BTreeMap::new(), modules: BTreeMap::new()
	}}

	/// Reports the [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier) of this environment.
	pub fn uuid (&self) -> Uuid {
		self.uuid
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
	/// * `uuid` – A UUID to assign the new environment.
	///
	/// # Returns
	///
	/// The merged environment, or a [`MergeError`] indicating what went wrong.
	pub fn merge (self, other: &Environment<ModuleType>, uuid: Uuid) -> Result<Self, MergeError> {
		let mut newEnv = Self::withUuid(uuid);
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
		tracing::info!("Merging shader environments: [{}] <- [{}]", self.uuid, other.uuid);
		tracing::debug!(
			"Skipped already referenced sub-environments: [{}]",
			skippedExistingEnvs.iter().map(|k| k.to_string()).intersperse(",".to_string()).collect::<String>()
		);


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
}
