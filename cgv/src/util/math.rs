
//////
//
// Imports
//

// Standard library
use std::ops::*;



//////
//
// Macros
//

/// Converts (for constant inputs at compile time) an angle given in degrees into radians.
///
/// # Arguments
///
/// * `deg` – An angle in degrees.
///
/// # Returns
///
/// The given angle in radians.
macro_rules! deg2rad { ($deg:expr) => {$deg * 3.1415926535897932384626433832795/180.} }
#[allow(unused_imports)] pub(crate) use deg2rad;

/// Converts (for constant inputs at compile time) an angle given in radians into degrees.
///
/// # Arguments
///
/// * `rad` – An angle in radians.
///
/// # Returns
///
/// The given angle in degrees.
macro_rules! rad2deg { ($rad:expr) => {$rad * 180./3.1415926535897932384626433832795.} }
#[allow(unused_imports)] pub(crate) use rad2deg;



//////
//
// Functions
//

/// Returns the next biggest multiple (times the given factor) of the given number.
///
/// # Arguments
///
/// * `number` – t.b.d
/// * `factor` – t.b.d.
///
/// # Returns
///
/// The number rounded up to the nearest multiple of `factor`
pub fn alignToFactor<T: Copy + Rem<Output=T> + Add<Output=T> + Sub<Output=T>> (number: T, factor: T) -> T {
	number + factor - (number % factor)
}
