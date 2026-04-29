

//////
//
// Module definitions
//

/// Module implementing runtime-wrappers for compile-time guarantees about presence of data attributes.
mod guarantees;
pub use guarantees::*; // re-export all public facilities (mainly the guarantee wrapper and combination aliases).



//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Local imports
use crate::{self as cgv, *};



//////
//
// Traits
//

/// Trait of a collection of renderable data, ready for being turned into
/// [GPU-side render data](renderer::data::gpu::Data) for consumption by a [`Renderer`].
pub trait Data
{
	/// The iterator type for iterating positions in the data.
	type PosIterator<'data>: Iterator<Item=&'data glm::Vec3> where Self: 'data;

	/// Return the number of elements in the underlying data series.
	fn num (&self) -> u32;

	/// Iterate over the positions.
	fn positions (&self) -> Self::PosIterator<'_>;

	/// Reference a single position at the given index.
	fn pos (&self, index: u32) -> &glm::Vec3;

	/// Return the preferred [topology](wgpu::PrimitiveTopology) of the data. Some renderers, like
	/// [`renderer::Spheres`], will completely ignore this, while others like [`renderer::Mesh`] will require specific
	/// topologies.
	fn topology (&self) -> wgpu::PrimitiveTopology;
}

/// Marker trait for [`renderer::Data`] indicating that the data attributes are stored in an interleaved fashion (aka.
/// "array of structs").
pub trait Interleaved: Data {}

/// Marker trait for [`renderer::Data`] indicating that the data attributes are stored in a non-interleaved fashion
/// (aka. "struct of arrays").
pub trait NonInterleaved: Data {}

/// Trait indicating that the render data contains index information; that is, indices into the data that connect
/// individual data points to form complex primitives, like lines/line strips, triangles/triangle strips, etc.
///
/// **TODO: Add associated type of the actual index (typically u32 as hardcoded right now)**
pub trait Indexed: Data
{
	/// The iterator type for iterating indices in the data. The lifetime parameter `'data` ensures that implementations
	/// can use borrowing iterators.
	type IndexIterator<'data>: Iterator<Item=&'data u32> where Self: 'data;

	/// Return the number of indices over the underlying data series.
	fn numIndices (&self) -> u32;

	/// Iterate over the indices.
	fn indices (&self) -> Self::IndexIterator<'_>;

	/// Reference a single data index or a slice of data indices.
	fn index (&self, index: u32) -> u32;
}

///
pub trait CanHaveNormals: Data
{
	/// The iterator type for iterating normals in the data. The lifetime parameter `'data` ensures that implementations
	/// can use borrowing iterators.
	type NormalIterator<'data>: Iterator<Item=&'data glm::Vec3> where Self: 'data;

	/// Indicate whether normals are available in the data.
	fn hasNormals (&self) -> bool;

	/// Iterate over the normals.
	///
	/// # Returns
	///
	/// An [`Iterator`] over the normals in the data.
	///
	/// # Panics
	///
	/// If this method is called even though no normals are available (to be checked up-front via
	/// [`hasNormals`](Self::hasNormals).
	fn normals (&self) -> Self::NormalIterator<'_>;

	/// Reference a single position at the given index.
	///
	/// # Arguments
	///
	/// * `index` – The index of the normal to reference.
	///
	/// # Returns
	///
	/// A reference to the normal at the given `index`.
	///
	/// # Panics
	///
	/// If this method is called even though no normals are available (to be checked up-front via
	/// [`hasNormals`](Self::hasNormals), or if `index` was out-of-bounds.
	fn normal (&self, index: u32) -> &glm::Vec3;
}

///
pub trait HasNormals: CanHaveNormals {}

///
pub trait CanHaveTangents: Data
{
	/// The iterator type for iterating tangents in the data. The lifetime parameter `'data` ensures that implementations
	/// can use borrowing iterators.
	type TangentIterator<'data>: Iterator<Item=&'data glm::Vec3> where Self: 'data;

	/// Indicate whether tangents are available in the data.
	fn hasTangents (&self) -> bool;

	/// Iterate over the tangents.
	///
	/// # Returns
	///
	/// An [`Iterator`] over the tangents in the data.
	///
	/// # Panics
	///
	/// If this method is called even though no tangents are available (to be checked up-front via
	/// [`hasTangents`](Self::hasTangents).
	fn tangents (&self) -> Self::TangentIterator<'_>;

	/// Reference a single tangent at the given index.
	///
	/// # Arguments
	///
	/// * `index` – The index of the tangent to reference.
	///
	/// # Returns
	///
	/// A reference to the tangent at the given `index`.
	///
	/// # Panics
	///
	/// If this method is called even though no tangents are available (to be checked up-front via
	/// [`hasTangents`](Self::hasTangents), or if `index` was out-of-bounds.
	fn tangent (&self, index: u32) -> &glm::Vec3;
}

///
pub trait HasTangents: CanHaveTangents {}

///
pub trait CanHaveRadii: Data
{
	/// The iterator type for iterating radii in the data. The lifetime parameter `'data` ensures that implementations
	/// can use borrowing iterators.
	type RadiusIterator<'data>: Iterator<Item=&'data f32> where Self: 'data;

	/// Indicate whether radii are available in the data.
	fn hasRadii (&self) -> bool;

	/// Iterate over the radii.
	///
	/// # Returns
	///
	/// An [`Iterator`] over the radii in the data.
	///
	/// # Panics
	///
	/// If this method is called even though no radii are available (to be checked up-front via
	/// [`hasRadii`](Self::hasRadii).
	fn radii (&self) -> Self::RadiusIterator<'_>;

