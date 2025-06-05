
//////
//
// Imports
//

// Standard library
use std::{env, str::FromStr};

// Anyhow library
use anyhow::{anyhow, Result};

// Local imports
use crate::*;



//////
//
// Constants
//

/// The target triple the module was built for according the the build-time values of the *Cargo* `TARGET` environment
/// variable.
#[ctor::ctor]
static TARGET_TRIPLE_CARGO: TargetTriple = {
	TargetTriple::fromString(env!("CGV_TARGET_TRIPLE_CARGO").to_owned()).unwrap_or_else(|err| panic!("{}", err))
};



//////
//
// Structs
//

/// Representation of the [target triple](https://doc.rust-lang.org/cargo/appendix/glossary.html#target) that allows
/// easy individual access to all (sub-)components of the triple.
pub struct TargetTriple {
	full: String,
	arch: &'static str,
	vendor: &'static str,
	sys: &'static str,
	abi: &'static str
}
impl TargetTriple
{
	pub fn fromString (triple: String) -> Result<Self>
	{
		let full: &'static str = statify(&triple);
		let generateTripleErrorMsg = || {
			Err(anyhow!("Invalid target triple: {full}"))
		};

		let tripleElems: Vec<&str> = full.splitn(3, '-').collect();
		if tripleElems.len() < 3 {
			return generateTripleErrorMsg();
		}
		let arch = tripleElems[0];
		let vendor = tripleElems[1];
		let (sys, abi) = {
			let sys_abi: Vec<&str> = tripleElems[2].split('-').collect();
			if sys_abi.len() > 2 {
				return generateTripleErrorMsg();
			};
			let sys = sys_abi[0];
			let abi = if sys_abi.len() > 1 {
				sys_abi[1]
			}
			else {
				sys.split_at(sys.len()-1).1
			};
			(sys, abi)
		};
		Ok(TargetTriple {
			full: triple,
			arch, vendor, sys, abi
		})
	}
	pub fn full (&self) -> &str {
		return &self.full;
	}
	pub fn arch (&self) -> &str {
		return &self.arch;
	}
	pub fn vendor (&self) -> &str {
		return &self.vendor;
	}
	pub fn sys (&self) -> &str {
		return &self.sys;
	}
	pub fn abi (&self) -> &str {
		return &self.abi;
	}
}
impl FromStr for TargetTriple {
	type Err = anyhow::Error;
	fn from_str (triple: &str) -> anyhow::Result<Self> {
		Self::fromString(triple.to_owned())
	}
}

/*/// Enum of abstract platform types that *CGV-rs* differentiates between
pub enum PlatformType<'triple>
{
	Native(NativeTriple<'triple>),
	Wasm(WasmTriple<'triple>)
}*/



//////
//
// Functions
//

/*/// Determine the [platform type](PlatformType) that the calling code is being built for.
pub fn buildTimeTargetTriple() -> &'static TargetTriple {
	let
}*/