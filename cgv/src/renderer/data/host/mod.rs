
//////
//
// Module definitions
//

/// Module implementing runtime-wrappers for compile-time guarantees about presence of data attributes.
mod guarantees;
pub use guarantees::*; // re-export all public facilities (mainly the guarantee wrapper and combination aliases).

/// Our derives
pub mod derives {
	pub use cgv_derive::{
		// Re-export our related procedural derive macros from cgv-derive
		NoNormals, NoTangents, NoRadii, NoRadiusDerivs, NoOrientations, NoScalings, NoColors
	};
}



//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Local imports
use crate::{self as cgv, *};
pub use derives::*; // re-export our derives for easy access



//////
//
// Traits
//

/// Trait of a collection of renderable data, ready for being turned into
/// [GPU-side render data](renderer::data::gpu::Data) for consumption by a [`Renderer`].
pub trait Data:
	  CanHaveNormals+CanHaveTangents+CanHaveRadii+CanHaveRadiusDerivs+CanHaveOrientations+CanHaveScalings+CanHaveColors
{
	/// The iterator type for iterating positions in the data.
	type PosIterator<'data>: Iterator<Item=glm::Vec3> where Self: 'data;

	/// Return the number of elements in the underlying data series.
	fn num (&self) -> u32;

	/// Iterate over the positions.
	fn positions (&self) -> Self::PosIterator<'_>;

	/// Reference a single position at the given index.
	fn pos (&self, index: u32) -> glm::Vec3;

	/// Return the preferred [topology](wgpu::PrimitiveTopology) of the data. Some renderers, like
	/// [`renderer::Spheres`], will completely ignore this, while others like [`renderer::Mesh`] will require specific
	/// topologies.
	fn topology (&self) -> wgpu::PrimitiveTopology;
}
/// Blanket implementation for slices of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::InterleavedElem> Data for [T]
{
	type PosIterator<'data> = util::notsafe::StridedCopyIter<'data, glm::Vec3> where Self: 'data;
	fn num (&self) -> u32 { self.len() as u32 }
	fn positions (&self) -> Self::PosIterator<'_> { unsafe {
		// SAFETY: We are a `Vec` of structs, and `Vec` can be trusted to return the correct length and place elements
		// with appropriate alignment, so the validity of the fields the iterator accesses is guaranteed.
		util::notsafe::StridedCopyIter::new(self[0].pos(), size_of::<T>(), self.len())
	}}
	fn pos (&self, index: u32) -> glm::Vec3 { *self[index as usize].pos() }
	fn topology (&self) -> wgpu::PrimitiveTopology { wgpu::PrimitiveTopology::PointList }
}
/// Blanket implementation for static arrays of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::InterleavedElem, const N: usize> Data for [T; N]
{
	type PosIterator<'data> = util::notsafe::StridedCopyIter<'data, glm::Vec3> where Self: 'data;
	fn num (&self) -> u32 { self.len() as u32 }
	fn positions (&self) -> Self::PosIterator<'_> { unsafe {
		// SAFETY: We are a `Vec` of structs, and `Vec` can be trusted to return the correct length and place elements
		// with appropriate alignment, so the validity of the fields the iterator accesses is guaranteed.
		util::notsafe::StridedCopyIter::new(self[0].pos(), size_of::<T>(), self.len())
	}}
	fn pos (&self, index: u32) -> glm::Vec3 { *self[index as usize].pos() }
	fn topology (&self) -> wgpu::PrimitiveTopology { wgpu::PrimitiveTopology::PointList }
}
/// Blanket implementation for `Vec`s of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::InterleavedElem> Data for Vec<T>
{
	type PosIterator<'data> = util::notsafe::StridedCopyIter<'data, glm::Vec3> where Self: 'data;
	fn num (&self) -> u32 { self.len() as u32 }
	fn positions (&self) -> Self::PosIterator<'_> { unsafe {
		// SAFETY: We are a `Vec` of structs, and `Vec` can be trusted to return the correct length and place elements
		// with appropriate alignment, so the validity of the fields the iterator accesses is guaranteed.
		util::notsafe::StridedCopyIter::new(self[0].pos(), size_of::<T>(), self.len())
	}}
	fn pos (&self, index: u32) -> glm::Vec3 { *self[index as usize].pos() }
	fn topology (&self) -> wgpu::PrimitiveTopology { wgpu::PrimitiveTopology::PointList }
}

