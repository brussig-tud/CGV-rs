
//////
//
// Language config
//

// Eff this convention.
#![allow(non_snake_case)]

// We are a utilities library ffs...
#![allow(dead_code)]
#![allow(unused_macros)]



//////
//
// Module definitions
//

/// Submodule providing some useful datastructures
pub mod ds;

/// Submodule providing operations on the file system.
pub mod fs;

/// Submodule providing assorted math utilities.
pub mod math;

/// Submodule providing utilities for reasoning about meta-related things like the current build platform etc.
pub mod meta;

/// Submodule providing unsafe utilities
pub mod notsafe;

/// Submodule providing operations on file system paths.
pub mod path;

/// Submodule providing unique entity generators (IDs etc.)
pub mod unique;

/// Submodule providing assorted utilities for use by unit tests (rarely useful outside of that).
pub mod test;

/// Unit tests
#[cfg(test)]
mod tests;



//////
//
// Imports
//

// Standard library
use std::ops::{Deref, DerefMut};

// static_assertions
pub use static_assertions::{self, *}; // re-export

// Uuid library
pub use uuid; // re-export

// Normlize-path library
pub use normalize_path; // re-export



//////
//
// Macros
//

/// Returns a UTF-8 encoded `'static` string slice containing the full path to the indicated location inside the source
/// tree of the caller.
///
/// # Arguments
///
/// * `path` – The path inside the crate root folder (must, by convention, always start with a `/`).
///
/// # Returns
///
/// A `'static` string slice containing the verbatim characters of the sourced file.
#[macro_export]
macro_rules! pathInsideCrate {
	($path:expr) => {
		concat!(env!("CARGO_MANIFEST_DIR"), $path)
	};
}

/// Reads a UTF-8 encoded file located inside the source tree of the caller into a static string slice at compile time.
///
/// # Arguments
///
/// * `file` – The path of the file, indicated from the crate root folder (i.e. must always start with a `/`).
///
/// # Returns
///
/// A `'static` string slice containing the verbatim characters of the sourced file.
#[macro_export]
macro_rules! sourceFile {
	($file:expr) => {
		include_str!(concat!(env!("CARGO_MANIFEST_DIR"), $file))
	};
}

/// Reads a file located inside the source tree of the caller verbatim into a static `u8` slice at compile time,
/// treating the file as a blob.
///
/// # Arguments
///
/// * `file` – The path of the file, indicated from the crate root folder (i.e. must always start with a `/`).
///
/// # Returns
///
/// A `'static` slice of `u8` containing the bytes of the sourced file.
#[macro_export]
macro_rules! sourceBytes {
	($file:expr) => {
		include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), $file))
	};
}

/// Reads a UTF-8 encoded file located inside the *Cargo* build tree of the caller (as specified by the *Cargo*
/// environment variable `OUT_DIR`) into a `'static` string slice at compile time.
///
/// # Arguments
///
/// * `file` – The path of the file, indicated from the *Cargo* `OUT_DIR` as root (i.e. must always start with a `/`).
///
/// # Returns
///
/// A `'static` string slice containing the verbatim characters of the sourced file.
#[macro_export]
macro_rules! sourceGeneratedFile {
	($file:expr) => {
		include_str!(concat!(env!("OUT_DIR"), $file))
	};
}

/// Reads a file located inside the *Cargo* build tree of the caller (as specified by the *Cargo* environment variable
/// `OUT_DIR`) verbatim into a `'static` `u8` slice at compile time, treating the file as a blob.
///
/// # Arguments
///
/// * `file` – The path of the file, indicated from the *Cargo* `OUT_DIR` as root (i.e. must always start with a `/`).
///
/// # Returns
///
/// A `'static` slice of `u8` containing the bytes of the sourced file.
#[macro_export]
macro_rules! sourceGeneratedBytes {
	($file:expr) => {
		include_bytes!(concat!(env!("OUT_DIR"), $file))
	};
}



//////
//
// Structs
//

///
#[derive(Debug,Ord,PartialOrd,Eq,PartialEq,Copy,Clone)]
pub struct LaterInit<Object> {
	object: Option<Object>
}
impl<Object> LaterInit<Object>
{
	///
	#[inline(always)]
	pub fn uninit () -> Self {
		Self { object: None }
	}

