
//////
//
// Imports
//

// Standard library
use std::{env, str::FromStr, sync::LazyLock, path::{Path, PathBuf}};

// Anyhow library
use anyhow::{anyhow, Result};

// Local imports
use crate::*;



//////
//
// Constants
//

/// The *Cargo* `CARGO_MANIFEST_DIR` environment variable as it was set during the build.
static MANIFEST_DIR: LazyLock<PathBuf> = LazyLock::new(||
	env!("CARGO_MANIFEST_DIR").into()
);

/// The directory of the executable that was run to start the current process.
static CURRENT_EXE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
	#[cfg(not(target_arch="wasm32"))] {
		let ce = std::env::current_exe().unwrap_or_else(
			|err| panic!("{}", err)
		);
		ce.parent().map_or("".into(), |path| path.into())
	}
	#[cfg(target_arch="wasm32")] {
		"".into()
	}
});

/// The target triple the module was built for according the build-time values of the *Cargo* `TARGET` environment
/// variable.
static TARGET_TRIPLE_CARGO: LazyLock<TargetTriple> = LazyLock::new(||
	TargetTriple::fromString(env!("CGV_TARGET_TRIPLE_CARGO").to_owned()).unwrap_or_else(|err| panic!("{}", err))
);

/// Typed description of the platform the module was built for according to the build-time values of the *Cargo*
/// `TARGET` environment variable.
static CGV_PLATFORM: LazyLock<SupportedPlatform> = LazyLock::new(||
	SupportedPlatform::fromTargetTriple(&TARGET_TRIPLE_CARGO, None).unwrap_or_else(|err| panic!("{}", err))
);



//////
//
// Enums
//

/// Enum of supported ARM sub-architectures.
#[allow(non_camel_case_types)]
#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum Aarch32SubArchitecture {
	Generic, v7, v7a, v7k, v7r, v7s, v8r
}
/// Shorthand alias for scoped access to enum values of [`Aarch32SubArchitecture`].
pub use Aarch32SubArchitecture as ARM32Sub;

/// Enum of supported ARM64 sub-architectures.
#[allow(non_camel_case_types)]
#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum Aarch64SubArchitecture {
	Generic, be, e, ec,
}
/// Shorthand alias for scoped access to enum values of [`Aarch64SubArchitecture`].
pub use Aarch64SubArchitecture as ARM64Sub;

/// Enum of supported x86_64 sub-architectures.
#[allow(non_camel_case_types)]
#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum X86_64SubArchitecture {
	Generic, h
}
/// Shorthand alias for scoped access to enum values of [`X86_64SubArchitecture`].
pub use X86_64SubArchitecture as X64Sub;

/// Enum of supported platform architectures.
#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum PlatformArchitecture
{
	/// Intel 686, also known as x86 (32bit).
	#[allow(non_camel_case_types)]
	i686,

	/// Intel x86_64, also known as AMD64.
	#[allow(non_camel_case_types)]
	x86_64(X86_64SubArchitecture),

	/// ARM 32bit.
	Aarch32(Aarch32SubArchitecture),

	/// ARM 64bit.
	Aarch64(Aarch64SubArchitecture),

	/// Regular *WebAssembly* with 4GB of available linear memory.
	Wasm32,

	/// *WebAssembly* with [memory64](https://github.com/WebAssembly/memory64/blob/main/proposals/memory64/Overview.md)
	/// feature.
	Wasm64
}
impl PlatformArchitecture {
	/// `true` if the architecture is from the WebAssembly family, `false` otherwise.
	#[inline(always)]
	pub fn isWasm (self) -> bool {
		self == Self::Wasm32 || self == Self::Wasm64
	}
}

/// Enum of supported vendors. The exact semantics of each vendor are ambiguous and not very rigorously defined,
/// therefore, no documentation is given for the entries.
#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum PlatformVendor {
	Unknown, Linux, Apple, PC, Win7
}

/// Enum of supported ABIs of the *Apple* *iOS* system.
#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum AppleiOSABI {
	Generic, MacABI, Sim
}

