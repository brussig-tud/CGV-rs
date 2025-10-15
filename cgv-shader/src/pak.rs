
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

/// Error conditions when creating [`wgpu::ShaderModule`]s.
#[cfg(feature="wgpu_runtime")]
#[derive(Debug,PartialEq)]
pub enum CreateShaderModuleError
{
	#[doc=include_str!("_doc/_InvalidEntryPoint_withString.md")]
	InvalidEntryPoint(String),

	/// The package from which a module was requested does not contain an instance in the requested source type. Holds
	/// the `SourceType` that was not available.
	InvalidSourceType(WgpuSourceType)
}
#[cfg(feature="wgpu_runtime")]
impl Display for CreateShaderModuleError {
	fn fmt (&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let desc = match self {
			Self::InvalidEntryPoint(ep) => format!("invalid entry point: `{ep}`"),
			Self::InvalidSourceType(st) => format!("invalid source type: `{st}`")
		};
		write!(formatter, "InvalidEntryPointError[{desc}]")
	}
}
#[cfg(feature="wgpu_runtime")]
impl std::error::Error for CreateShaderModuleError {}


/// Error conditions when building [shader program instances](Program).
#[derive(Debug)]
pub enum ProgramInstanceBuildError
{
	#[doc=include_str!("_doc/_InvalidEntryPoint_withString.md")]
	InvalidEntryPoint(String),

	/// Some other (nested) error caused by external code.
	External(anyhow::Error)
}
impl Display for ProgramInstanceBuildError {
	fn fmt (&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		match self {
			Self::InvalidEntryPoint(ep)
				=> write!(formatter, "ProgramInstanceBuildError[invalid entry point: `{ep}`]"),

			Self::External(e) => {
				writeln!(formatter, "ProgramInstanceBuildError[<Nested>]:")?;
				write!(formatter, "-> {e}")
			}
		}
	}
}
impl std::error::Error for ProgramInstanceBuildError {}



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

	pub fn code (&self, entryPointName: Option<&str>) -> Option<&[u8]> {
		self.entryPoints.get(&entryPointName.map(|name| name.to_owned())).map(
			|ep| ep.code.as_slice()
		)
	}
}

/// Represents a package of one or more ready-to-use instances of a shader program compiled to different representations
/// (e.g. *SPIR-V* or *WGSL*).
#[derive(bitcode::Encode,bitcode::Decode)]
pub struct Package {
	name: String,
	instances: BTreeMap<WgpuSourceType, Program>
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
	pub fn withSingleInstance (sourceType: WgpuSourceType, program: Program, name: Option<&str>)
	-> anyhow::Result<Self> {
		Ok(Self {
			name: name.unwrap_or("<unnamed>").to_owned(),
			instances: BTreeMap::from([(sourceType, program)])
		})
	}

	/// Internal helper function to create a single program instance from a *Slang* shader source file.
	///
	/// # Arguments
	///
	/// * `slangContext` – The [`SlangContext`] to use for building the sole instance in the package.
	/// * `filepath` – The path to the file containing the *Slang* source code.
	/// * `entryPoints` – Optionally a set of entry points to include in the sole instance in the package. This set can
	///                   contain [`None`] to indicate that the non-specialized program with all entry points should be
	///                   included in the instance.
	///
	/// # Returns
	///
	/// A tuple with the built [program instance](Program) and, for convenience, the [`SourceType`] it was built for as
	/// dictated by the passed-in `slangContext`.
	#[cfg(feature="slang_runtime")]
	fn buildSingleInstanceFromSlang (
		slangContext: &slang::Context, filepath: impl AsRef<Path>, entryPoints: Option<&BTreeSet<Option<&str>>>
	) -> Result<(Program, WgpuSourceType), ProgramInstanceBuildError>
	{
		// Compile Slang code
		let slangProg = slangContext.buildProgram(filepath).or_else(
			|e| Err(ProgramInstanceBuildError::External(e))
		)?;

		// Create the program instance for the compilation target indicated by the Slang context, with code for the
		// indicated entry points if any, or the generic code if no entry points were specified.
		if let Some(entryPoints) = entryPoints
		{
			let mut progInstance = Program { entryPoints: BTreeMap::new() };
			for &entryPoint in entryPoints
			{
				if let Some(entryPointName) = entryPoint
				{
					if let Some(ep) = slangProg.entryPointProgs().iter().find(
						|&ep| ep.slangEntryPoint().function_reflection().name() == Some(entryPointName)
					){
						progInstance.addEntryPoint(Some(entryPointName), ep.programBytecode().to_owned());
					}
					else {
						return Err(ProgramInstanceBuildError::InvalidEntryPoint(entryPointName.to_owned()))
					}
				}
				else {
					progInstance.addEntryPoint(None, slangProg.allEntryPointsProg().to_owned());
				}
			}
			Ok((progInstance, slangContext.targetType()))
		}
		else {
			// Only include the generic program that includes code paths from all entry points
			Ok((
				Program::generic(slangProg.allEntryPointsProg().to_owned()), slangContext.targetType()
			))
		}
	}

	/// Create the package from the given *Slang* shader source file, compiling it under several contexts to produce
	/// different instances for the [source types](SourceType) each [`slang::Context`] is set up for.
	#[cfg(feature="slang_runtime")]
	pub fn fromSlangMultiple (
		slangContexts: &[&slang::Context], filename: impl AsRef<Path>, entryPoints: Option<BTreeSet<Option<&str>>>
	) -> anyhow::Result<Self>
	{
		let mut package = Self { name: filename.as_ref().display().to_string(), instances: BTreeMap::new() };
		for &slangContext in slangContexts {
			let (program, sourceType) = Self::buildSingleInstanceFromSlang(
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
		Self::fromSlangMultiple(&[slangContext], filename, entryPoints)
	}

	/// Add an instance of the program for the given source type to the package. If there is already an instance for
	/// the given source type, it will be replaced.
	pub fn addInstance (&mut self, sourceType: WgpuSourceType, program: Program) {
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
							/*device.create_shader_module_passthrough(
								wgpu::ShaderModuleDescriptorPassthrough::SpirV(
									wgpu::ShaderModuleDescriptorSpirV {label, source: wgpu::util::make_spirv_raw(code)}
								)
							)*/
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
	pub fn writeToFile(&self, filename: impl AsRef<Path>) -> anyhow::Result<()> {
		Ok(std::fs::write(filename, self.serialize())?)
	}
}
