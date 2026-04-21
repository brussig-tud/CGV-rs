
//////
//
// Imports
//

// Standard library
use std::ops::{Deref, DerefMut};

// GLM linear algebra library
use glm;

// Local imports
use crate::{self as cgv, *, renderer::{*, data::*}};



//////
//
// Structs
//

/// Wrapper to turn runtime presence of tangents into a compile-time guarantee. Panics during construction if the
/// wrappee does not actually have tangents.
pub struct GuaranteeTangents<DataWithTangents: CanHaveTangents>(DataWithTangents);
impl<DataWithTangents: CanHaveTangents> GuaranteeTangents<DataWithTangents>
{
	/// Wraps a given instance of render data that [`CanHaveTangents`] with a compile-time guarantee that tangents are
	/// present.
	///
	/// # Arguments
	///
	/// * `data` – The [`renderer::Data`] to guarantee tangents for.
	///
	/// # Returns
	///
	/// A new `GuaranteeTangents` wrapping the provided `data`.
	///
	/// # Panics
	///
	/// If the provided `data` does not contain tangents.
	#[inline(always)]
	pub fn new (data: DataWithTangents) -> Self {
		assert!(data.hasTangents(), "to-be-wrapped data must contain tangents!");
		GuaranteeTangents(data)
	}
}
impl<DataWithTangents: CanHaveTangents> Data for GuaranteeTangents<DataWithTangents>
{
	type PosIterator = DataWithTangents::PosIterator;

	#[inline(always)]
	fn num (&self) -> u32 { self.0.num() }
	#[inline(always)]
	fn positions (&self) -> Self::PosIterator { self.0.positions() }
	#[inline(always)]
	fn pos (&self, index: u32) -> &glm::Vec3 { self.0.pos(index) }
}
impl<T: CanHaveTangents+Indexed> Indexed for GuaranteeTangents<T>
{
	type IndexIterator = T::IndexIterator;

	#[inline(always)]
	fn numIndices (&self) -> u32 { self.0.numIndices() }
	#[inline(always)]
	fn indices (&self) -> Self::IndexIterator { self.0.indices() }
	#[inline(always)]
	fn index (&self, index: u32) -> u32 { self.0.index(index) }
}
impl<T: CanHaveTangents+Interleaved> Interleaved for GuaranteeTangents<T> {}
impl<T: CanHaveTangents+NonInterleaved> NonInterleaved for GuaranteeTangents<T> {}

impl<DataWithTangents: CanHaveTangents> CanHaveTangents for GuaranteeTangents<DataWithTangents>
{
	type TangentIterator = DataWithTangents::TangentIterator;

	#[inline(always)]
	fn hasTangents (&self) -> bool { true }
	#[inline(always)]
	fn tangents (&self) -> Self::TangentIterator { self.0.tangents() }
	#[inline(always)]
	fn tangent (&self, index: u32) -> &glm::Vec3 { self.0.tangent(index) }
}
impl<DataWithTangents: CanHaveTangents> HasTangents for GuaranteeTangents<DataWithTangents> {}

impl<T: CanHaveTangents+CanHaveNormals> CanHaveNormals for GuaranteeTangents<T>
{
	type NormalIterator = T::NormalIterator;

	#[inline(always)]
	fn hasNormals (&self) -> bool { self.0.hasNormals() }
	#[inline(always)]
	fn normals (&self) -> Self::NormalIterator { self.0.normals() }
	#[inline(always)]
	fn normal (&self, index: u32) -> &glm::Vec3 { self.0.normal(index) }
}
impl<T: CanHaveTangents+HasNormals> HasNormals for GuaranteeTangents<T> {}

impl<T: CanHaveTangents+CanHaveRadii> CanHaveRadii for GuaranteeTangents<T>
{
	type RadiusIterator = T::RadiusIterator;

	#[inline(always)]
	fn hasRadii (&self) -> bool { self.0.hasRadii() }
	#[inline(always)]
	fn radii (&self) -> Self::RadiusIterator { self.0.radii() }
	#[inline(always)]
	fn radius (&self, index: u32) -> f32 { self.0.radius(index) }
}
impl<T: CanHaveTangents+HasRadii> HasRadii for GuaranteeTangents<T> {}

impl<T: CanHaveTangents+CanHaveRadiusDerivs> CanHaveRadiusDerivs for GuaranteeTangents<T>
{
	#[inline(always)]
	fn hasRadiusDerivs (&self) -> bool { self.0.hasRadiusDerivs() }
	#[inline(always)]
	fn radiusDerivs (&self) -> Self::RadiusIterator { self.0.radiusDerivs() }
	#[inline(always)]
	fn radiusDeriv (&self, index: u32) -> f32 { self.0.radiusDeriv(index) }
}
impl<T: CanHaveTangents+HasRadiusDerivs> HasRadiusDerivs for GuaranteeTangents<T> {}

impl<T: CanHaveTangents+CanHaveOrientations> CanHaveOrientations for GuaranteeTangents<T>
{
	type OrientationIterator = T::OrientationIterator;

	#[inline(always)]
	fn hasOrientations (&self) -> bool { self.0.hasOrientations() }
	#[inline(always)]
	fn orientations (&self) -> Self::OrientationIterator { self.0.orientations() }
	#[inline(always)]
	fn orientation (&self, index: u32) -> &glm::Quat { self.0.orientation(index) }
}
impl<T: CanHaveTangents+HasOrientations> HasOrientations for GuaranteeTangents<T> {}

impl<T: CanHaveTangents+CanHaveScalings> CanHaveScalings for GuaranteeTangents<T>
{
	type ScaleIterator = T::ScaleIterator;

	#[inline(always)]
	fn hasScalings (&self) -> bool { self.0.hasScalings() }
	#[inline(always)]
	fn scalings (&self) -> Self::ScaleIterator { self.0.scalings() }
	#[inline(always)]
	fn scaling (&self, index: u32) -> &glm::Vec3 { self.0.scaling(index) }
}
impl<T: CanHaveTangents+HasScalings> HasScalings for GuaranteeTangents<T> {}

impl<T: CanHaveTangents+CanHaveColors> CanHaveColors for GuaranteeTangents<T>
{
	type ColorIterator = T::ColorIterator;

	#[inline(always)]
	fn hasColors (&self) -> bool { self.0.hasColors() }
	#[inline(always)]
	fn colors (&self) -> Self::ColorIterator { self.0.colors() }
	#[inline(always)]
	fn color (&self, index: u32) -> &cgv::RGBA { self.0.color(index) }
}
impl<T: CanHaveTangents+HasColors> HasColors for GuaranteeTangents<T> {}

/// When required we `deref` to the wrapped type.
impl<DataWithTangents: CanHaveTangents> Deref for GuaranteeTangents<DataWithTangents> {
	/// Deref to our wrapped type.
	type Target = DataWithTangents;

	#[inline(always)]
	fn deref (&self) -> &Self::Target { &self.0 }
}
impl<DataWithTangents: CanHaveTangents> DerefMut for GuaranteeTangents<DataWithTangents> {
	#[inline(always)]
	fn deref_mut (&mut self) -> &mut Self::Target { &mut self.0 }
}
