
//////
//
// Imports
//

// Standard library
use std::f32::consts;

// rand library
use rand::prelude::*;

// CGV re-imports
use cgv::{self, rand, rand_distr, glm, tracing};

// Local imports
use crate::*;



//////
//
// Functions
//

/// Sample a uniformly random unit vector on the 2-sphere.
pub fn sampleUnitVec (rng: &mut impl rand::Rng) -> glm::Vec3 {
	// z is uniform in [-1, 1]; azimuthal angle is uniform in [0, 2π)
	let z: f32 = rng.random_range(-1f32..1.);
	let theta: f32 = rng.random_range(0f32..consts::TAU);
	let r = (1. - z * z).sqrt();
	glm::vec3(r * theta.cos(), r * theta.sin(), z)
}

/// Find a unit vector perpendicular to `v` by Gram-Schmidt orthogonalisation against a random candidate.
pub fn perpendicularUnit (v: &glm::Vec3, rng: &mut impl rand::Rng) -> glm::Vec3
{
	const TOLERANCE: f32 = 8.*f32::EPSILON;
	loop {
		let candidate = sampleUnitVec(rng);
		let proj = glm::dot(v, &candidate);
		let perp = candidate - v * proj;
		let len = glm::length(&perp);
		if len > TOLERANCE {
			return perp / len;
		}
		// Extremely unlikely parallel case – retry
	}
}

/// Generate new test data. Data points will be placed inside blocks, with the amount of blocks depending on the number
/// of data points such that a reasonable density is maintained. In essence, the generated data will take up more volume
/// the more data points are requested, keeping the density stable.
pub fn regenerateData (out: &mut Vec<DataPoint>, num: usize)
{
	////
	// Preamble

	// Data volume parameters
	let blockWidth = 2.;
	let meanRadius = 3f32/32.;
	let meanRadiusSigma = 2f32/32.;
	let pointsPerBlock = ((blockWidth/meanRadius).powi(3)/16.) as usize;
	let block0MinPoint = glm::vec3(-1., -1., -1.);

	// If num is less than we currently have, just remove the excess
	if num < out.len() {
		out.truncate(num);
		return;
	}


	////
	// Helpers

	/// Decode a linear 3D Morton code index into (bx, by, bz) block grid coordinates.
	/// Bits are interleaved as ...z1 y1 x1 z0 y0 x0, so x occupies bits 0,3,6,…, y bits 1,4,7,…, z bits 2,5,8,…
	fn decodeMorton3 (mut code: usize) -> (usize, usize, usize)
	{
		let (mut bx, mut by, mut bz) = (0usize, 0usize, 0usize);
		let mut bit = 0usize;
		while code > 0 {
			bx |= (code & 1) << bit; code >>= 1;
			by |= (code & 1) << bit; code >>= 1;
			bz |= (code & 1) << bit; code >>= 1;
			bit += 1;
		}
		(bx, by, bz)
	}

	/// Convert HSV (h in [0°,360°), s and v in [0,1]) to linear RGB.
	fn hsvToRgb (h: f32, s: f32, v: f32) -> (f32, f32, f32)
	{
		let h = h.rem_euclid(360f32);
		let hBy60 = h/60.;
		let c = v * s;
		let x = c * (1. - (hBy60.rem_euclid(2.)-1.).abs());
		let m = v - c;
		let (r, g, b) = match hBy60 as u32 {
			0 => (c, x, 0.),
			1 => (x, c, 0.),
			2 => (0., c, x),
			3 => (0., x, c),
			4 => (x, 0., c),
			_ => (c, 0., x),
		};
		(r + m, g + m, b + m)
	}


	////
	// Generation

	// Generate the missing points from out.len() up to num
	let mut rng = rand::rng();
	let mut blockIdx = 0;
	for i in out.len()..num
	{
		// Determine which block this point belongs to and decode its 3D grid position
		let (bx, by, bz) = {
			let newBlockIdx = i/pointsPerBlock;
			let (bx, by, bz) = decodeMorton3(newBlockIdx);
			if newBlockIdx > blockIdx {
				tracing::debug!("regenerateData: point {i}, spawning new Block #{blockIdx} → grid ({bx}, {by}, {bz})");
				blockIdx = newBlockIdx;
			}
			(bx, by, bz)
		};

		// Compute the minimum corner of this block.
		// x and y grow positively; z grows towards -∞ (OpenGL right-handed).
		let blockMin = block0MinPoint + glm::vec3(
			bx as f32 * blockWidth, by as f32 * blockWidth, -(bz as f32 * blockWidth),
		);

		// Uniform position inside the block
		let pos = blockMin + glm::vec3(
			rng.random_range(0f32..blockWidth), rng.random_range(0f32..blockWidth),
			rng.random_range(0f32..blockWidth),
		);

		// Radius from normal distribution N(meanRadius, meanRadiusSigma), clamped to a small positive minimum
		let radius = rand_distr::Normal::new(meanRadius, meanRadiusSigma).unwrap()
			.sample(&mut rng)
			.max(f32::EPSILON);

		// Random orientation: normal is a random unit vector; tangent is perpendicular to it
		let normal  = sampleUnitVec(&mut rng);
		let tangent = perpendicularUnit(&normal, &mut rng);

		// Assign a random color by picking a random hue on the HSV wheel
		let hue = rng.random_range(0f32..360.);
		let (r, g, b) = hsvToRgb(hue, 0.85, 1.);
		let color = cgv::RGBA::from_rgba_premultiplied(r, g, b, 1.);

		out.push(DataPoint { pos, radius, tangent, radDeriv: 0., normal, color });
	}
}
