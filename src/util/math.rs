
//////
//
// Imports
//

// Standard library
/* Nothing here yet */



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