/// Marker trait for [`renderer::HostData`] indicating that the data attributes are stored in an interleaved fashion (aka.
/// "array of structs").
pub trait Interleaved: Data {}
/// Blanket implementation for slices of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::InterleavedElem> Interleaved for [T] {}
/// Blanket implementation for static arrays of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::InterleavedElem, const N: usize> Interleaved for [T; N] {}
/// Blanket implementation for `Vec`s of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::InterleavedElem> Interleaved for Vec<T> {}

/// Marker trait for [`renderer::HostData`] indicating that the data attributes are stored in a non-interleaved fashion
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
	type IndexIterator<'data>: Iterator<Item=u32> where Self: 'data;

	/// Return the number of indices over the underlying data series.
	fn numIndices (&self) -> u32;

	/// Iterate over the indices.
	fn indices (&self) -> Self::IndexIterator<'_>;

	/// Reference a single data index or a slice of data indices.
	fn index (&self, index: u32) -> u32;
}

/// The trait of data knowing about the semantics of *normals*.
pub trait CanHaveNormals
{
	/// The iterator type for iterating normals in the data. The lifetime parameter `'data` ensures that implementations
	/// can use borrowing iterators.
	type NormalIterator<'data>: Iterator<Item=glm::Vec3> where Self: 'data;

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
	fn normal (&self, index: u32) -> glm::Vec3;
}
/// Blanket implementation for slices of [`renderer::data::InterleavedElem`]s.
impl<T: renderer::data::_ElemNormalBase> CanHaveNormals for [T]
{
	type NormalIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasNormals (&self) -> bool { T::_available() }
	#[inline(always)]
	fn normals (&self) -> Self::NormalIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn normal (&self, index: u32) -> glm::Vec3 { *self[index as usize].normal() }
}
/// Blanket implementation for static arrays of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::_ElemNormalBase, const N: usize> CanHaveNormals for [T; N]
{
	type NormalIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasNormals (&self) -> bool { T::_available() }
	#[inline(always)]
	fn normals (&self) -> Self::NormalIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn normal (&self, index: u32) -> glm::Vec3 { *self[index as usize].normal() }
}
/// Blanket implementation for `Vec`s of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::_ElemNormalBase> CanHaveNormals for Vec<T>
{
	type NormalIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasNormals (&self) -> bool { T::_available() }
	#[inline(always)]
	fn normals (&self) -> Self::NormalIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn normal (&self, index: u32) -> glm::Vec3 { *self[index as usize].normal() }
}

/// The trait of [host-side render data](Data) that is guaranteed to contain *normals*.
pub trait HasNormals: CanHaveNormals {}
/// Blanket implementation for slices of [`renderer::ElemWithNormal`]s.
impl<T: renderer::data::ElemWithNormal> HasNormals for [T] {}
/// Blanket implementation for static arrays of [`renderer::ElemWithNormal`]s.
impl<T: renderer::data::ElemWithNormal, const N: usize> HasNormals for [T; N] {}
/// Blanket implementation for `Vec`s of [`renderer::ElemWithNormal`]s.
impl<T: renderer::data::ElemWithNormal> HasNormals for Vec<T> {}

/// The trait of data knowing about the semantics of *tangents*.
pub trait CanHaveTangents
{
	/// The iterator type for iterating tangents in the data. The lifetime parameter `'data` ensures that
	/// implementations can use borrowing iterators.
	type TangentIterator<'data>: Iterator<Item=glm::Vec3> where Self: 'data;

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
	fn tangent (&self, index: u32) -> glm::Vec3;
}
/// Blanket implementation for slices of [`renderer::ElemWithTangent`]s.
impl<T: renderer::data::_ElemTangentBase> CanHaveTangents for [T]
{
	type TangentIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasTangents (&self) -> bool { T::_available() }
	#[inline(always)]
	fn tangents (&self) -> Self::TangentIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn tangent (&self, index: u32) -> glm::Vec3 { *self[index as usize].tangent() }
}
/// Blanket implementation for static arrays of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::_ElemTangentBase, const N: usize> CanHaveTangents for [T; N]
{
	type TangentIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasTangents (&self) -> bool { T::_available() }
	#[inline(always)]
	fn tangents (&self) -> Self::TangentIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn tangent (&self, index: u32) -> glm::Vec3 { *self[index as usize].tangent() }
}
/// Blanket implementation for `Vec`s of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::_ElemTangentBase> CanHaveTangents for Vec<T>
{
	type TangentIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasTangents (&self) -> bool { T::_available() }
	#[inline(always)]
	fn tangents (&self) -> Self::TangentIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn tangent (&self, index: u32) -> glm::Vec3 { *self[index as usize].tangent() }
}

