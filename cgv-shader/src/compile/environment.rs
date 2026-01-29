
//////
//
// Imports
//

// Standard library
use std::{path::{PathBuf, Path}, error::Error, fmt::{Display, Formatter}};

// Serialization
use serde;
use postcard;

// UUID library
use util::uuid as uuid;
use uuid::Uuid;

// CGV-rs core libraries
use cgv_util::{self as util, ds::UniqueVecElement};

// Local imports
use crate::compile;



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
			Self::DuplicateModulePaths(path) => &format!("duplicate module paths: {}", path.display())
		};
		write!(formatter, "MergeError[{desc}]")
	}
}
impl Error for MergeError {}

#[derive(Debug)]
pub enum AddModuleError {
	DuplicateModulePaths(PathBuf)
}
impl Display for AddModuleError
{
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::DuplicateModulePaths(path) => &format!("duplicate module paths: {}", path.display())
		};
		write!(formatter, "AddModuleError[{desc}]")
	}
}
impl Error for AddModuleError {}



//////
//
// Traits
//

/// The trait of modules that make up a [`compile::Environment`].
pub trait Module: Sized + Clone {}



//////
//
// Structs
//



//////
//
// Structs
//

///
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub struct BytesModule(Vec<u8>);
impl BytesModule
{
	///
	#[inline(always)]
	pub fn new (bytes: Vec<u8>) -> Self {
		Self(bytes)
	}

	///
	#[inline(always)]
	pub fn fromSlice (bytes: &[u8]) -> Self {
		Self(bytes.to_owned())
	}

	///
	#[inline(always)]
	pub fn irBytes (&self) -> &[u8] {
		self.0.as_slice()
	}
}
impl Module for BytesModule {}
impl<Bytes: AsRef<[u8]>> From<Bytes> for BytesModule {
	#[inline(always)]
	fn from (bytes: Bytes) -> Self { Self::fromSlice(bytes.as_ref()) }
}

///
#[derive(Clone,serde::Serialize,serde::Deserialize)]
pub struct ModuleEntry<ModuleType: Module> {
	pub path: PathBuf,
	pub module: ModuleType,
	pub sourceEnv: Option<Uuid>
}
impl<ModuleType: Module> util::ds::UniqueVecElement for ModuleEntry<ModuleType> {
	type Key<'a> = &'a Path where ModuleType: 'a;

	fn key (&self) -> Self::Key<'_> {
		&self.path
	}
}

