
//////
//
// Imports
//

// Standard library
use std::ops::*;

// GLM library
use nalgebra_glm as glm;



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
#[macro_export]
macro_rules! deg2rad { ($deg:expr) => {$deg * 3.1415926535897932384626433832795/180.} }
pub use deg2rad;

/// Converts (for constant inputs at compile time) an angle given in radians into degrees.
///
/// # Arguments
///
/// * `rad` – An angle in radians.
///
/// # Returns
///
/// The given angle in degrees.
#[macro_export]
macro_rules! rad2deg { ($rad:expr) => {$rad * 180./3.1415926535897932384626433832795} }
pub use rad2deg;



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
	glm::clamp_scalar(-2.*t2*t_linear + 3.*t2, 0., 1.)
}

/// Generic C1-smooth cubic interpolation for scalars.
pub fn smoothLerp (v1: f32, v2: f32, t_linear: f32) -> f32 {
	glm::mix_scalar(v1, v2, smoothstep(t_linear))
}

/// C1-smooth cubic interpolation for 2D vectors.
pub fn smoothLerp2 (v1: &glm::Vec2, v2: &glm::Vec2, t_linear: f32) -> glm::Vec2 {
	glm::mix(v1, v2, smoothstep(t_linear))
}

/// C1-smooth cubic interpolation for 3D vectors.
pub fn smoothLerp3 (v1: &glm::Vec3, v2: &glm::Vec3, t_linear: f32) -> glm::Vec3 {
	glm::mix(v1, v2, smoothstep(t_linear))
}
