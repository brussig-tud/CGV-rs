
//////
//
// Imports
//

// Standard library
use std::ops::{Deref, DerefMut};



//////
//
// Module definitions
//

/// Submodule providing some useful datastructures
#[cfg(not(feature="buildScriptUsage"))]
pub mod ds;

/// Submodule providing unsafe utilities
#[cfg(not(feature="buildScriptUsage"))]
pub mod notsafe;

/// Submodule providing assorted math utilities.
#[cfg(not(feature="buildScriptUsage"))]
pub mod math;

/// Submodule providing operations on file system paths.
pub mod path;

/// Submodule providing various reusable UI widgets
#[cfg(not(feature="buildScriptUsage"))]
pub mod widgets;



//////
//
// Macros
//

/// Reads a UTF-8 encoded file into a static string slice at compile time, with the path always being relative to the
/// Crate root directory.
///
/// # Arguments
///
/// * `file` – The path of the file, indicated from the repository root (i.e. must always start with a `/`).
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

#[allow(unused_imports)]
pub use sourceFile;

/// Reads a file verbatim into a static `u8` slice, treating the file as a blob.
///
/// # Arguments
///
/// * `file` – The path of the file, indicated from the repository root (i.e. must always start with a `/`).
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
#[allow(unused_imports)]
pub use sourceBytes;



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
/// A `'static` slice of `u8` over the bytes in memory of the provided data.
pub fn slicify<T: ?Sized> (data: &T) -> &'static [u8] {
	unsafe { std::slice::from_raw_parts(data as *const T as *const u8, size_of_val(data)) }
}

/// Decorates the given reference with a `'static` lifetime.
///
/// # Arguments
///
/// * `reference` – The reference to statify.
///
/// # Returns
///
/// A `'static` reference to the data that the input reference pointed to.
#[inline(always)]
pub fn statify<T: ?Sized> (reference: &T) -> &'static T {
	unsafe { &*(reference as *const T) }
}

/// Returns a mutable reference to the given object behind the given immutable reference.
///
/// # Arguments
///
/// * `reference` – The reference to mutify.
///
/// # Returns
///
/// A mutable `'static` reference to the data that the input reference pointed to.
#[inline(always)]
pub fn mutify<T: ?Sized> (reference: &T) -> &'static mut T {
	#[allow(invalid_reference_casting)]
	unsafe { &mut *((reference as *const T) as *mut T) }
}

/// Turns a [`Ref`](std::cell::Ref) into an actual (primitive) reference.
///
/// # Arguments
///
/// * `reference` – The wrapper to turn into a primitive reference.
///
/// # Returns
///
/// A `'static` reference to the same data the input [`RefMut`](std::cell::Ref) references.
#[inline(always)]
fn refify<T> (reference: std::cell::Ref<T>) -> &'static T {
	unsafe { &*(reference.deref() as *const T) }
}

/// Turns a [`RefMut`](std::cell::RefMut) into an actual (primitive) mutable reference.
///
/// # Arguments
///
/// * `reference` – The wrapper to turn into a primitive mutable reference.
///
/// # Returns
///
/// A mutable `'static` reference to the same data the input [`RefMut`](std::cell::RefMut) references.
#[inline(always)]
fn refify_mut<T> (reference: std::cell::RefMut<T>) -> &'static mut T {
	let mut refMut = reference;
	unsafe { &mut *(refMut.deref_mut() as *mut T) }
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
