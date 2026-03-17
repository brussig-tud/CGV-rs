
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
// Traits
//

/// The trait used to represent generic numbers throughout the *CGV-rs*. It basically relies on [`nalgebra_glm::Number`]
/// to do the heavy lifting, but adds various constructors to enable formulating generic expressions with constants.
pub trait Number<S=i8>: glm::Number+From<S>
{
	///
	#[inline(always)]
	fn new (value: S) -> Self {
		value.into()
	}

	///
	#[inline(always)]
	fn two () -> Self where Self: From<i8> {
		2.into()
	}

	///
	#[inline(always)]
	fn three () -> Self where Self: From<i8> {
		3.into()
	}
}
impl<S: Sized, T: glm::Number+From<S>> Number<S> for T {}



//////
//
// Functions
//

/// Quantize the given number with upwards rounding, i.e. return the smallest multiple of `stride` that is greater than
/// or equal to `number`.
///
/// # Arguments
///
/// * `number` – The number to quantize.
/// * `stride` – The quantization step size.
///
/// # Returns
///
/// The quantized value of `number`.
pub fn roundUpToQuantization<T: Copy + Rem<Output=T> + Add<Output=T> + Sub<Output=T>> (number: T, stride: T) -> T {
	number + ((stride - (number % stride)) % stride)
}

/// Performs linear interpolation between two values.
///
/// # Arguments
///
/// * `v1` – The value at `t=0`.
/// * `v2` – The value at `t=1`.
/// * `t` – The interpolation factor.
///
/// # Returns
///
/// The value of the expression `v1·(1-t) + v2·t`, that is, the linearly interpolated value between `v1` and `v2`.
pub fn lerp<S: glm::Number, V: Copy+Mul<S>+Add> (v1: V, v2: V, t: S) -> V
	where <V as Mul<S>>::Output: Copy+Add<Output=V>
{
	v1*(S::one()-t) + v2*t
}

/// Generic evaluation of the polynomial -2·*t*³ + 3·*t*² for *C*1-smooth cubic interpolation.
///
/// # Arguments
///
/// * `t_linear` – The linear interpolation factor to transform. This will not be clamped; callers that require
///                interpolation strictly within the range `[0, 1]` must clamp this argument themselves.
///
/// # Returns
///
/// The value of the cubic interpolator polynomial at `t_linear`.
pub fn smoothstep<T: Number> (t_linear: T) -> T {
	let t2 = t_linear*t_linear;
	t2*(T::three()-T::two()*t_linear)
}

/// Performs *C*1-smooth cubic interpolation between two values.
///
/// # Arguments
///
/// * `v1` – The value at `t_linear=0`.
/// * `v2` – The value at `t_linear=1`.
/// * `t_linear` – The linear interpolation factor, internally transformed via [`smoothstep`].
///
/// # Returns
///
/// The cubically interpolated value at `t_linear`.
pub fn smoothLerp<S: Number, V: Copy+Mul<S>+Add> (v1: V, v2: V, t_linear: S) -> V
	where <V as Mul<S>>::Output: Copy+Add<Output=V>
{
	lerp(v1, v2, smoothstep(t_linear))
}
