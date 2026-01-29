
//////
//
// Imports
//

// Standard library
#[allow(unused_imports)] // BTreeSet is only required in case of some enabled features
use std::{path::Path, collections::{BTreeMap, BTreeSet}, fmt::Display};

// Anyhow library
use anyhow;

// Bitcode library
use bitcode;

// WGPU library
#[cfg(feature="wgpu_runtime")]
use wgpu;

// Tracing library
#[cfg(feature="wgpu_runtime")]
use tracing;

// Local imports
use crate::*;



//////
//
// Errors
//

/// Error conditions when creating [`wgpu::ShaderModule`]s.
#[cfg(feature="wgpu_runtime")]
#[derive(Debug,PartialEq)]
pub enum CreateShaderModuleError
{
	#[doc=include_str!("_doc/_InvalidEntryPoint_withString.md")]
	InvalidEntryPoint(String),

	/// The package from which a module was requested does not contain an instance in the requested source type. Holds
	/// the requested `SourceType` that was unavailable.
	InvalidSourceType(WgpuSourceType)
}
#[cfg(feature="wgpu_runtime")]
impl Display for CreateShaderModuleError
{
	fn fmt (&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::InvalidEntryPoint(ep) => format!("invalid entry point: `{ep}`"),
			Self::InvalidSourceType(st) => format!("invalid source type: `{st}`")
		};
		write!(formatter, "CreateShaderModuleError[{desc}]")
	}
}
#[cfg(feature="wgpu_runtime")]
impl std::error::Error for CreateShaderModuleError {}


/// Error conditions when building [shader program instances](ProgramInstance).
#[derive(Debug)]
pub enum ProgramInstanceCreationError
{
	/// A [`compile::Context`] was supposed to compile a program instance for the given [`WgpuSourceType`] but did not
	/// support the corresponding [compilation target](compile::Target).
	IncompatibleContext(WgpuSourceType),

	#[doc=include_str!("_doc/_InvalidEntryPoint_withString.md")]
	InvalidEntryPoint(String),

	/// A backend error that occurred during some part of the build process.
	Backend(anyhow::Error)
}
impl Display for ProgramInstanceCreationError
{
	fn fmt (&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		let desc = match self {
			Self::IncompatibleContext(srcType) => format!("context cannot compile to {srcType}"),
			Self::InvalidEntryPoint(ep) => format!("invalid entry point: `{ep}`"),
			Self::Backend(err) => format!("backend error: {err}")
		};
		write!(formatter, "ProgramInstanceCreationError[{desc}]")
	}
}
impl std::error::Error for ProgramInstanceCreationError {}


///
#[derive(Debug)]
pub enum PackageFromProgramError {
	/// The program was built for a compilation target does not match any [`WgpuSourceType`].
	IncompatibleProgram(compile::Target),

	/// Some problem instantiating the shader program for a source type occurred.
	InstanceCreation(ProgramInstanceCreationError)
}
impl Display for PackageFromProgramError
{
	fn fmt (&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		match self {
			Self::IncompatibleProgram(target) => write!(
				formatter, "PackageFromProgramError[program uses incompatible code type {target}]"
			),

			Self::InstanceCreation(err) => write!(
				formatter, "PackageFromProgramError[`{err}`]"
			)
		}
	}
}
impl std::error::Error for PackageFromProgramError {}



//////
//
// Classes
//

/// Represents a single *WGPU* compatible shader program for one or multiple shader stages, potentially with different
/// specializations for each of its various entry points.
#[derive(bitcode::Encode,bitcode::Decode)]
pub struct ProgramInstance {
	entryPoints: BTreeMap<Option<String>, Vec<u8>>,
}
impl ProgramInstance
{
	///
	#[inline(always)]
	pub fn fromSingleEntryPoint (name: Option<String>, code: Vec<u8>) -> Self {
		Self { entryPoints: BTreeMap::from([(name, code)])}
	}

	/// Shorthand for `ProgramInstance::`[`fromSingleEntryPoint(None, code)`](Self::fromSingleEntryPoint)`.
	#[inline(always)]
	pub fn generic (code: Vec<u8>) -> Self {
		Self::fromSingleEntryPoint(None, code)
	}

	///
	#[inline]
	pub fn addEntryPoint (&mut self, name: Option<&str>, code: Vec<u8>) {
		self.entryPoints.insert(name.map(|name| name.to_owned()), code);
	}

	///
	pub fn code (&self, entryPointName: Option<&str>) -> Option<&[u8]> {
		self.entryPoints.get(&entryPointName.map(|name| name.to_owned())).map(
			|ep| ep.as_slice()
		)
	}
}

