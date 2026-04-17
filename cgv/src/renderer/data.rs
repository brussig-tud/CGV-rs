
//////
//
// Imports
//

// Standard library
use std::{ops::Index, slice::SliceIndex};

// Local imports
use crate::*;



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

/// Trait indicating that the render data contains topological information; that is, indices into the data that connect
/// individual data points to form complex primitives, like lines/line strips, triangles/triangle strips, etc.
pub trait Topological: Data
{
	/// The iterator type for iterating indices in the data.
	type IndexIterator: Iterator<Item=u32>;

	/// Return the number of indices over the underlying data series.
	fn numIndices (&self) -> u32;

	/// Iterate over the indices.
	fn indices (&self) -> Self::IndexIterator;

	/// Reference a single data index or a slice of data indices.
	fn index<I: SliceIndex<[u32]>> (&self, index: I) -> &I::Output;
}

///
pub trait HasPositions: Data {
	/// The iterator type for iterating positions in the data.
	type PosIterator: Iterator<Item=glm::Vec4>;

	/// Iterate over the positions.
	fn positions (&self) -> Self::PosIterator;

	/// Reference a single position a slice of positions.
	fn pos<I: SliceIndex<[glm::Vec4]>> (&self, index: I) -> &I::Output;
}

///
pub trait HasNormals: Data {
	/// The iterator type for iterating normals in the data.
	type NormalIterator: Iterator<Item=glm::Vec4>;

	/// Iterate over the normals.
	fn normals (&self) -> Self::NormalIterator;

	/// Reference a single normal a slice of normals.
	fn normal<I: SliceIndex<[glm::Vec4]>> (&self, index: I) -> &I::Output;
}

///
pub trait HasTangents: Data {
	/// The iterator type for iterating tangents in the data.
	type TangentIterator: Iterator<Item=glm::Vec4>;

	/// Iterate over the tangents.
	fn tangents (&self) -> Self::TangentIterator;

	/// Reference a single tangent a slice of tangents.
	fn tangent<I: SliceIndex<[glm::Vec4]>> (&self, index: I) -> &I::Output;
}

///
pub trait HasRadii: Data {
	/// The iterator type for iterating radii in the data.
	type RadiusIterator: Iterator<Item=f32>;

	/// Iterate over the radii.
	fn radii (&self) -> Self::RadiusIterator;

	/// Reference a single radius a slice of radii.
	fn radius<I: SliceIndex<[f32]>> (&self, index: I) -> &I::Output;
}

///
pub trait HasOrientation: Data {
	/// The iterator type for iterating orientations in the data.
	type OrientationIterator: Iterator<Item=glm::Quat>;

	/// Iterate over the orientations.
	fn orientations (&self) -> Self::OrientationIterator;

	/// Reference a single orientation a slice of orientations.
	fn orientation<I: SliceIndex<[glm::Quat]>> (&self, index: I) -> &I::Output;
}

///
pub trait HasScale: Data {
	/// The iterator type for iterating scaling values in the data.
	type ScaleIterator: Iterator<Item=glm::Vec3>;

	/// Iterate over the scaling values.
	fn scales (&self) -> Self::ScaleIterator;

	/// Reference a single scaling value a slice of scaling values.
	fn scale<I: SliceIndex<[glm::Vec3]>> (&self, index: I) -> &I::Output;
}

///
pub trait HasColors: Data {
	/// The iterator type for iterating colors in the data.
	type ColorIterator: Iterator<Item=egui::ecolor::Rgba>;

	/// Iterate over the colors.
	fn colors (&self) -> Self::ColorIterator;

	/// Reference a single color a slice of colors.
	fn color<I: SliceIndex<[egui::ecolor::Rgba]>> (&self, index: I) -> &I::Output;
}



//////
//
// Structs
//

///

/// An interface for random-access to values of a single-attribute data series.
pub trait AttributeAccessor<'accessor, T>
{
	/// Return the number of elements in the underlying series.
	fn num (&self) -> usize;

	/// Obtain a reference to the value at the given index `idx`.
	///
	/// # Panics
	///
	/// Accessing with an out-of-bounds `idx` is undefined behavior. This function may panic (depending on the
	/// implementation) in this case.
	fn at (&self, idx: usize) -> &'accessor T;

	/// Obtain a mutable reference to the value at the given index `idx`.
	///
	/// # Panics
	///
	/// Accessing with an out-of-bounds `idx` is undefined behavior. This function may panic (depending on the
	/// implementation) in this case.
	fn at_mut (&mut self, idx: usize) -> &'accessor mut T;
}

/// The common interface for augmented position data containers.
pub trait AugmentedPositionData<'container>
{
	/// Report whether the data is internally stored in an interleaved fashion (*"array of structs"*) or not (*"struct
	/// of arrays"*).
	fn isInterleaved (&self) -> bool;

	/// Return an accessor for the 3D *positions* of the data points.
	fn positions (&self) -> &'container dyn AttributeAccessor<'container, glm::Vec4>;

	/// Temporarily gain mutable access to the 3D *positions* of the data points.
	fn mutatePositions<R, Action> (&self, action: Action) -> R
		where Action: FnOnce(&mut dyn AttributeAccessor<'_, glm::Vec4>)->R;
}

/// The common interface for all renderers that visualize augmented position data with any kind of 3D primitive.
pub trait PrimitiveRenderer
{
	fn setData<'data> (&mut self, data: impl AugmentedPositionData<'data>);
}