	///
	pub fn with (object: Object) -> Self { Self {
		object: Some(object)
	}}

	///
	#[inline(always)]
	pub fn set (&mut self, object: Object) {
		self.object.replace(object);
	}

	///
	#[inline(always)]
	pub fn swap (&mut self, object: Object) -> Option<Object> {
		self.object.replace(object)
	}

	///
	#[inline(always)]
	pub fn isInitialized (&self) -> bool {
		self.object.is_some()
	}
}
impl<Object: Default> Default for LaterInit<Object> {
	fn default () -> Self { Self {
		object: Some(Object::default())
	}}
}
impl<Object> Deref for LaterInit<Object> {
	type Target = Object;

	#[inline(always)]
	fn deref (&self) -> &Self::Target {
		self.object.as_ref().expect("a LaterInit should be initialized before dereferencing")
	}
}
impl<Object> DerefMut for LaterInit<Object> {
	#[inline(always)]
	fn deref_mut (&mut self) -> &mut Self::Target {
		self.object.as_mut().expect("a LaterInit should be initialized before dereferencing")
	}
}
impl<Object> AsRef<Object> for LaterInit<Object> {
	#[inline(always)]
	fn as_ref (&self) -> &Object { self }
}
impl<Object> AsMut<Object> for LaterInit<Object> {
	#[inline(always)]
	fn as_mut (&mut self) -> &mut Object { self }
}




//////
//
// Functions
//

/// Converts any kind of data that can have its size known at compile or at runtime into a slice of u8.
///
/// # Arguments
///
/// * `data` – The data to slicify.
///
/// # Returns
///
/// A slice of `u8` over the bytes in memory of the provided data.
pub fn slicify<T: ?Sized> (data: &T) -> &[u8] {
	unsafe { std::slice::from_raw_parts(data as *const T as *const u8, size_of_val(data)) }
}

/// Converts any kind of data that can have its size known at compile or at runtime into a slice of elements of generic
/// type `E`.
///
/// # Arguments
///
/// * `data` – The data to slicify.
///
/// # Returns
///
/// A slice of `E` over the bytes in memory of the provided data.
pub fn slicifyInto<T: Sized, E> (data: &T) -> &[E] {
	let remainder = const { size_of::<T>() }  %  const { size_of::<E>() };
	assert_eq!(remainder,   0);
	unsafe { std::slice::from_raw_parts(
		data as *const T as *const E,    const { size_of::<T>() }  /  const { size_of::<E>() })
	}
}

/// If the given option contains a string or string slice, returns an option containing the concatenation of the two
/// inputs.
///
/// # Arguments
///
/// * `option` – The optional string.
/// * `concat` – The string to concatenate to the option in case it does contain something.
///
/// # Returns
///
/// The concatenation of both strings in case `option` contained something, [`None`] otherwise.
#[inline(always)]
pub fn concatIfSome<Str: AsRef<str>> (option: &Option<Str>, concat: &str) -> Option<String> {
	option.as_ref().map(|source| format!("{}{concat}", source.as_ref()))
}

/// Construct a [`Range`](std::ops::Range) byte view on a subslice of the given `whole` `&str` slice.
///
/// # Arguments
///
/// * `whole` – The `&str` slice which the resulting byte range will be relative to (i.e. the parent slice).
/// * `sub` – A subslice of `whole` of which we want to get the corresponding byte `Range` for.
///
/// # Returns
///
/// The `Range` representing the bytes of `sub` inside `whole`.
///
/// # Panics
///
/// When `sub` is not a subslice of `whole`, i.e. when the start or end character of `sub` (or both) lie outside of
/// `whole`.
pub fn substrByteRange (whole: &str, sub: &str) -> std::ops::Range<usize>
{
	// Preamble
	let wholeStart = whole.as_ptr() as usize;
	let rangeStart = sub.as_ptr() as usize;

	// Sanity checks
	assert!(wholeStart <= rangeStart, "sub must be within whole, but was {sub}");
	assert!(
		rangeStart + sub.len() <= wholeStart + whole.len(),
		"rangeStart + sub length must be smaller than wholeStart + whole length, but was {}",
		rangeStart + sub.len()
	);

	// Calculate and construct
	let offset = rangeStart - wholeStart;
	offset..(offset+sub.len())
}