/// Represents a package of one or more ready-to-use instances of a shader program compiled to different representations
/// (e.g. *SPIR-V* or *WGSL*).
#[derive(bitcode::Encode,bitcode::Decode)]
pub struct Package {
	name: String,
	instances: BTreeMap<WgpuSourceType, ProgramInstance>
}
impl Package
{
	////
	// Internal helpers

	fn programIntoInstance (prog: Program, entryPoints: Option<&BTreeSet<Option<&str>>>)
		-> Result<ProgramInstance, ProgramInstanceCreationError>
	{
		// Create the program instance with specialized code for the indicated entry points if any, or just the generic
		// code if no entry points were specified.
		if let Some(entryPoints) = entryPoints
		{
			let mut progInstance = ProgramInstance { entryPoints: BTreeMap::new() };
			for &entryPoint in entryPoints
			{
				if let Some(entryPointName) = entryPoint
				{
					if let Some(code) = prog.entryPointProg(entryPointName) {
						progInstance.addEntryPoint(Some(entryPointName), code.toVec());
					}
					else {
						return Err(ProgramInstanceCreationError::InvalidEntryPoint(entryPointName.to_owned()))
					}
				}
				else {
					progInstance.addEntryPoint(None, prog.allEntryPointsProg().toVec());
				}
			}
			Ok(progInstance)
		}
		else {
			// Only include the generic program that includes all code paths from all entry points
			Ok(ProgramInstance::generic(prog.allEntryPointsProg().toVec()))
		}
	}


	////
	// Public API

	/// Deserialize from the given bytes.
	pub fn deserialize (bytes: &[u8]) -> anyhow::Result<Self> {
		Ok(bitcode::decode(bytes)?)
	}

	/// Deserialize from the file.
	pub fn fromFile (filename: impl AsRef<Path>) -> anyhow::Result<Self> {
		Ok(bitcode::decode(std::fs::read(filename)?.as_slice())?)
	}

	/// Construct with the given [`ProgramInstance`] as the sole instance of type `sourceType` in the package.
	pub fn fromSingleInstance (sourceType: WgpuSourceType, instance: ProgramInstance, name: Option<String>) -> Self {
		Self {
			name: name.unwrap_or_else(|| uniqueAnonymousName()), instances: BTreeMap::from([(sourceType, instance)])
		}
	}