/// Enum of supported ABIs of the *Apple* *iOS* system.
#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum AppleEmbeddedABI {
	Generic, Sim
}

/// Enum of supported *Windows* ABIs.
#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum WindowsABI
{
	/// *Windows* PE executable linked against the *Microsoft* *Visual C* runtime.
	MSVC,

	/// *Windows* PE executable linked against *MinGW* *GNU*/GCC runtime.
	GNU,

	/// *Windows* PE executable linked against the UCRT (*Universal C Runtime*) using LLVM-based binutils.
	UCRT
}

/// Enum of supported *Linux* ABIs.
#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum LinuxABI {
	/// Linux executable linked against *GNU* glibc.
	GNU,

	/// Linux executable linked against the *MUSL* C standard library.
	MUSL
}

/// Enum of supported systems.
#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum PlatformSystem
{
	/// An *unknown* system. More often than not actually means a *generic* system.
	Unknown,

	/// *Darwin*, the common OS core of *Apple* operating systems (usable by desktop apps only).
	Darwin,

	/// The Linux kernel.
	Linux(LinuxABI),

	/// *Microsoft Windows*.
	Windows(WindowsABI),

	/// The *cygwin* *GNU*/*Linux* environment for *Microsoft Windows*.
	Cygwin,

	/// *iOS*, indicating a mobile application build.
	#[allow(non_camel_case_types)]
	iOS(AppleiOSABI),

	/// *visionos*, indicating an application build to run on the embedded OS of *Apple Vision* Mixed Reality devices.
	VisionOS(AppleEmbeddedABI),

	/// *tvos*, indicating an application build to run on the embedded OS of *Apple TV* devices.
	TVOS(AppleEmbeddedABI)
}



//////
//
// Classes
//

/// Representation of the [target triple](https://doc.rust-lang.org/cargo/appendix/glossary.html#target) that allows
/// easy individual access to all (sub-)components of the triple.
#[derive(Debug,Eq)]
pub struct TargetTriple<'this> {
	full: String,
	arch: &'this str,
	vendor: &'this str,
	sys: &'this str,
	abi: &'this str
}
impl TargetTriple<'_>
{
	/// Internal function to create a functionally uninitialized instance.
	#[inline]
	fn uninitialized() -> Self {
		let triple = String::default();
		let full: &'static str = statify(triple.as_str());
		Self {
			full: triple, arch: full, vendor: full, sys: full, abi: full
		}
	}

	/// Create from the given target triple descriptor string.
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
				sys.split_at(sys.len()).1
			};
			(sys, abi)
		};
		Ok(TargetTriple {
			full: triple,
			arch, vendor, sys, abi
		})
	}

	/// Borrow a slice over the full target triple descriptor string.
	pub fn full (&self) -> &str {
		return &self.full;
	}

	/// Borrow a slice over the architecture part of the target triple descriptor string.
	pub fn arch (&self) -> &str {
		return &self.arch;
	}

	/// Borrow a slice over the vendor part of the target triple descriptor string.
	pub fn vendor (&self) -> &str {
		return &self.vendor;
	}

	/// Borrow a slice over the system part of the target triple descriptor string.
	pub fn sys (&self) -> &str {
		return &self.sys;
	}

	/// Borrow a slice over the ABI part of the target triple descriptor string.
	pub fn abi (&self) -> &str {
		return &self.abi;
	}
}
impl FromStr for TargetTriple<'_> {
	type Err = anyhow::Error;

	#[inline(always)]
	fn from_str (triple: &str) -> anyhow::Result<Self> {
		Self::fromString(triple.to_owned())
	}
}
impl PartialEq<TargetTriple<'_>> for TargetTriple<'_> {
	fn eq(&self, other: &TargetTriple<'_>) -> bool {
		self.full == other.full
	}
}
impl Clone for TargetTriple<'_>
{
	#[inline(always)]
	fn clone(&self) -> Self {
		let mut new = Self::uninitialized();
		new.clone_from(self);
		new
	}

