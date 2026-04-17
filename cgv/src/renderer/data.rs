
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

/// Trait of a collection of renderable data, ready for consumption by a [`Renderer`].
pub trait Data
{
	/// Return the number of elements in the underlying data series.
	fn num (&self) -> u32;
}

/// Marker trait for [`renderer::Data`] indicating that the data attributes are stored in an interleaved fashion (aka.
/// "array of structs").
pub trait Interleaved: Data {}

/// Marker trait for [`renderer::Data`] indicating that the data attributes are stored in a non-interleaved fashion
/// (aka. "struct of arrays").
pub trait NonInterleaved: Data {}

/// Trait indicating that the render data contains index information; that is, indices into the data that connect
/// individual data points to form complex primitives, like lines/line strips, triangles/triangle strips, etc.
pub trait Indexed: Data
{
	/// The iterator type for iterating indices in the data.
	type IndexIterator: Iterator<Item=u32>;

	/// Return the number of indices over the underlying data series.
	fn numIndices (&self) -> u32;

	/// Iterate over the indices.
	fn indices (&self) -> Self::IndexIterator;

	/// Reference a single data index or a slice of data indices.
	fn index (&self, index: u32) -> u32;
}

///
pub trait HasPositions: Data {
	/// The iterator type for iterating positions in the data.
	type PosIterator: Iterator<Item=glm::Vec4>;

	/// Iterate over the positions.
	fn positions (&self) -> Self::PosIterator;

	/// Reference a single position at the given index.
	fn pos (&self, index: u32) -> &glm::Vec4;
}

///
pub trait HasNormals: Data {
	/// The iterator type for iterating normals in the data.
	type NormalIterator: Iterator<Item=glm::Vec4>;

	/// Iterate over the normals.
	fn normals (&self) -> Self::NormalIterator;

	/// Reference a single position at the given index.
	fn normal (&self, index: u32) -> &glm::Vec4;
}

///
pub trait HasTangents: Data {
	/// The iterator type for iterating tangents in the data.
	type TangentIterator: Iterator<Item=glm::Vec4>;

	/// Iterate over the tangents.
	fn tangents (&self) -> Self::TangentIterator;

	/// Reference a single tangent at the given index.
	fn tangent (&self, index: u32) -> &glm::Vec4;
}

///
pub trait HasRadii: Data {
	/// The iterator type for iterating radii in the data.
	type RadiusIterator: Iterator<Item=f32>;

	/// Iterate over the radii.
	fn radii (&self) -> Self::RadiusIterator;

	/// Reference a single radius at the given index.
	fn radius (&self, index: u32) -> f32;
}

///
pub trait HasOrientation: Data {
	/// The iterator type for iterating orientations in the data.
	type OrientationIterator: Iterator<Item=glm::Quat>;

	/// Iterate over the orientations.
	fn orientations (&self) -> Self::OrientationIterator;

	/// Reference a single orientation at the given index.
	fn orientation (&self, index: u32) -> &glm::Quat;
}

///
pub trait HasScale: Data {
	/// The iterator type for iterating scaling values in the data.
	type ScaleIterator: Iterator<Item=glm::Vec3>;

	/// Iterate over the scaling values.
	fn scales (&self) -> Self::ScaleIterator;

	/// Reference a single scaling vector at the given index.
	fn scale (&self, index: u32) -> &glm::Vec3;
}

///
pub trait HasColors: Data {
	/// The iterator type for iterating colors in the data.
	type ColorIterator: Iterator<Item=cgv::RGBA>;

	/// Iterate over the colors.
	fn colors (&self) -> Self::ColorIterator;

	/// Reference a single color at the given index.
	fn color (&self, index: u32) -> &cgv::RGBA;
}