	/// Return the radius at the given index.
	///
	/// # Arguments
	///
	/// * `index` – The index of the radius to return.
	///
	/// # Returns
	///
	/// The radius at the given `index`.
	///
	/// # Panics
	///
	/// If this method is called even though no radii are available (to be checked up-front via
	/// [`hasRadii`](Self::hasRadii), or if `index` was out-of-bounds.
	fn radius (&self, index: u32) -> f32;
}

///
pub trait HasRadii: CanHaveRadii {}

///
pub trait CanHaveRadiusDerivs: CanHaveRadii
{
	/// Indicate whether radius derivatives are available in the data.
	fn hasRadiusDerivs (&self) -> bool;

	/// Iterate over the radius derivatives.
	///
	/// # Returns
	///
	/// An [`Iterator`] over the radius derivatives in the data.
	///
	/// # Panics
	///
	/// If this method is called even though no radius derivatives are available (to be checked up-front via
	/// [`hasRadiusDerivs`](Self::hasRadiusDerivs).
	fn radiusDerivs (&self) -> Self::RadiusIterator<'_>;

	/// Return the radius derivative at the given index.
	///
	/// # Arguments
	///
	/// * `index` – The index of the radius derivative to return.
	///
	/// # Returns
	///
	/// The radius derivative at the given `index`.
	///
	/// # Panics
	///
	/// If this method is called even though no radius derivatives are available (to be checked up-front via
	/// [`hasRadiusDerivs`](Self::hasRadiusDerivs), or if `index` was out-of-bounds.
	fn radiusDeriv (&self, index: u32) -> f32;
}

///
pub trait HasRadiusDerivs: CanHaveRadiusDerivs+HasRadii {}

///
pub trait CanHaveOrientations: Data
{
	/// The iterator type for iterating orientations in the data. The lifetime parameter `'data` ensures that
	/// implementations can use borrowing iterators.
	type OrientationIterator<'data>: Iterator<Item=&'data glm::Quat> where Self: 'data;

	/// Indicate whether orientations are available in the data.
	fn hasOrientations (&self) -> bool;

	/// Iterate over the orientations.
	///
	/// # Returns
	///
	/// An [`Iterator`] over the orientations in the data.
	///
	/// # Panics
	///
	/// If this method is called even though no orientations are available (to be checked up-front via
	/// [`hasOrientations`](Self::hasOrientations).
	fn orientations (&self) -> Self::OrientationIterator<'_>;

	/// Reference a single orientation at the given index.
	///
	/// # Arguments
	///
	/// * `index` – The index of the orientation to reference.
	///
	/// # Returns
	///
	/// A reference to the orientation at the given `index`.
	///
	/// # Panics
	///
	/// If this method is called even though no orientations are available (to be checked up-front via
	/// [`hasOrientations`](Self::hasOrientations), or if `index` was out-of-bounds.
	fn orientation (&self, index: u32) -> &glm::Quat;
}

///
pub trait HasOrientations: CanHaveOrientations {}

///
pub trait CanHaveScalings: Data
{
	/// The iterator type for iterating scaling vectors in the data. The lifetime parameter `'data` ensures that
	/// implementations can use borrowing iterators.
	type ScaleIterator<'data>: Iterator<Item=&'data glm::Vec3> where Self: 'data;

	/// Indicate whether scaling vectors are available in the data.
	fn hasScalings (&self) -> bool;

	/// Iterate over the scaling vectors.
	///
	/// # Returns
	///
	/// An [`Iterator`] over the scaling vectors in the data.
	///
	/// # Panics
	///
	/// If this method is called even though no scaling vectors are available (to be checked up-front via
	/// [`hasTangents`](Self::hasTangents).
	fn scalings (&self) -> Self::ScaleIterator<'_>;

	/// Reference a single scaling vector at the given index.
	///
	/// # Arguments
	///
	/// * `index` – The index of the scaling vector to reference.
	///
	/// # Returns
	///
	/// A reference to the scaling vector at the given `index`.
	///
	/// # Panics
	///
	/// If this method is called even though no scaling vectors are available (to be checked up-front via
	/// [`hasScalings`](Self::hasScalings), or if `index` was out-of-bounds.
	fn scaling (&self, index: u32) -> &glm::Vec3;
}

///
pub trait HasScalings: CanHaveScalings {}

///
pub trait CanHaveColors: Data
{
	/// The iterator type for iterating colors in the data. The lifetime parameter `'data` ensures that implementations
	/// can use borrowing iterators.
	type ColorIterator<'data>: Iterator<Item=&'data cgv::RGBA> where Self: 'data;

	/// Indicate whether colors are available in the data.
	fn hasColors (&self) -> bool;

	/// Iterate over the colors.
	///
	/// # Returns
	///
	/// An [`Iterator`] over the colors in the data.
	///
	/// # Panics
	///
	/// If this method is called even though no colors are available (to be checked up-front via
	/// [`hasColors`](Self::hasColors).
	fn colors (&self) -> Self::ColorIterator<'_>;

	/// Reference a single color at the given index.
	///
	/// # Arguments
	///
	/// * `index` – The index of the color to reference.
	///
	/// # Returns
	///
	/// A reference to the color at the given `index`.
	///
	/// # Panics
	///
	/// If this method is called even though no colors are available (to be checked up-front via
	/// [`hasColors`](Self::hasColors), or if `index` was out-of-bounds.
	fn color (&self, index: u32) -> &cgv::RGBA;
}

///
pub trait HasColors: CanHaveColors {}