	fn clone_from(&mut self, source: &Self) {
		self.full = source.full.clone();
		let offset = unsafe {
			// SAFETY: We are violating the "from the same allocation" invariant here, but we know the behavior of the
			//         platforms we support (desktop[x86,arm] and WASM) and there it works exactly like we expect. The
			//         other invariants hold, most notably "distance must be multiple of size of T>" since we are
			//         dealing with T=u8 here which is the smallest possible address difference on all supported
			//         platforms
			self.full.as_ptr().offset_from(source.full.as_ptr())
		};
		unsafe {
			// SAFETY: The full string is identical to the input one, so we can use all the same indices and offsets
			//         from the input. Also, TargetTriple hides all its fields inside the private scope and defines no
			//         mutating methods, meaning Rust's aliasing rules are effectively never violated.
			self.arch = notsafe::offsetStr(&source.arch, offset);
			self.vendor = notsafe::offsetStr(&source.vendor, offset);
			self.sys = notsafe::offsetStr(&source.sys, offset);
			self.abi = notsafe::offsetStr(&source.abi, offset);
		}
	}
}
impl std::fmt::Display for TargetTriple<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.full)
	}
}

/// A typed representation of a platform that *CGV-rs* explicitly supports, following the [`TargetTriple`] scheme.
#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub struct SupportedPlatform
{
	/// The *architecture* of the platform.
	pub arch: PlatformArchitecture,

	/// The *vendor* indicator of the platform.
	pub vendor: PlatformVendor,

	/// The *system* indicator of the platform.
	pub sys: PlatformSystem,

	/// If `Some`, indicates whether the platform was targeted in a debug build. If `None`, then this information is
	/// simply not available or does not apply to the context.
	pub debug: Option<bool>
}
impl SupportedPlatform
{
	/// Create the platform representation from a [`TargetTriple`] descriptor.
	pub fn fromTargetTriple (triple: &TargetTriple, debug: Option<bool>) -> Result<Self>
	{
		// Determine architecture
		let arch = match triple.arch
		{
			// Aarch64
			"aarch64"   => PlatformArchitecture::Aarch64(ARM64Sub::Generic),
			"aarch64_be"=> PlatformArchitecture::Aarch64(ARM64Sub::be),
			"arm64e"    => PlatformArchitecture::Aarch64(ARM64Sub::e),
			"arm64ec"   => PlatformArchitecture::Aarch64(ARM64Sub::ec),

			// Aarch32
			"arm"       => PlatformArchitecture::Aarch32(ARM32Sub::Generic),
			"armv7"     => PlatformArchitecture::Aarch32(ARM32Sub::v7),
			"armv7a"    => PlatformArchitecture::Aarch32(ARM32Sub::v7a),
			"armv7k"    => PlatformArchitecture::Aarch32(ARM32Sub::v7k),
			"armv7r"    => PlatformArchitecture::Aarch32(ARM32Sub::v7r),
			"armv7s"    => PlatformArchitecture::Aarch32(ARM32Sub::v7s),
			"armv8r"    => PlatformArchitecture::Aarch32(ARM32Sub::v8r),

			// IBM PC
			"i686"      => PlatformArchitecture::i686,
			"x86_64"    => PlatformArchitecture::x86_64(X64Sub::Generic),
			"x86_64h"   => PlatformArchitecture::x86_64(X64Sub::h),

			// WebAssembly
			"wasm32"    => PlatformArchitecture::Wasm32,
			"wasm64"    => PlatformArchitecture::Wasm64,

			// Unsupported
			_ => return Err(anyhow!("Unsupported platform architecture: {}", triple.arch))
		};

		// Determine vendor
		let vendor = match triple.vendor
		{
			// Aarch64
			"unknown" => PlatformVendor::Unknown,
			"linux"   => PlatformVendor::Linux,
			"apple"   => PlatformVendor::Apple,
			"pc"      => PlatformVendor::PC,
			"win7"    => PlatformVendor::Win7,

			// Unsupported
			_ => return Err(anyhow!("Unsupported platform vendor: {}", triple.vendor))
		};

		// Determine system
		// - shared error output
		#[inline(always)] fn unsupportedABI (sys: &str, abi: &str) -> Result<SupportedPlatform> {
			Err(anyhow!("Unsupported ABI on '{sys}': {abi}"))
		}
		// - actual parsing
		let sys = match triple.sys
		{
			"unknown" => PlatformSystem::Unknown,
			"darwin"  => PlatformSystem::Darwin,
			"linux"   => match triple.abi {
				"gnu"    => PlatformSystem::Linux(LinuxABI::GNU),
				"musl"   => PlatformSystem::Linux(LinuxABI::MUSL),
				  _      => return unsupportedABI(triple.sys, triple.abi)
			},
			"windows" => match triple.abi {
				"msvc"   => PlatformSystem::Windows(WindowsABI::MSVC),
				"gnu"    => PlatformSystem::Windows(WindowsABI::GNU),
				"gnullvm"=> PlatformSystem::Windows(WindowsABI::UCRT),
				  _      => return unsupportedABI(triple.sys, triple.abi)
			},
			"ios"     => match triple.abi {
				""       => PlatformSystem::iOS(AppleiOSABI::Generic),
				"macabi" => PlatformSystem::iOS(AppleiOSABI::MacABI),
				"sim"    => PlatformSystem::iOS(AppleiOSABI::Sim),
				  _      => return unsupportedABI(triple.sys, triple.abi)
			},
			"visionos"=> match triple.abi {
				""       => PlatformSystem::VisionOS(AppleEmbeddedABI::Generic),
				"sim"    => PlatformSystem::VisionOS(AppleEmbeddedABI::Sim),
				  _      => return unsupportedABI(triple.sys, triple.abi)
			},
			"tvos"    => match triple.abi {
				""       => PlatformSystem::TVOS(AppleEmbeddedABI::Generic),
				"sim"    => PlatformSystem::TVOS(AppleEmbeddedABI::Sim),
				  _      => return unsupportedABI(triple.sys, triple.abi)
			},

			// Unsupported
			_ => return Err(anyhow!("Unsupported platform system: {}", triple.sys))
		};

