
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
// Structs
//

///
#[derive(bitcode::Encode,bitcode::Decode)]
struct EntryPoint {
	pub code: Vec<u8>,
}



//////
//
// Errors
//

/// An error resulting from an invalid entry point name, typically because no entry point with that name existed in a
/// [`Program`].
#[derive(Debug)]
pub struct InvalidEntryPointError {
	epName: String
}
impl Display for InvalidEntryPointError {
	fn fmt (&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(formatter, "InvalidEntryPointError[`{}`]", self.epName)
	}
}
impl std::error::Error for InvalidEntryPointError {}

/// An error resulting from attempting to use a [`Program`] instance for a [source type](SourceType) that is not
/// available, e.g. because a [`Package`] does not include it.
#[derive(Debug)]
pub struct InvalidSourceTypeError {
	sourceType: SourceType
}
impl Display for InvalidSourceTypeError {
	fn fmt (&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(formatter, "InvalidSourceTypeError[`{}`]", self.sourceType)
	}
}
impl std::error::Error for InvalidSourceTypeError {}

/// An error resulting from attempting to use an automatically detected suitable [`Program`] instance for some platform
/// or scenario but the [`Package`] did not include any suitable instances.
#[derive(Debug)]
pub struct NoSuitableProgramInstanceError {}
impl Display for NoSuitableProgramInstanceError {
	fn fmt (&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(formatter, "NoSuitableProgramInstanceError")
	}
}
impl std::error::Error for NoSuitableProgramInstanceError {}



//////
//
// Classes
//

/// Represents a single *WGPU* compatible shader program for one or multiple shader stages, potentially with different
/// specializations for each of its various entry points.
#[derive(bitcode::Encode,bitcode::Decode)]
pub struct Program {
	entryPoints: BTreeMap<Option<String>, EntryPoint>,
}
impl Program
{
	pub fn fromSingleEntryPoint (name: Option<String>, code: Vec<u8>) -> Self {
		Self { entryPoints: BTreeMap::from([(name, EntryPoint { code })])}
	}

	#[inline]
	pub fn generic (code: Vec<u8>) -> Self {
		Self::fromSingleEntryPoint(None, code)
	}

	pub fn addEntryPoint (&mut self, name: Option<&str>, code: Vec<u8>) {
		self.entryPoints.insert(name.map(|name| name.to_owned()), EntryPoint { code });
	}

	pub fn code (&self, entryPointName: Option<&str>) -> std::result::Result<&[u8], InvalidEntryPointError> {
		self.entryPoints.get(&entryPointName.map(|name| name.to_owned())).map(
			|ep| ep.code.as_slice()
		).ok_or(InvalidEntryPointError {
			epName: entryPointName.unwrap_or("<all/generic>").to_owned()
		})
	}
}

/// Represents a package of one or more ready-to-use instances of a shader program compiled to different representations
/// (e.g. *SPIR-V* or *WGSL*).
#[derive(bitcode::Encode,bitcode::Decode)]
pub struct Package {
	instances: BTreeMap<SourceType, Program>,
}
impl Package
{
	/// Deserialize from the given bytes.
	pub fn deserialize (bytes: &[u8]) -> anyhow::Result<Self> {
		Ok(bitcode::decode(bytes)?)
	}

	/// Deserialize from the file.
	pub fn fromFile (filename: impl AsRef<Path>) -> anyhow::Result<Self> {
		Ok(bitcode::decode(std::fs::read(filename)?.as_slice())?)
	}

	/// Create a package with a single instance.
	pub fn withSingleInstance (sourceType: SourceType, program: Program) -> anyhow::Result<Self> {
		Ok(Self { instances: BTreeMap::from([(sourceType, program)]) })
	}

	/// Internal helper function to create a single program instance from a *Slang* shader source file.
	#[cfg(feature="slang_runtime")]
	fn buildSingleInstanceFromSlang (
		slangContext: &slang::Context, filename: impl AsRef<Path>, entryPoints: Option<&BTreeSet<Option<&str>>>
	) -> anyhow::Result<(SourceType, Program)>
	{
		// Compile Slang code
		let slangProg = slangContext.buildProgram(filename)?;

		// Create the program instance for the compilation target indicated by the Slang context, with code for the
		// indicated entry points if any, or the generic code if no entry points were specified.
		if let Some(entryPoints) = entryPoints
		{
			let mut progInstance = Program { entryPoints: BTreeMap::new() };
			for &entryPoint in entryPoints
			{
				if let Some(entryPointName) = entryPoint
				{
					if let Some(ep) = slangProg.entryPoints().iter().find(
						|&ep| ep.slang.function_reflection().name() == entryPointName
					){
						progInstance.addEntryPoint(Some(entryPointName), ep.buildArtifact().to_owned());
					}
					else {
						return Err(InvalidEntryPointError { epName: entryPointName.to_owned() }.into())
					}
				}
				else {
					progInstance.addEntryPoint(None, slangProg.genericBuildArtifact().to_owned());
				}
			}
			Ok((slangContext.compilationTarget, progInstance))
		}
		else {
			// Only include the generic program that includes code paths from all entry points
			Ok((
				slangContext.compilationTarget, Program::generic(slangProg.genericBuildArtifact().to_owned())
			))
		}
	}

