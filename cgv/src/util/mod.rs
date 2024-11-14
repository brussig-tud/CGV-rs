
//////
//
// Imports
//

// Standard library
// /* nothing here yet */



//////
//
// Module definitions
//

/// Submodule providing unsafe utilities
#[cfg(not(feature="buildScriptUsage"))]
pub mod notsafe;

/// Submodule providing assorted math utilities.
#[cfg(not(feature="buildScriptUsage"))]
pub mod math;

/// Submodule providing operations on file system paths.
pub mod path;



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

#[inline(always)]
pub fn detachRef<T: ?Sized> (reference: &T) -> &T {
	let ptr = reference as *const T;
	let reference = unsafe { &*ptr };
	reference
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
pub fn mutify<T: ?Sized> (reference: &T) -> &'static mut T
{
	#[allow(invalid_reference_casting)]
	unsafe { &mut *((reference as *const T) as *mut T) }
}

/// Creates an (invalid if derefenced) reference to an object of the specified type.
///
/// # Returns
///
/// A `'static` reference to some object of the specified type. Must not be dereferenced.
#[inline(always)]
pub const fn defaultRef<T> () -> &'static T {
	unsafe { &*(1usize as *const T) }
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