	///
	pub fn fromModuleMultipleTypes<'outer, Context> (
		sourceTypes: &[WgpuSourceType], context: &'outer Context, module: &Context::ModuleType<'outer>,
		entryPoints: Option<BTreeSet<Option<&str>>>
	) -> Result<Self, ProgramInstanceCreationError>
	where
		Context: compile::Context
	{
		// Build module
		use compile::Module;
		let linkedProg = compile::buildModule(context, module).map_err(
			|err| ProgramInstanceCreationError::Backend(err.into())
		)?;

		// Instantiate for every source type
		let mut instances = BTreeMap::new();
		for &sourceType in sourceTypes
		{
			// Check compilation target
			let target = sourceType.into();
			if !context.supportsTarget(target) {
				return Err(ProgramInstanceCreationError::IncompatibleContext(sourceType));
			}

			// Translate code
			let program = Program::fromLinkedComposite(context, target, &linkedProg).map_err(
				|err| ProgramInstanceCreationError::Backend(err)
			)?;

			// Instantiate program instance
			instances.insert(sourceType, Self::programIntoInstance(program, entryPoints.as_ref())?);
		}

		Ok(Self {
			name: unsafe {
				// SAFETY:
				// If we're here, then the module could be built from the filename (or was assigned a virtual one), so
				// we know the path definitely contains a filename component. Calling `file_name().unwrap_unchecked()`
				// can thus never produce `None`.
				module.virtualFilepath().file_name().unwrap_unchecked().to_string_lossy().into()
			},
			instances
		})
	}

	/// Create the package from the given *Slang* shader source file, compiling it under several contexts to produce
	/// different instances for the [source types](SourceType) each [`slang::Context`] is set up for.
	pub fn fromSourceFileMultipleTypes<CompileContext> (
		sourceTypes: &[WgpuSourceType], context: &CompileContext, filename: impl AsRef<Path>, entryPoints: Option<BTreeSet<Option<&str>>>
	) -> Result<Self, ProgramInstanceCreationError>
	where
		CompileContext: compile::HasFileSystemAccess
	{
		// Compile from source file
		let module = context.compile(&filename).map_err(
			|err| ProgramInstanceCreationError::Backend(err.into())
		)?;

		// Package an instance for every source type
		Self::fromModuleMultipleTypes(sourceTypes, context, &module, entryPoints)
	}

	/// Create the package from the given *Slang* shader source code, compiling it under several contexts to produce
	/// different instances for the [source types](SourceType) each [`slang::Context`] is set up for.
	pub fn fromSourceMultipleTypes<CompileContext> (
		sourceTypes: &[WgpuSourceType], context: &CompileContext, programName: impl AsRef<Path>,
		sourceCode: impl AsRef<str>, entryPoints: Option<BTreeSet<Option<&str>>>
	) -> Result<Self, ProgramInstanceCreationError>
	where
		CompileContext: compile::Context
	{
		// Compile from source code string
		let module = context.compileFromNamedSource(
			programName, sourceCode.as_ref()
		).map_err(
			|err| ProgramInstanceCreationError::Backend(err.into())
		)?;

		// Package an instance for every source type
		Self::fromModuleMultipleTypes(sourceTypes, context, &module, entryPoints)
	}

	/// Create the package with the given [`Program`] as the sole instance.
	pub fn fromProgram (program: Program, name: Option<String>, entryPoints: Option<BTreeSet<Option<&str>>>)
		-> Result<Self, PackageFromProgramError>
	{
		let srcType = program.target().intoWgpuSourceType().ok_or_else(
			|| PackageFromProgramError::IncompatibleProgram(program.target())
		)?;
		let instance = Self::programIntoInstance(program, entryPoints.as_ref()).map_err(
			|err| PackageFromProgramError::InstanceCreation(err)
		)?;
		Ok(Self::fromSingleInstance(srcType, instance, name))
	}

	/// Create the package from the given shader source file.
	#[inline(always)]
	pub fn fromSourceFile<CompileContext> (
		sourceType: WgpuSourceType, context: &CompileContext, filename: impl AsRef<Path>,
		entryPoints: Option<BTreeSet<Option<&str>>>
	) -> Result<Self, ProgramInstanceCreationError>
	where
		CompileContext: compile::HasFileSystemAccess
	{
		Self::fromSourceFileMultipleTypes(&[sourceType], context, filename, entryPoints)
	}

	/// Create the package from the given *Slang* shader source code string.
	#[inline(always)]
	pub fn fromSource<CompileContext> (
		sourceType: WgpuSourceType, context: &CompileContext, programName: impl AsRef<Path>,
		sourceCode: impl AsRef<str>, entryPoints: Option<BTreeSet<Option<&str>>>
	) -> Result<Self, ProgramInstanceCreationError>
	where CompileContext: compile::Context {
		Self::fromSourceMultipleTypes(&[sourceType], context, programName, sourceCode, entryPoints)
	}

	/// Set the instance of the program for the given source type to the package. If there is already an instance for
	/// the given source type, it will be replaced.
	pub fn setInstance (&mut self, sourceType: WgpuSourceType, instance: ProgramInstance) {
		self.instances.insert(sourceType, instance);
	}

	/// Create a *WGPU* shader module ready for binding to a pipeline from the contained program instance of the given
	/// source type.
	///
	/// # Arguments
	///
	/// * `device` – The *WGPU* device on which to create the shader module.
	/// * `sourceType` – The desired type of shader source.
	/// * `entryPointName` – Optionally get the program specialization for the entry point of this name. If no entry
	///                      point is specified, the generic code containing all code paths for all entry points will be
	///                      used if available or a [`CreateShaderModuleError::InvalidEntryPoint`] will be emitted.
	/// * `label` – The string to internally label the GPU-side shader module object with.
	///
	/// # Returns
	///
	/// A ready-to-use *WGPU* shader module if the requested source type and a specialization for the requested entry
	/// point exist, or an error describing the encountered [failure condition](CreateShaderModuleError).
	#[cfg(feature="wgpu_runtime")]
	pub fn createShaderModule (
		&self, device: &wgpu::Device, sourceType: WgpuSourceType, entryPointName: Option<&str>, label: Option<&str>
	) -> Result<wgpu::ShaderModule, CreateShaderModuleError>
	{
		// Find requested entry point in the requested instance

		let progInstance = self.instances.get(&sourceType).ok_or(
			CreateShaderModuleError::InvalidSourceType(sourceType)
		)?;
		let code = progInstance.code(entryPointName).ok_or_else(||
			CreateShaderModuleError::InvalidEntryPoint(entryPointName.unwrap_or("<default>").to_owned())
		)?;

		// Create the shader module
		let shaderModule = match sourceType
		{
			WgpuSourceType::SPIRV => {
				let shaderModule;
				#[cfg(target_arch="wasm32")] {
					// WASM WebGPU requires internal transpiling to WGSL via Naga.
					shaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
						label, source: wgpu::util::make_spirv(code)
					})
				};
				#[cfg(not(target_arch="wasm32"))] {
					// Support DX12 and Metal devices by having WGPU transpile the SPIR-V to Naga-IR
					// - native SPIR-V passthrough on Vulkan
					if unsafe {
						// SAFETY: we won't do anything with the device if it is not the expected Vulkan backend
						device.as_hal::<wgpu::hal::api::Vulkan>().is_some()
					}{
						shaderModule = unsafe {
							// SAFETY: we already verified that the code is SPIR-V
							device.create_shader_module_passthrough(wgpu::ShaderModuleDescriptorPassthrough {
								entry_point: "NOT_USED".into(), label, spirv: Some(wgpu::util::make_spirv_raw(code)),
								..Default::default()
							})
						}
					}
					// - transpile from SPIR-V otherwise
					else {
						shaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
							label, source: wgpu::util::make_spirv(code)
						})
					}
				}
				shaderModule
			},

			WgpuSourceType::WGSL => {
				device.create_shader_module(wgpu::ShaderModuleDescriptor {
					label, source: wgpu::ShaderSource::Wgsl(str::from_utf8(code).unwrap().into()),
				})
			},

			WgpuSourceType::GLSL => unimplemented!(
				"GLSL source type not yet supported pending more complete implementation in upstream WGPU"
			)
		};

		// Done!
		Ok(shaderModule)
	}

	/// Create a *WGPU* shader module ready for binding to a pipeline from the most suitable program instance contained
	/// in the package.
	///
	/// Currently, this will prefer the following source types:
	/// * All **native backends**: *SPIR-V*, *WGSL*
	/// * **WebGPU**/WASM: *WGSL*, *SPIR-V*
	///
	/// # Arguments
	///
	/// * `device` – The *WGPU* device on which to create the shader module.
	/// * `entryPointName` – Optionally get the program specialization for the entry point of this name. If no entry
	///                      point is specified, the generic code containing all code paths for all entry points will be
	///                      used if available or an [`InvalidEntryPointError`] will be emitted.
	/// * `label` – The string to internally label the GPU-side shader module object with.
	///
	/// # Returns
	///
	/// `Some` ready-to-use *WGPU* shader module if the package contains a suitable instance and a specialization for
	/// the requested entry point exists, 'None' otherwise.
	#[cfg(feature="wgpu_runtime")]
	pub fn createShaderModuleFromBestInstance (
		&self, device: &wgpu::Device, entryPointName: Option<&str>, label: Option<&str>
	) -> Option<wgpu::ShaderModule>
	{
		// Define feasible source types from most to least suitable
		let sourceTypes;
		// - WebGPU/WASM
		#[cfg(target_arch="wasm32")] {
			const SOURCE_TYPES: [WgpuSourceType; 2] = [WgpuSourceType::WGSL, WgpuSourceType::SPIRV];
			sourceTypes = SOURCE_TYPES;
		}
		// - all native backends (currently always considers SPIR-V preferable even on non-Vulkan backends)
		#[cfg(not(target_arch="wasm32"))] {
			const SOURCE_TYPES: [WgpuSourceType; 2] = [WgpuSourceType::SPIRV, WgpuSourceType::WGSL];
			sourceTypes = SOURCE_TYPES;
		}

		// Try to get an instance from the feasible source types
		for sourceType in sourceTypes
		{
			match self.createShaderModule(device, sourceType, entryPointName, label)
			{
				Ok(shaderModule) => return Some(shaderModule),
				Err(err) => {
					// Move on if no instance of this source type is available
					match err {
						CreateShaderModuleError::InvalidSourceType(_) => continue, // try next source type
						CreateShaderModuleError::InvalidEntryPoint(ep) => tracing::warn!(
							"Shader entry point `{ep}` not found in {sourceType} program instance"
						)
					}
				}
			}
		}

		// No instance in the package could be turned into a functioning shader module!
		None
	}

	/// Serialize the package into a series of bytes (e.g. for storing in a file).
	pub fn serialize (&self) -> Vec<u8> {
		bitcode::encode(self)
	}

	/// Save the package to a file.
	pub fn writeToFile(&self, filename: impl AsRef<Path>) ->std::io::Result<()> {
		Ok(std::fs::write(filename, self.serialize())?)
	}
}