	/// Create the package from the given *Slang* shader source file, compiling it under several contexts to produce
	/// several instances for different [source types](SourceType).
	#[cfg(feature="slang_runtime")]
	pub fn fromSlangMultipleContexts (
		slangContexts: &[&slang::Context], filename: impl AsRef<Path>, entryPoints: Option<BTreeSet<Option<&str>>>
	) -> anyhow::Result<Self>
	{
		let mut package = Self { instances: BTreeMap::new() };
		for &slangContext in slangContexts {
			let (sourceType, program) = Self::buildSingleInstanceFromSlang(
				slangContext, filename.as_ref(), entryPoints.as_ref()
			)?;
			package.addInstance(sourceType, program);
		}
		Ok(package)
	}

	/// Create the package from the given *Slang* shader source file.
	#[cfg(feature="slang_runtime")]
	#[inline]
	pub fn fromSlang (
		slangContext: &slang::Context, filename: impl AsRef<Path>, entryPoints: Option<BTreeSet<Option<&str>>>
	) -> anyhow::Result<Self> {
		Self::fromSlangMultipleContexts(&[slangContext], filename, entryPoints)
	}

	/// Add an instance of the program for the given source type to the package. If there is already an instance for
	/// the given source type, it will be replaced.
	pub fn addInstance (&mut self, sourceType: SourceType, program: Program) {
		self.instances.insert(sourceType, program);
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
	///                      used if available or an [`InvalidEntryPointError`] will be emitted.
	///
	/// # Returns
	///
	/// A ready-to-use *WGPU* shader module if the requested source type and a specialization for the requested entry
	/// point exist. May otherwise fail with any of the following errors:
	/// * [`InvalidSourceTypeError`] – The package does not contain an instance in the requested source type.
	/// * [`InvalidEntryPointError`] – The requested entry point does not exist.
	#[cfg(feature="wgpu_runtime")]
	pub fn createShaderModule (
		&self, device: &wgpu::Device, sourceType: SourceType, entryPointName: Option<&str>, label: Option<&str>
	) -> anyhow::Result<wgpu::ShaderModule>
	{
		// Find requested entry point in the requested instance
		let progInstance = self.instances.get(&sourceType).ok_or(InvalidSourceTypeError { sourceType })?;
		let code = progInstance.code(entryPointName)?;

		// Create the shader module
		let shaderModule = match sourceType
		{
			SourceType::SPIRV => {
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
						device.as_hal::<wgpu::hal::api::Vulkan, _, bool>(|dev| dev.is_some())
					}{
						shaderModule = unsafe {
							// SAFETY: we already verified that the code is SPIR-V
							device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
								label, source: wgpu::util::make_spirv_raw(code)
							})
						};
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

			SourceType::WGSL => {
				device.create_shader_module(wgpu::ShaderModuleDescriptor {
					label, source: wgpu::ShaderSource::Wgsl(str::from_utf8(code)?.into()),
				})
			}
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
	///
	/// # Returns
	///
	/// A ready-to-use *WGPU* shader module if the package contains a suitable instance and a specialization for the
	/// requested entry point exists. May otherwise fail with any of the following errors:
	/// * [`InvalidEntryPointError`] – The requested entry point does not exist.
	/// * [`NoSuitableProgramInstanceError`] – The package does not contain an instance for a suitable source type.
	#[cfg(feature="wgpu_runtime")]
	pub fn createShaderModuleFromBestInstance (
		&self, device: &wgpu::Device, entryPointName: Option<&str>, label: Option<&str>
	) -> anyhow::Result<wgpu::ShaderModule>
	{
		// Define feasible source types from most to least suitable
		let sourceTypes;
		// - WebGPU/WASM
		#[cfg(target_arch="wasm32")] {
			const SOURCE_TYPES: [SourceType; 2] = [SourceType::WGSL, SourceType::SPIRV];
			sourceTypes = SOURCE_TYPES;
		}
		// - all native backends (currently always considers SPIR-V preferable even on non-Vulkan backends)
		#[cfg(not(target_arch="wasm32"))] {
			const SOURCE_TYPES: [SourceType; 2] = [SourceType::SPIRV, SourceType::WGSL];
			sourceTypes = SOURCE_TYPES;
		}

		// Try to get an instance from the feasible source types
		for sourceType in sourceTypes
		{
			match self.createShaderModule(device, sourceType, entryPointName, label)
			{
				Ok(shaderModule) => return Ok(shaderModule),
				Err(err) => {
					// Move on if no instance of this source type is available
					if err.downcast_ref::<InvalidSourceTypeError>().is_some() {
						continue;
					}
					// Warn if the instance was missing the entry point
					if let Some(err) = err.downcast_ref::<InvalidEntryPointError>() {
						tracing::warn!("Shader entry point `{}` not found in {} program instance", err.epName, sourceType);
					}
					// Warn of any other error
					else {
						tracing::warn!("Could not process {} program instance!\nReason: {}", sourceType, err);
					}
				}
			}
		}

		// No instance in the package could be turned into a functioning shader module!
		Err(NoSuitableProgramInstanceError {}.into())
	}

	/// Serialize the package into a series of bytes (e.g. for storing in a file).
	pub fn serialize (&self) -> Vec<u8> {
		bitcode::encode(self)
	}

	/// Save the package to a file.
	pub fn writeToFile(&self, filename: impl AsRef<Path>) -> anyhow::Result<()> {
		Ok(std::fs::write(filename, self.serialize())?)
	}
}