/// Represents a shader *compilation environment* that, in essence, collects shader modules that should be accessible in
/// conjunction with each other to shader programs that might want to use them.
///
/// Environments should be utilized the following way: *CGV-rs* crates that want to provide shaders should collect all
/// their shaders either during compile-time (preferred) or at initialization, potentially already compiling them to
/// some intermediate representation, and put each one into an [`env::Module`](Module). The modules collected this way
/// should then go into an instance of [`Environment`] that the crate will make available through some means of its
/// choosing. Since the modules the crate provides will typically also depend on modules from other crates (the most
/// notable exception being the core `cgv` crate, which only provides self-contained shader functionality), **this
/// environment must be [merged](Environment::mergeWith) with the environments that the other crates provide their
/// modules in** to make it usable in a self-contained manner\* (akin to static linking).
///
/// If the collected modules depend on some sort of compiler or preprocessor settings, some hash that uniquely
/// identifies the required configuration should be set which will be checked when [merging](Environment::merge) and
/// which crates using the environment can check to make sure that they are compatible. The crate exporting the
/// environment should naturally provide a way for clients to find and use compatible settings.
///
/// ##### Providing shader environments
///
/// A convenient way for crates to serve their shader environments is to provide a function that returns a `'static`
/// reference to a singleton [`Environment`] instance. The `cgv` core crate does this via its
/// `cgv::obtainShaderCompileEnvironment`, where the environment – as compiled, serialized and baked into its object
/// code during build time – is lazily deserialized from memory via an internal [`LazyLock`] static. This only works if
/// the underlying [module type](Module) is [`Send`] and [`Sync`].
///
/// This is the case for the `slang::EnvModule` provided by the `cgv_shader` crate with the `slang_runtime` feature
/// enabled. *CGV-rs* promotes and actively facilitates the use of the *Slang* shading language, but if custom
/// [module types](Module) are to be used (e.g., when using other shading languages), clients are advised to ensure
/// their implementations of [`env::Module`](Module) are [`Send`] and [`Sync`] if they want to use this method.
///
/// ##### Footnotes
///
/// \*This is merely *CGV-rs* **convention**, in principle crates could require clients to merge in a list of stated
/// dependencies on their side.
#[derive(Clone,serde::Serialize,serde::Deserialize)]
pub struct Environment<ModuleType: Module> {
	uuid: Uuid,
	label: String,
	compatHash: u64,
	modules: util::ds::BTreeUniqueVec<ModuleEntry<ModuleType>>
}
impl<ModuleType> Environment<ModuleType>
	where ModuleType: Module + serde::Serialize+(for<'de> serde::Deserialize<'de>)
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
	fn externalizeModules (&self) -> util::ds::BTreeUniqueVec<ModuleEntry<ModuleType>>
	{
		let mut newModules = util::ds::BTreeUniqueVec::withCapacity(
			self.modules.len()
		);
		for entry in self.modules.iter() {
			assert!(newModules.push(self.externalizeModule(&entry)));
		}
		newModules
	}


	////
	// Constructors

	///
	pub fn deserialize (bytes: &[u8]) -> anyhow::Result<Self> {
		Ok(postcard::from_bytes(bytes).map_err(anyhow::Error::new)?)
	}

	///
	#[inline(always)]
	pub fn deserializeFromFile (filepath: impl AsRef<Path>) -> anyhow::Result<Self> {
		Self::deserialize(std::fs::read(filepath)?.as_slice())
	}

	/// Construct an empty environment identified by the given
	/// [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier).
	pub fn withUuid (uuid: Uuid, label: &str) -> Self { Self {
		uuid, label: label.to_owned(), compatHash: 0, modules: util::ds::BTreeUniqueVec::new()
	}}

	/// Construct an empty environment identified by the given
	/// [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier) and setting compatibility with the provided
	/// [`compile::EnvironmentEnabled`]
	pub fn forContextWithUuid (context: &impl compile::EnvironmentEnabled, uuid: Uuid, label: &str)
	-> Self { Self {
		uuid, label: label.to_owned(), compatHash: context.environmentCompatHash(),
		modules: util::ds::BTreeUniqueVec::new()
	}}

	/// Construct an empty environment identified by the given
	/// [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier) and an initial compatibility hash.
	pub fn withUuidAndCompatHash (uuid: Uuid, label: &str, compatHash: u64) -> Self { Self {
		uuid, label: label.to_owned(), compatHash, modules: util::ds::BTreeUniqueVec::new()
	}}


	////
	// Methods

	///
	pub fn serialize (&self) -> Vec<u8> {
		postcard::to_allocvec(self).unwrap()
	}

	///
	#[inline(always)]
	pub fn serializeToFile (&self, filepath: impl AsRef<Path>) -> std::io::Result<()> {
		std::fs::write(filepath, self.serialize())
	}

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
	pub fn cloneWithNewUuid (&self, newUuid: Uuid, newLabel: &str) -> Self
	{
		// Commit all modules, marking the ones owned by `self` as being sourced from `self.uuid`
		let newModules = self.externalizeModules();

		// Build cloned environment
		Self {
			uuid: newUuid, label: newLabel.to_owned(), compatHash: self.compatHash, modules: newModules
		}
	}

	/// Report the [UUID](https://de.wikipedia.org/wiki/Universally_Unique_Identifier) of this environment.
	#[inline(always)]
	pub fn uuid (&self) -> Uuid {
		self.uuid
	}

	/// Reference the label string of this environment.
	#[inline(always)]
	pub fn label (&self) -> &str {
		&self.label
	}

	/// Report the compatibility hash of this environment.
	#[inline(always)]
	pub fn compatHash (&self) -> u64 {
		self.compatHash
	}

	/// Obtain an iterator over all [modules](ModuleEntry) in the environment.
	#[inline]
	pub fn modules (&self) -> impl Iterator<Item=&ModuleEntry<ModuleType>> {
		self.modules.iter()
	}

	/// Adds a new module to the collection of modules in the current context.
	///
	/// **Note**: By convention, modules added this way will **always** be considered to be introduced by *this*
	/// environment. Externally sourced modules can only be added via [merging](Environment::mergeWith) with other
	/// environments.
	///
	/// # Arguments
	///
	/// * `path` – The path associated with the module being added. This determines how the module can be found during
	/// compilation or linking of shaders by a [`compile::EnvironmentEnabled`] later on.
	/// * `module` – A module of this environment's [`ModuleType`] to add to the environment.
	///
	/// # Returns
	///
	/// `Ok` if the module is successfully added, or an `AddModuleError` if there was a problem (most typically, a
	/// module already exists at the desired `path`).
	///
	/// # Example
	///
	/// ```
	/// use cgv_util::{self as util, uuid::Uuid};
	/// use cgv_shader::compile;
	///
	/// let mut env = compile::Environment::<compile::env::BytesModule>::withUuid(
	/// 	Uuid::from_u64_pair(util::unique::uint64(), util::unique::uint64()), "testEnvironment"
	/// );
	/// let module = compile::env::BytesModule::new(vec![0, 1, 2]);
	/// let module1 = vec![0, 1, 2].into(); // dummy
	/// let module2 = vec![3, 4, 5].into(); // bytecodes
	///
	/// assert!(env.addModule("/namespace/mymodule", module1).is_ok());
	/// assert!(env.addModule("/namespace/mymodule", module2).is_err()); // <- duplicate module path
	/// ```
	pub fn addModule (&mut self, path: impl AsRef<Path>, module: ModuleType) -> Result<(), AddModuleError>
	{
		if self.modules.push(ModuleEntry { path: path.as_ref().to_owned(), module, sourceEnv: None }) {
			Ok(())
		}
		else {
			Err(AddModuleError::DuplicateModulePaths(path.as_ref().to_owned()))
		}
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
	pub fn merge (&self, other: &Environment<ModuleType>, uuid: Uuid, label: &str) -> Result<Self, MergeError> {
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
		let mut newModules = util::ds::BTreeUniqueVec::withCapacity(other.modules.len());
		for otherEntry in other.modules.iter()
		{
			// Decide what to do with this module
			match self.modules.fetch(&otherEntry.key())
			{
				// Case 1: this is an unseen module, include
				None => assert!(newModules.push(other.externalizeModule(otherEntry))),

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
		self.modules.extend(newModules);

		// Done!
		Ok(())
	}
}
unsafe impl<ModuleType: Module+Send> Send for Environment<ModuleType> {}
unsafe impl<ModuleType: Module+Sync> Sync for Environment<ModuleType> {}