		// Done!
		Ok(Self { arch, vendor, sys, debug })
	}

	/// Create the platform representation from a full target triple descriptor string. This is equivalent to
	/// calling `Self::fromTargetTriple(SupportedPlatform::fromString(...))`
	#[inline(always)]
	pub fn fromString (triple: String, debug: Option<bool>) -> Result<Self> {
		Self::fromTargetTriple(&TargetTriple::fromString(triple)?, debug)
	}

	/// This is merely a convenience shorthand for [`Self::arch::isWasm`](PlatformArchitecture::isWasm).
	#[inline(always)]
	pub fn isWasm (&self) -> bool {
		self.arch.isWasm()
	}

	/// Evaluates to `true` if and only if [`Self::debug`] is `Some(true)`. Evaluates to `false` in all other cases.
	#[inline(always)]
	pub fn isDebug (&self) -> bool {
		self.debug.unwrap_or(false)
	}
}
impl FromStr for SupportedPlatform {
	type Err = anyhow::Error;

	fn from_str (triple: &str) -> anyhow::Result<Self> {
		Self::fromString(triple.to_owned(), None)
	}
}



//////
//
// Functions
//

/// Report the directory path that the [manifest file](https://doc.rust-lang.org/cargo/reference/manifest.html) of the
/// calling crate is located in. This equivalent to the expression
/// `&std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))`, with the difference that the returned `Path` slice has a
/// static lifetime.
pub fn manifestDir() -> &'static Path {
	&MANIFEST_DIR
}

/// Report the directory that the executable of the calling process was started from resides in. For WASM builds, this
/// will just be the empty path `""`.
pub fn currentExeDir() -> &'static Path {
	&CURRENT_EXE_DIR
}

/// Retrieve the [target triple](https://doc.rust-lang.org/cargo/appendix/glossary.html#target) that the calling code
/// was compiled for.
pub fn platformTargetTriple() -> &'static TargetTriple<'static> {
	&TARGET_TRIPLE_CARGO
}

/// Reference the typed platform descriptor of the platform the calling code was compiled for.
pub fn platform() -> &'static SupportedPlatform {
	&CGV_PLATFORM
}
