
//////
//
// Imports
//

// Standard library
use std::ops::*;

// GLM library
use glm;



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

/// Generic cubic polynomial for C1-smooth interpolation.
pub fn smoothstep (t_linear: f32) -> f32 {
	let t2 = t_linear*t_linear;
	glm::clamp_scalar(-2f32*t2*t_linear + 3f32*t2, 0f32, 1f32)
}

/// Generic C1-smooth cubic interpolation.
pub fn smoothLerp3 (v1: &glm::Vec3, v2: &glm::Vec3, t_linear: f32) -> glm::Vec3 {
	glm::mix(v1, v2, smoothstep(t_linear))
}