/// The trait of [host-side render data](Data) that is guaranteed to contain *tangents*.
pub trait HasTangents: CanHaveTangents {}
/// Blanket implementation for slices of [`renderer::ElemWithTangent`]s.
impl<T: renderer::data::ElemWithTangent> HasTangents for [T] {}
/// Blanket implementation for static arrays of [`renderer::ElemWithTangent`]s.
impl<T: renderer::data::ElemWithTangent, const N: usize> HasTangents for [T; N] {}
/// Blanket implementation for `Vec`s of [`renderer::ElemWithTangent`]s.
impl<T: renderer::data::ElemWithTangent> HasTangents for Vec<T> {}

/// The trait of data knowing about the semantics of *radii*.
pub trait CanHaveRadii
{
	/// The iterator type for iterating radii in the data. The lifetime parameter `'data` ensures that implementations
	/// can use borrowing iterators.
	type RadiusIterator<'data>: Iterator<Item=f32> where Self: 'data;

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
/// Blanket implementation for slices of [`renderer::ElemWithRadius`]s.
impl<T: renderer::data::_ElemRadiusBase> CanHaveRadii for [T]
{
	type RadiusIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasRadii (&self) -> bool { T::_available() }
	#[inline(always)]
	fn radii (&self) -> Self::RadiusIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn radius (&self, index: u32) -> f32 { *self[index as usize].radius() }
}
/// Blanket implementation for static arrays of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::_ElemRadiusBase, const N: usize> CanHaveRadii for [T; N]
{
	type RadiusIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasRadii (&self) -> bool { T::_available() }
	#[inline(always)]
	fn radii (&self) -> Self::RadiusIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn radius (&self, index: u32) -> f32 { *self[index as usize].radius() }
}
/// Blanket implementation for `Vec`s of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::_ElemRadiusBase> CanHaveRadii for Vec<T>
{
	type RadiusIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasRadii (&self) -> bool { T::_available() }
	#[inline(always)]
	fn radii (&self) -> Self::RadiusIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn radius (&self, index: u32) -> f32 { *self[index as usize].radius() }
}

/// The trait of [host-side render data](Data) that is guaranteed to contain *radii*.
pub trait HasRadii: CanHaveRadii {}
/// Blanket implementation for slices of [`renderer::ElemWithRadius`]s.
impl<T: renderer::data::ElemWithRadius> HasRadii for [T] {}
/// Blanket implementation for static arrays of [`renderer::ElemWithRadius`]s.
impl<T: renderer::data::ElemWithRadius, const N: usize> HasRadii for [T; N] {}
/// Blanket implementation for `Vec`s of [`renderer::ElemWithRadius`]s.
impl<T: renderer::data::ElemWithRadius> HasRadii for Vec<T> {}

/// The trait of data knowing about the semantics of *radius derivatives*. Note that we do not require that the data
/// also knows about [radii](CanHaveRadii). We do actually treat radius derivatives as just another unrelated attribute.
pub trait CanHaveRadiusDerivs
{
	/// The iterator type for iterating radius derivatives in the data. The lifetime parameter `'data` ensures that
	/// implementations can use borrowing iterators.
	type RadiusDerivIterator<'data>: Iterator<Item=f32> where Self: 'data;

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
	fn radiusDerivs (&self) -> Self::RadiusDerivIterator<'_>;

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
/// Blanket implementation for slices of [`renderer::ElemWithRadiusDeriv`]s.
impl<T: renderer::data::_ElemRadiusDerivBase> CanHaveRadiusDerivs for [T]
{
	type RadiusDerivIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasRadiusDerivs (&self) -> bool { T::_available() }
	#[inline(always)]
	fn radiusDerivs (&self) -> Self::RadiusDerivIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn radiusDeriv (&self, index: u32) -> f32 { *self[index as usize].radiusDeriv() }
}
/// Blanket implementation for static arrays of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::_ElemRadiusDerivBase, const N: usize> CanHaveRadiusDerivs for [T; N]
{
	type RadiusDerivIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasRadiusDerivs (&self) -> bool { T::_available() }
	#[inline(always)]
	fn radiusDerivs (&self) -> Self::RadiusDerivIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn radiusDeriv (&self, index: u32) -> f32 { *self[index as usize].radiusDeriv() }
}
/// Blanket implementation for `Vec`s of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::_ElemRadiusDerivBase> CanHaveRadiusDerivs for Vec<T>
{
	type RadiusDerivIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasRadiusDerivs (&self) -> bool { T::_available() }
	#[inline(always)]
	fn radiusDerivs (&self) -> Self::RadiusDerivIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn radiusDeriv (&self, index: u32) -> f32 { *self[index as usize].radiusDeriv() }
}

/// The trait of [host-side render data](Data) that is guaranteed to contain *radius derivatives*.
pub trait HasRadiusDerivs: CanHaveRadiusDerivs {}
/// Blanket implementation for slices of [`renderer::ElemWithRadiusDeriv`]s.
impl<T: renderer::data::ElemWithRadiusDeriv> HasRadiusDerivs for [T] {}
/// Blanket implementation for static arrays of [`renderer::ElemWithRadiusDeriv`]s.
impl<T: renderer::data::ElemWithRadiusDeriv, const N: usize> HasRadiusDerivs for [T; N] {}
/// Blanket implementation for `Vec`s of [`renderer::ElemWithRadiusDeriv`]s.
impl<T: renderer::data::ElemWithRadiusDeriv> HasRadiusDerivs for Vec<T> {}

/// The trait of data knowing about the semantics of *orientations*.
pub trait CanHaveOrientations
{
	/// The iterator type for iterating orientations in the data. The lifetime parameter `'data` ensures that
	/// implementations can use borrowing iterators.
	type OrientationIterator<'data>: Iterator<Item=glm::Quat> where Self: 'data;

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
	fn orientation (&self, index: u32) -> glm::Quat;
}
/// Blanket implementation for slices of [`renderer::ElemWithOrientation`]s.
impl<T: renderer::data::_ElemOrientationBase> CanHaveOrientations for [T]
{
	type OrientationIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasOrientations (&self) -> bool { T::_available() }
	#[inline(always)]
	fn orientations (&self) -> Self::OrientationIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn orientation (&self, index: u32) -> glm::Quat { *self[index as usize].orientation() }
}
/// Blanket implementation for static arrays of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::_ElemOrientationBase, const N: usize> CanHaveOrientations for [T; N]
{
	type OrientationIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasOrientations (&self) -> bool { T::_available() }
	#[inline(always)]
	fn orientations (&self) -> Self::OrientationIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn orientation (&self, index: u32) -> glm::Quat { *self[index as usize].orientation() }
}
/// Blanket implementation for `Vec`s of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::_ElemOrientationBase> CanHaveOrientations for Vec<T>
{
	type OrientationIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasOrientations (&self) -> bool { T::_available() }
	#[inline(always)]
	fn orientations (&self) -> Self::OrientationIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn orientation (&self, index: u32) -> glm::Quat { *self[index as usize].orientation() }
}

/// The trait of [host-side render data](Data) that is guaranteed to contain *orientations*.
pub trait HasOrientations: CanHaveOrientations {}
/// Blanket implementation for slices of [`renderer::ElemWithOrientation`]s.
impl<T: renderer::data::ElemWithOrientation> HasOrientations for [T] {}
/// Blanket implementation for static arrays of [`renderer::ElemWithOrientation`]s.
impl<T: renderer::data::ElemWithOrientation, const N: usize> HasOrientations for [T; N] {}
/// Blanket implementation for `Vec`s of [`renderer::ElemWithOrientation`]s.
impl<T: renderer::data::ElemWithOrientation> HasOrientations for Vec<T> {}

/// The trait of data knowing about the semantics of *scaling vectors*.
pub trait CanHaveScalings
{
	/// The iterator type for iterating scaling vectors in the data. The lifetime parameter `'data` ensures that
	/// implementations can use borrowing iterators.
	type ScaleIterator<'data>: Iterator<Item=glm::Vec3> where Self: 'data;

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
	fn scaling (&self, index: u32) -> glm::Vec3;
}
/// Blanket implementation for slices of [`renderer::ElemWithScaling`]s.
impl<T: renderer::data::_ElemScalingBase> CanHaveScalings for [T]
{
	type ScaleIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasScalings (&self) -> bool { T::_available() }
	#[inline(always)]
	fn scalings (&self) -> Self::ScaleIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn scaling (&self, index: u32) -> glm::Vec3 { *self[index as usize].scaling() }
}
/// Blanket implementation for static arrays of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::_ElemScalingBase, const N: usize> CanHaveScalings for [T; N]
{
	type ScaleIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasScalings (&self) -> bool { T::_available() }
	#[inline(always)]
	fn scalings (&self) -> Self::ScaleIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn scaling (&self, index: u32) -> glm::Vec3 { *self[index as usize].scaling() }
}
/// Blanket implementation for `Vec`s of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::_ElemScalingBase> CanHaveScalings for Vec<T>
{
	type ScaleIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasScalings (&self) -> bool { T::_available() }
	#[inline(always)]
	fn scalings (&self) -> Self::ScaleIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn scaling (&self, index: u32) -> glm::Vec3 { *self[index as usize].scaling() }
}

/// The trait of [host-side render data](Data) that is guaranteed to contain *scaling values*.
pub trait HasScalings: CanHaveScalings {}
/// Blanket implementation for slices of [`renderer::ElemWithScaling`]s.
impl<T: renderer::data::ElemWithScaling> HasScalings for [T] {}
/// Blanket implementation for static arrays of [`renderer::ElemWithScaling`]s.
impl<T: renderer::data::ElemWithScaling, const N: usize> HasScalings for [T; N] {}
/// Blanket implementation for `Vec`s of [`renderer::ElemWithScaling`]s.
impl<T: renderer::data::ElemWithScaling> HasScalings for Vec<T> {}

/// The trait of data knowing about the semantics of *colors*.
pub trait CanHaveColors
{
	/// The iterator type for iterating colors in the data. The lifetime parameter `'data` ensures that implementations
	/// can use borrowing iterators.
	type ColorIterator<'data>: Iterator<Item=cgv::RGBA> where Self: 'data;

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
	fn color (&self, index: u32) -> cgv::RGBA;
}
/// Blanket implementation for slices of [`renderer::ElemWithColor`]s.
impl<T: renderer::data::_ElemColorBase> CanHaveColors for [T]
{
	type ColorIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasColors (&self) -> bool { T::_available() }
	#[inline(always)]
	fn colors (&self) -> Self::ColorIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn color (&self, index: u32) -> cgv::RGBA { *self[index as usize].color() }
}
/// Blanket implementation for static arrays of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::_ElemColorBase, const N: usize> CanHaveColors for [T; N]
{
	type ColorIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasColors (&self) -> bool { T::_available() }
	#[inline(always)]
	fn colors (&self) -> Self::ColorIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn color (&self, index: u32) -> cgv::RGBA { *self[index as usize].color() }
}
/// Blanket implementation for `Vec`s of [`renderer::InterleavedElem`]s.
impl<T: renderer::data::_ElemColorBase> CanHaveColors for Vec<T>
{
	type ColorIterator<'data> = T::_Iterator<'data> where Self: 'data;
	#[inline(always)]
	fn hasColors (&self) -> bool { T::_available() }
	#[inline(always)]
	fn colors (&self) -> Self::ColorIterator<'_> { self[0]._iter(self.len()) }
	#[inline(always)]
	fn color (&self, index: u32) -> cgv::RGBA { *self[index as usize].color() }
}

/// The trait of [host-side render data](Data) that is guaranteed to contain *colors*.
pub trait HasColors: CanHaveColors {}
/// Blanket implementation for slices of [`renderer::ElemWithColor`]s.
impl<T: renderer::data::ElemWithColor> HasColors for [T] {}
/// Blanket implementation for static arrays of [`renderer::ElemWithColor`]s.
impl<T: renderer::data::ElemWithColor, const N: usize> HasColors for [T; N] {}
/// Blanket implementation for `Vec`s of [`renderer::ElemWithColor`]s.
impl<T: renderer::data::ElemWithColor> HasColors for Vec<T> {}
