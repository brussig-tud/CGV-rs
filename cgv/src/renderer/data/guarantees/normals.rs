
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

/// Wrapper to turn runtime presence of normals into a compile-time guarantee. Panics during construction if the
/// wrappee does not actually have normals.
pub struct GuaranteeNormals<DataWithNormals: CanHaveNormals>(DataWithNormals);
impl<DataWithNormals: CanHaveNormals> GuaranteeNormals<DataWithNormals>
{
	/// Wraps a given instance of render data that [`CanHaveNormals`] with a compile-time guarantee that normals are
	/// present.
	///
	/// # Arguments
	///
	/// * `data` – The [`renderer::Data`] to guarantee normals for.
	///
	/// # Returns
	///
	/// A new `GuaranteeNormals` wrapping the provided `data`.
	///
	/// # Panics
	///
	/// If the provided `data` does not contain normals.
	#[inline(always)]
	pub fn new (data: DataWithNormals) -> Self {
		assert!(data.hasNormals(), "to-be-wrapped data must contain normals!");
		GuaranteeNormals(data)
	}
}
impl<DataWithNormals: CanHaveNormals> Data for GuaranteeNormals<DataWithNormals>
{
	type PosIterator = DataWithNormals::PosIterator;

	#[inline(always)]
	fn num (&self) -> u32 { self.0.num() }
	#[inline(always)]
	fn positions (&self) -> Self::PosIterator { self.0.positions() }
	#[inline(always)]
	fn pos (&self, index: u32) -> &glm::Vec3 { self.0.pos(index) }
}
impl<T: CanHaveNormals+Indexed> Indexed for GuaranteeNormals<T>
{
	type IndexIterator = T::IndexIterator;

	#[inline(always)]
	fn numIndices (&self) -> u32 { self.0.numIndices() }
	#[inline(always)]
	fn indices (&self) -> Self::IndexIterator { self.0.indices() }
	#[inline(always)]
	fn index (&self, index: u32) -> u32 { self.0.index(index) }
}
impl<T: CanHaveNormals+Interleaved> Interleaved for GuaranteeNormals<T> {}
impl<T: CanHaveNormals+NonInterleaved> NonInterleaved for GuaranteeNormals<T> {}

impl<DataWithNormals: CanHaveNormals> CanHaveNormals for GuaranteeNormals<DataWithNormals>
{
	type NormalIterator = DataWithNormals::NormalIterator;

	#[inline(always)]
	fn hasNormals (&self) -> bool { true }
	#[inline(always)]
	fn normals (&self) -> Self::NormalIterator { self.0.normals() }
	#[inline(always)]
	fn normal (&self, index: u32) -> &glm::Vec3 { self.0.normal(index) }
}
impl<DataWithNormals: CanHaveNormals> HasNormals for GuaranteeNormals<DataWithNormals> {}

impl<T: CanHaveNormals+CanHaveTangents> CanHaveTangents for GuaranteeNormals<T>
{
	type TangentIterator = T::TangentIterator;

	#[inline(always)]
	fn hasTangents (&self) -> bool { self.0.hasTangents() }
	#[inline(always)]
	fn tangents (&self) -> Self::TangentIterator { self.0.tangents() }
	#[inline(always)]
	fn tangent (&self, index: u32) -> &glm::Vec3 { self.0.tangent(index) }
}
impl<T: CanHaveNormals+HasTangents> HasTangents for GuaranteeNormals<T> {}

impl<T: CanHaveNormals+CanHaveRadii> CanHaveRadii for GuaranteeNormals<T>
{
	type RadiusIterator = T::RadiusIterator;

	#[inline(always)]
	fn hasRadii (&self) -> bool { self.0.hasRadii() }
	#[inline(always)]
	fn radii (&self) -> Self::RadiusIterator { self.0.radii() }
	#[inline(always)]
	fn radius (&self, index: u32) -> f32 { self.0.radius(index) }
}
impl<T: CanHaveNormals+HasRadii> HasRadii for GuaranteeNormals<T> {}

impl<T: CanHaveNormals+CanHaveRadiusDerivs> CanHaveRadiusDerivs for GuaranteeNormals<T>
{
	#[inline(always)]
	fn hasRadiusDerivs (&self) -> bool { self.0.hasRadiusDerivs() }
	#[inline(always)]
	fn radiusDerivs (&self) -> Self::RadiusIterator { self.0.radiusDerivs() }
	#[inline(always)]
	fn radiusDeriv (&self, index: u32) -> f32 { self.0.radiusDeriv(index) }
}
impl<T: CanHaveNormals+HasRadiusDerivs> HasRadiusDerivs for GuaranteeNormals<T> {}

impl<T: CanHaveNormals+CanHaveOrientations> CanHaveOrientations for GuaranteeNormals<T>
{
	type OrientationIterator = T::OrientationIterator;

	#[inline(always)]
	fn hasOrientations (&self) -> bool { self.0.hasOrientations() }
	#[inline(always)]
	fn orientations (&self) -> Self::OrientationIterator { self.0.orientations() }
	#[inline(always)]
	fn orientation (&self, index: u32) -> &glm::Quat { self.0.orientation(index) }
}
impl<T: CanHaveNormals+HasOrientations> HasOrientations for GuaranteeNormals<T> {}

impl<T: CanHaveNormals+CanHaveScalings> CanHaveScalings for GuaranteeNormals<T>
{
	type ScaleIterator = T::ScaleIterator;

	#[inline(always)]
	fn hasScalings (&self) -> bool { self.0.hasScalings() }
	#[inline(always)]
	fn scalings (&self) -> Self::ScaleIterator { self.0.scalings() }
	#[inline(always)]
	fn scaling (&self, index: u32) -> &glm::Vec3 { self.0.scaling(index) }
}
impl<T: CanHaveNormals+HasScalings> HasScalings for GuaranteeNormals<T> {}

impl<T: CanHaveNormals+CanHaveColors> CanHaveColors for GuaranteeNormals<T>
{
	type ColorIterator = T::ColorIterator;

	#[inline(always)]
	fn hasColors (&self) -> bool { self.0.hasColors() }
	#[inline(always)]
	fn colors (&self) -> Self::ColorIterator { self.0.colors() }
	#[inline(always)]
	fn color (&self, index: u32) -> &cgv::RGBA { self.0.color(index) }
}
impl<T: CanHaveNormals+HasColors> HasColors for GuaranteeNormals<T> {}

/// When required we `deref` to the wrapped type.
impl<DataWithNormals: CanHaveNormals> Deref for GuaranteeNormals<DataWithNormals> {
	/// Deref to our wrapped type.
	type Target = DataWithNormals;

	#[inline(always)]
	fn deref (&self) -> &Self::Target { &self.0 }
}
impl<DataWithNormals: CanHaveNormals> DerefMut for GuaranteeNormals<DataWithNormals> {
	#[inline(always)]
	fn deref_mut (&mut self) -> &mut Self::Target { &mut self.0 }
}
