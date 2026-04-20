
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
	/// The iterator type for iterating positions in the data.
	type PosIterator: Iterator<Item=glm::Vec3>;

	/// Return the number of elements in the underlying data series.
	fn num (&self) -> u32;

	/// Iterate over the positions.
	fn positions (&self) -> Self::PosIterator;

	/// Reference a single position at the given index.
	fn pos (&self, index: u32) -> &glm::Vec3;
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
pub trait CanHaveNormals: Data
{
	/// The iterator type for iterating normals in the data.
	type NormalIterator: Iterator<Item=glm::Vec3>;

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
	fn normals (&self) -> Self::NormalIterator;

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
pub trait HasNormals: CanHaveNormals {
	/// We are guaranteed to have normals.
	fn hasNormals (&self) -> bool { true }
}

///
pub trait CanHaveTangents: Data
{
	/// The iterator type for iterating tangents in the data.
	type TangentIterator: Iterator<Item=glm::Vec3>;

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
	fn tangents (&self) -> Self::TangentIterator;

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
pub trait HasTangents: CanHaveTangents {
	/// We are guaranteed to have tangents.
	fn hasTangents (&self) -> bool { true }
}

///
pub trait CanHaveRadii: Data
{
	/// The iterator type for iterating radii in the data.
	type RadiusIterator: Iterator<Item=f32>;

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
	fn radii (&self) -> Self::RadiusIterator;

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
pub trait HasRadii: CanHaveRadii {
	/// We are guaranteed to have radii.
	fn hasRadii (&self) -> bool { true }
}

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
	fn radiusDerivs (&self) -> Self::RadiusIterator;

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
pub trait HasRadiusDerivs: CanHaveRadiusDerivs {
	/// We are guaranteed to have radius derivatives.
	fn hasRadiusDerivs (&self) -> bool { true }
}

///
pub trait CanHaveOrientations: Data
{
	/// The iterator type for iterating orientations in the data.
	type OrientationIterator: Iterator<Item=glm::Quat>;

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
	fn orientations (&self) -> Self::OrientationIterator;

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
pub trait HasOrientations: CanHaveOrientations {
	/// We are guaranteed to have orientations.
	fn hasOrientations (&self) -> bool { true }
}

///
pub trait CanHaveScalings: Data
{
	/// The iterator type for iterating scaling vectors in the data.
	type ScaleIterator: Iterator<Item=glm::Vec3>;


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
	fn scalings (&self) -> Self::ScaleIterator;

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
pub trait HasScalings: CanHaveScalings {
	/// We are guaranteed to have scaling vectors.
	fn hasScalings (&self) -> bool { true }
}

///
pub trait CanHaveColors: Data
{
	/// The iterator type for iterating colors in the data.
	type ColorIterator: Iterator<Item=cgv::RGBA>;

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
	fn colors (&self) -> Self::ColorIterator;

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
pub trait HasColors: CanHaveColors {
	/// We are guaranteed to have colors.
	fn hasColors (&self) -> bool { true }
}



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
impl<DataWithNormals: CanHaveNormals> CanHaveNormals for GuaranteeNormals<DataWithNormals> {
	type NormalIterator = DataWithNormals::NormalIterator;
	#[inline(always)]
	fn hasNormals (&self) -> bool { true }
	#[inline(always)]
	fn normals (&self) -> Self::NormalIterator { self.0.normals() }
	#[inline(always)]
	fn normal (&self, index: u32) -> &glm::Vec3 { self.0.normal(index) }
}
impl<DataWithNormals: CanHaveNormals> HasNormals for GuaranteeNormals<DataWithNormals> {}
impl<T: CanHaveNormals+CanHaveTangents> CanHaveTangents for GuaranteeNormals<T> {
	type TangentIterator = T::TangentIterator;
	#[inline(always)]
	fn hasTangents (&self) -> bool { self.0.hasTangents() }
	#[inline(always)]
	fn tangents (&self) -> Self::TangentIterator { self.0.tangents() }
	#[inline(always)]
	fn tangent (&self, index: u32) -> &glm::Vec3 { self.0.tangent(index) }
}
impl<T: CanHaveNormals+HasTangents> HasTangents for GuaranteeNormals<T> {}
impl<T: CanHaveNormals+CanHaveRadii> CanHaveRadii for GuaranteeNormals<T> {
	type RadiusIterator = T::RadiusIterator;
	#[inline(always)]
	fn hasRadii (&self) -> bool { self.0.hasRadii() }
	#[inline(always)]
	fn radii (&self) -> Self::RadiusIterator { self.0.radii() }
	#[inline(always)]
	fn radius (&self, index: u32) -> f32 { self.0.radius(index) }
}
impl<T: CanHaveNormals+HasRadii> HasRadii for GuaranteeNormals<T> {}
impl<T: CanHaveNormals+CanHaveRadiusDerivs> CanHaveRadiusDerivs for GuaranteeNormals<T> {
	#[inline(always)]
	fn hasRadiusDerivs (&self) -> bool { self.0.hasRadiusDerivs() }
	#[inline(always)]
	fn radiusDerivs (&self) -> Self::RadiusIterator { self.0.radiusDerivs() }
	#[inline(always)]
	fn radiusDeriv (&self, index: u32) -> f32 { self.0.radiusDeriv(index) }
}
impl<T: CanHaveNormals+HasRadiusDerivs> HasRadiusDerivs for GuaranteeNormals<T> {}
impl<T: CanHaveNormals+CanHaveOrientations> CanHaveOrientations for GuaranteeNormals<T> {
	type OrientationIterator = T::OrientationIterator;
	#[inline(always)]
	fn hasOrientations (&self) -> bool { self.0.hasOrientations() }
	#[inline(always)]
	fn orientations (&self) -> Self::OrientationIterator { self.0.orientations() }
	#[inline(always)]
	fn orientation (&self, index: u32) -> &glm::Quat { self.0.orientation(index) }
}
impl<T: CanHaveNormals+HasOrientations> HasOrientations for GuaranteeNormals<T> {}
impl<T: CanHaveNormals+CanHaveScalings> CanHaveScalings for GuaranteeNormals<T> {
	type ScaleIterator = T::ScaleIterator;
	#[inline(always)]
	fn hasScalings (&self) -> bool { self.0.hasScalings() }
	#[inline(always)]
	fn scalings (&self) -> Self::ScaleIterator { self.0.scalings() }
	#[inline(always)]
	fn scaling (&self, index: u32) -> &glm::Vec3 { self.0.scaling(index) }
}
impl<T: CanHaveNormals+HasScalings> HasScalings for GuaranteeNormals<T> {}
impl<T: CanHaveNormals+CanHaveColors> CanHaveColors for GuaranteeNormals<T> {
	type ColorIterator = T::ColorIterator;
	#[inline(always)]
	fn hasColors (&self) -> bool { self.0.hasColors() }
	#[inline(always)]
	fn colors (&self) -> Self::ColorIterator { self.0.colors() }
	#[inline(always)]
	fn color (&self, index: u32) -> &cgv::RGBA { self.0.color(index) }
}
impl<T: CanHaveNormals+HasColors> HasColors for GuaranteeNormals<T> {}
impl<DataWithNormals: CanHaveNormals> std::ops::Deref for GuaranteeNormals<DataWithNormals> {
	type Target = DataWithNormals;

