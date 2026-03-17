
//////
//
// Imports
//

// GLM library
use nalgebra_glm as glm;

// Local imports
use crate::math::*;



//////
//
// Helpers
//

fn assert_close_f32(actual: f32, expected: f32) {
	let diff = actual - expected;
	assert!(diff.abs() <= f32::EPSILON, "expected {expected}, got {actual} (diff {diff})");
}

fn assert_close_f64(actual: f64, expected: f64) {
	let diff = actual - expected;
	assert!(diff.abs() <= f64::EPSILON, "expected {expected}, got {actual} (diff {diff})");
}

fn assert_close_vec2(actual: &glm::Vec2, expected: &glm::Vec2) {
	assert_close_f32(actual[0], expected[0]);
	assert_close_f32(actual[1], expected[1]);
}



//////
//
// Tests
//

#[test]
fn test_roundUpToQuantization_zero_is_stable() {
	assert_eq!(roundUpToQuantization(0usize, 4usize), 0);
}

#[test]
fn test_roundUpToQuantization_preserves_exact_multiples() {
	assert_eq!(roundUpToQuantization(8usize, 4usize), 8);
	assert_eq!(roundUpToQuantization(256u32, 256u32), 256);
}

#[test]
fn test_roundUpToQuantization_non_multiples() {
	assert_eq!(roundUpToQuantization(1usize, 4usize), 4);
	assert_eq!(roundUpToQuantization(7usize, 4usize), 8);
	assert_eq!(roundUpToQuantization(257u32, 256u32), 512);
}

#[test]
fn test_roundUpToQuantization_stride_one_is_identity() {
	assert_eq!(roundUpToQuantization(0usize, 1usize), 0);
	assert_eq!(roundUpToQuantization(17usize, 1usize), 17);
}

#[test]
fn test_roundUpToQuantization_signed_values() {
	assert_eq!(roundUpToQuantization(-5i32, 4i32), -4);
	assert_eq!(roundUpToQuantization(-4i32, 4i32), -4);
}

#[test]
fn test_lerp_scalar_endpoints() {
	let v1 = -4.0f32;
	let v2 = 8.0f32;
	assert_close_f32(lerp(v1, v2, 0.0), v1);
	assert_close_f32(lerp(v1, v2, 1.0), v2);
}

#[test]
fn test_lerp_scalar_intermediate_values() {
	let v1 = -4.0f32;
	let v2 = 8.0f32;
	assert_close_f32(lerp(v1, v2, 0.25), -1.0);
	assert_close_f32(lerp(v1, v2, 0.5), 2.0);
}

#[test]
fn test_lerp_scalar_equal_inputs() {
	let value = 3.5f32;
	assert_close_f32(lerp(value, value, 0.0), value);
	assert_close_f32(lerp(value, value, 0.37), value);
	assert_close_f32(lerp(value, value, 1.0), value);
}

#[test]
fn test_lerp_scalar_extrapolates_without_clamping() {
	let v1 = 2.0f32;
	let v2 = 10.0f32;
	assert_close_f32(lerp(v1, v2, -0.5), -2.0);
	assert_close_f32(lerp(v1, v2, 1.5), 14.0);
}

#[test]
fn test_lerp_scalar_is_symmetric() {
	let v1 = -1.25f32;
	let v2 = 7.75f32;
	let t = 0.3f32;
	assert_close_f32(lerp(v1, v2, t), lerp(v2, v1, 1.0 - t));
}

#[test]
fn test_lerp_vec2_interpolates_componentwise() {
	let v1 = glm::vec2(-2.0, 4.0);
	let v2 = glm::vec2(6.0, -8.0);
	assert_close_vec2(&lerp(v1, v2, 0.25), &glm::vec2(0.0, 1.0));
	assert_close_vec2(&lerp(v1, v2, 0.5), &glm::vec2(2.0, -2.0));
}

#[test]
fn test_lerp_vec2_extrapolates_componentwise() {
	let v1 = glm::vec2(1.0, -3.0);
	let v2 = glm::vec2(5.0, 9.0);
	assert_close_vec2(&lerp(v1, v2, -0.5), &glm::vec2(-1.0, -9.0));
	assert_close_vec2(&lerp(v1, v2, 1.5), &glm::vec2(7.0, 15.0));
}

#[test]
fn test_smoothstep_f32_reference_values() {
	assert_close_f32(smoothstep(0.0f32), 0.0);
	assert_close_f32(smoothstep(0.5f32), 0.5);
	assert_close_f32(smoothstep(1.0f32), 1.0);
}

#[test]
fn test_smoothstep_f64_reference_values() {
	assert_close_f64(smoothstep(0.0f64), 0.0);
	assert_close_f64(smoothstep(0.5f64), 0.5);
	assert_close_f64(smoothstep(1.0f64), 1.0);
}

#[test]
fn test_smoothstep_extrapolates_outside_unit_interval() {
	assert_close_f32(smoothstep(-1.0f32), 5.0);
	assert_close_f32(smoothstep(2.0f32), -4.0);
	assert_eq!(smoothstep(-1i32), 5);
	assert_eq!(smoothstep(2i32), -4);
}

#[test]
fn test_smoothLerp_scalar_endpoints() {
	let v1 = -4.0f32;
	let v2 = 8.0f32;
	assert_close_f32(smoothLerp(v1, v2, 0.0), v1);
	assert_close_f32(smoothLerp(v1, v2, 1.0), v2);
}

#[test]
fn test_smoothLerp_scalar_reference_values() {
	let v1 = -4.0f32;
	let v2 = 8.0f32;
	assert_close_f32(smoothLerp(v1, v2, 0.25), -2.125);
	assert_close_f32(smoothLerp(v1, v2, 0.5), 2.0);
	assert_close_f32(smoothLerp(v1, v2, 0.75), 6.125);
}

#[test]
fn test_smoothLerp_scalar_equal_inputs() {
	let value = 3.5f32;
	assert_close_f32(smoothLerp(value, value, 0.0), value);
	assert_close_f32(smoothLerp(value, value, 0.37), value);
	assert_close_f32(smoothLerp(value, value, 1.0), value);
}

#[test]
fn test_smoothLerp_scalar_extrapolates_via_smoothstep()
{
	let v1 = 2.0f32;
	let v2 = 10.0f32;
	assert_close_f32(smoothLerp(v1, v2, -1.0), lerp(v1, v2, smoothstep(-1.0)));
	assert_close_f32(smoothLerp(v1, v2, -1.0), 42.0);
	assert_close_f32(smoothLerp(v1, v2, 2.0), lerp(v1, v2, smoothstep(2.0)));
	assert_close_f32(smoothLerp(v1, v2, 2.0), -30.0);
}

#[test]
fn test_smoothLerp_vec2_extrapolates_componentwise() {
	let v1 = glm::vec2(1.0, -3.0);
	let v2 = glm::vec2(5.0, 9.0);
	let t = 2.0f32;
	assert_close_vec2(&smoothLerp(v1, v2, t), &lerp(v1, v2, smoothstep(t)));
	assert_close_vec2(&smoothLerp(v1, v2, t), &glm::vec2(-15.0, -51.0));
}

#[test]
fn test_smoothLerp_vec2_interpolates_componentwise() {
	let v1 = glm::vec2(-2.0, 4.0);
	let v2 = glm::vec2(6.0, -8.0);
	let t = 0.25f32;
	assert_close_vec2(&smoothLerp(v1, v2, t), &lerp(v1, v2, smoothstep(t)));
	assert_close_vec2(&smoothLerp(v1, v2, t), &glm::vec2(-0.75, 2.125));
}
