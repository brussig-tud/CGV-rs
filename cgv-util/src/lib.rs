
//////
//
// Language config
//

// Eff this convention.
#![allow(non_snake_case)]

// We are a utilities library ffs...
#![allow(dead_code)]
#![allow(unused_macros)]

// No point allowing unstable features if we still get warnings.
#![allow(incomplete_features)]

// Experimental language features
#![feature(generic_const_exprs)] // required for notsafe::Phony
#![feature(str_from_raw_parts)]  // required for notsafe::offsetStr



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



//////
//
// Imports
//

// Standard library
use std::ops::{Deref, DerefMut};

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

/// A zero-cost wrapper around [`Vec`] that dereferences to a slice of references to its element type. Very useful for
/// convenience functions that create an array of things that require caller ownership, but which the owner will most
/// typically just borrow to other functions (often as a slice of references).
pub struct BorrowVec<'this, T: 'this> {
	vec: Vec<T>,
	borrowVec: Vec<&'this T>,
}
impl<'this, T: 'this> BorrowVec<'this, T>
{
	/// Create the `BorrowVec` by moving in the given regular [`Vec`].
	///
	/// # Arguments
	///
	/// * `vec` – The regular vec to wrap with the new `BorrowVec`.
	///
	/// # Returns
	///
	/// The `BorrowVec` wrapping `vec`.
	#[inline(always)]
	pub fn new (vec: Vec<T>) -> Self {
		let slice = unsafe {
			// SAFETY: we move ownership of this memory into a struct that cannot outlive this memory, so the self-
			//         references to it that this struct will hold cannot outlive it either.  Also, BorrowVec hides all
			//         its fields inside the private scope and defines no mutating methods, meaning Rust's aliasing
			//         rules are effectively never violated.
			&*(vec.as_slice() as *const [T])
		};
		Self { vec, borrowVec: slice.into_iter().map(|elem| elem).collect() }
	}

	/// Borrow a slice of the Vector of **owned** elements.
	pub fn owned (&self) -> &[T] {
		self.vec.as_slice()
	}

	/// Obtain a slice of the Vector of **borrowed** elements.
	pub fn borrowed (&self) -> &[&T] {
		self.borrowVec.as_slice()
	}
}
impl<'this, T: 'this> From<Vec<T>> for BorrowVec<'this, T> {
	#[inline(always)]
	fn from (value: Vec<T>) -> Self {
		Self::new(value)
	}
}
impl<'this, T: 'this> std::ops::Deref for BorrowVec<'this, T> {
	type Target = [&'this T];

	#[inline(always)]
	fn deref (&self) -> &Self::Target {
		self.borrowVec.as_slice()
	}
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