	#[inline(always)]
	fn deref (&self) -> &Self::Target { &self.0 }
}
impl<DataWithNormals: CanHaveNormals> std::ops::DerefMut for GuaranteeNormals<DataWithNormals> {
	#[inline(always)]
	fn deref_mut (&mut self) -> &mut Self::Target { &mut self.0 }
}

/*/// Non-indexed, interleaved test [render data](renderer::data::Data) with positions and normals.
struct GpuDataTanCol {
	data: Vec<(/* positions: */glm::Vec4, /* normals: */glm::Vec4, /* radius */f32, /* color */cgv::RGBA)>
}
impl renderer::Data for NonIndexedInterleavedPosNormalRadiusColor {
	fn num (&self) -> u32 {
		self.data.len() as u32
	}
}
impl Interleaved for NonIndexedInterleavedPosNormalRadiusColor {}
impl HasPositions for NonIndexedInterleavedPosNormalRadiusColor {
	type PosIterator = util::notsafe::StridedIter<glm::Vec4>;

	fn positions (&self) -> Self::PosIterator {
		unsafe {
			// SAFETY: We store a `Vec` of structs, and `Vec` can be trusted to return the correct length, so alignment
			// and validity of the fields the iterator accesses is guaranteed.
			util::notsafe::stridedIter!(self.data, 0, glm::Vec4)
		}
	}

	fn pos (&self, index: u32) -> &glm::Vec4 {
		&self.data[index as usize].0
	}
}
impl HasNormals for NonIndexedInterleavedPosNormalRadiusColor {
	type NormalIterator = util::notsafe::StridedIter<glm::Vec4>;

	fn normals (&self) -> Self::NormalIterator {
		unsafe {
			// SAFETY: We store a `Vec` of structs, and `Vec` can be trusted to return the correct length, so alignment
			// and validity of the fields the iterator accesses is guaranteed.
			util::notsafe::stridedIter!(self.data, 1, glm::Vec4)
		}
	}

	fn normal (&self, index: u32) -> &glm::Vec4 {
		&self.data[index as usize].1
	}
}
impl HasRadii for NonIndexedInterleavedPosNormalRadiusColor {
	type RadiusIterator = util::notsafe::StridedIter<f32>;

	fn radii (&self) -> Self::RadiusIterator {
		unsafe {
			// SAFETY: We store a `Vec` of structs, and `Vec` can be trusted to return the correct length, so alignment
			// and validity of the fields the iterator accesses is guaranteed.
			util::notsafe::stridedIter!(self.data, 2, f32)
		}
	}

	fn radius (&self, index: u32) -> f32 {
		self.data[index as usize].2
	}
}
impl HasColors for NonIndexedInterleavedPosNormalRadiusColor {
	type ColorIterator = util::notsafe::StridedIter<cgv::RGBA>;

	fn colors (&self) -> Self::ColorIterator {
		unsafe {
			// SAFETY: We store a `Vec` of structs, and `Vec` can be trusted to return the correct length, so alignment
			// and validity of the fields the iterator accesses is guaranteed.
			util::notsafe::stridedIter!(self.data, 3, cgv::RGBA)
		}
	}

	fn color (&self, index: u32) -> &cgv::RGBA {
		&self.data[index as usize].3
	}
}*/
