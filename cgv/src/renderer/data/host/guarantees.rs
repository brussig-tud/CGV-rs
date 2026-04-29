
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
// Macros
//

/// Helper macro to save some code down below when defining the aliases for all possible [`GuaranteeAttributes`]
/// instantiations.
macro_rules! defineGuaranteeAliases {
	($( $(#[$meta:meta])* $alias:ident = $wrapped:ty; )+ ) => {
		$(
			$(#[$meta])*
			pub type $alias<Wrappee> = $wrapped;
		)+
	};
}



//////
//
// Traits
//

/// Helper trait for asserting the presence of normals only when required by an instance of [`GuaranteeAttributes`].
trait RequireNormals<const REQUIRED: bool> {
	fn assertNormals (&self);
}
impl<T> RequireNormals<false> for T {
	#[inline(always)]
	fn assertNormals (&self) {}
}
impl<T: host::CanHaveNormals> RequireNormals<true> for T {
	#[inline(always)]
	fn assertNormals (&self) {
		assert!(self.hasNormals(), "to-be-wrapped data must contain normals!");
	}
}

/// Helper trait for asserting the presence of tangents only when required by an instance of [`GuaranteeAttributes`].
trait RequireTangents<const REQUIRED: bool> {
	fn assertTangents (&self);
}
impl<T> RequireTangents<false> for T {
	#[inline(always)]
	fn assertTangents (&self) {}
}
impl<T: host::CanHaveTangents> RequireTangents<true> for T {
	#[inline(always)]
	fn assertTangents (&self) {
		assert!(self.hasTangents(), "to-be-wrapped data must contain tangents!");
	}
}

/// Helper trait for asserting the presence of radii only when required by an instance of [`GuaranteeAttributes`].
trait RequireRadii<const REQUIRED: bool> {
	fn assertRadii (&self);
}
impl<T> RequireRadii<false> for T {
	#[inline(always)]
	fn assertRadii (&self) {}
}
impl<T: host::CanHaveRadii> RequireRadii<true> for T {
	#[inline(always)]
	fn assertRadii (&self) {
		assert!(self.hasRadii(), "to-be-wrapped data must contain radii!");
	}
}

/// Helper trait for asserting the presence of radius derivatives (and by extension radii) only when required by an
/// instance of [`GuaranteeAttributes`].
trait RequireRadiusDerivs<const REQUIRED: bool> {
	fn assertRadiusDerivs (&self);
}
impl<T> RequireRadiusDerivs<false> for T {
	#[inline(always)]
	fn assertRadiusDerivs (&self) {}
}
impl<T: host::CanHaveRadiusDerivs> RequireRadiusDerivs<true> for T {
	#[inline(always)]
	fn assertRadiusDerivs (&self) {
		assert!(self.hasRadii(), "to-be-wrapped data with radius derivatives must also contain radii!");
		assert!(self.hasRadiusDerivs(), "to-be-wrapped data must contain radius derivatives!");
	}
}

/// Helper trait for asserting the presence of orientations only when required by an instance of
/// [`GuaranteeAttributes`].
trait RequireOrientations<const REQUIRED: bool> {
	fn assertOrientations (&self);
}
impl<T> RequireOrientations<false> for T {
	#[inline(always)]
	fn assertOrientations (&self) {}
}
impl<T: host::CanHaveOrientations> RequireOrientations<true> for T {
	#[inline(always)]
	fn assertOrientations (&self) {
		assert!(self.hasOrientations(), "to-be-wrapped data must contain orientations!");
	}
}

/// Helper trait for asserting the presence of scaling vectors only when required by an instance of
/// [`GuaranteeAttributes`].
trait RequireScalings<const REQUIRED: bool> {
	fn assertScalings (&self);
}
impl<T> RequireScalings<false> for T {
	#[inline(always)]
	fn assertScalings (&self) {}
}
impl<T: host::CanHaveScalings> RequireScalings<true> for T {
	#[inline(always)]
	fn assertScalings (&self) {
		assert!(self.hasScalings(), "to-be-wrapped data must contain scaling vectors!");
	}
}

/// Helper trait for asserting the presence of colors only when required by an instance of [`GuaranteeAttributes`].
trait RequireColors<const REQUIRED: bool> {
	fn assertColors (&self);
}
impl<T> RequireColors<false> for T {
	#[inline(always)]
	fn assertColors (&self) {}
}
impl<T: host::CanHaveColors> RequireColors<true> for T {
	#[inline(always)]
	fn assertColors (&self) {
		assert!(self.hasColors(), "to-be-wrapped data must contain colors!");
	}
}



//////
//
// Structs
//

////
// GuaranteeAttributes

/// Wrapper to turn runtime presence of the specified attributes in some [`renderer::Data`] into a compile-time
/// guarantee. Panics during construction if the wrappee does not actually have the asked-for attributes.
pub struct GuaranteeAttributes<
	Wrappee: HostData, const NORMALS: bool, const TANGENTS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
>(Wrappee);

#[expect(private_bounds)] // TODO: Check if this works as intended when `GuaranteeAttributes` is used from the outside
impl<
	Wrappee: HostData, const NORMALS: bool, const TANGENTS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>
	where Wrappee: HostData + RequireNormals<NORMALS>+RequireTangents<TANGENTS>+RequireRadii<RADII>
	                        + RequireRadiusDerivs<RADIUS_DERIVS>+RequireOrientations<ORIENTATIONS>
	                        + RequireScalings<SCALINGS>+RequireColors<COLORS>
{
	/// Wraps a given instance of render data that with a compile-time guarantee that the attributes the wrapper is
	/// instantiated with are present.
	///
	/// # Arguments
	///
	/// * `data` – The [`renderer::Data`] to guarantee attributes for.
	///
	/// # Returns
	///
	/// A new `GuaranteeAttributes` wrapping the provided `data`.
	///
	/// # Panics
	///
	/// If the provided `data` does not contain the attributes the wrapper was instantiated with.
	#[inline(always)]
	pub fn new (data: Wrappee) -> Self
	{
		<Wrappee as RequireNormals<NORMALS>>::assertNormals(&data);
		<Wrappee as RequireTangents<TANGENTS>>::assertTangents(&data);
		<Wrappee as RequireRadii<RADII>>::assertRadii(&data);
		<Wrappee as RequireRadiusDerivs<RADIUS_DERIVS>>::assertRadiusDerivs(&data);
		<Wrappee as RequireOrientations<ORIENTATIONS>>::assertOrientations(&data);
		<Wrappee as RequireScalings<SCALINGS>>::assertScalings(&data);
		<Wrappee as RequireColors<COLORS>>::assertColors(&data);
		Self(data)
	}
}
impl<
	Wrappee: HostData, const NORMALS: bool, const TANGENTS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> HostData for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{
	type PosIterator<'data> = Wrappee::PosIterator<'data> where Wrappee: 'data;

	#[inline(always)]
	fn num (&self) -> u32 { self.0.num() }
	#[inline(always)]
	fn positions (&self) -> Self::PosIterator<'_> { self.0.positions() }
	#[inline(always)]
	fn pos (&self, index: u32) -> &glm::Vec3 { self.0.pos(index) }
	#[inline(always)]
	fn topology(&self) -> wgpu::PrimitiveTopology { self.0.topology() }
}
impl<
	Wrappee: host::Indexed, const NORMALS: bool, const TANGENTS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> host::Indexed for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{
	type IndexIterator<'data> = Wrappee::IndexIterator<'data> where Wrappee: 'data;

	#[inline(always)]
	fn numIndices (&self) -> u32 { self.0.numIndices() }
	#[inline(always)]
	fn indices (&self) -> Self::IndexIterator<'_> { self.0.indices() }
	#[inline(always)]
	fn index (&self, index: u32) -> u32 { self.0.index(index) }
}
impl<
	Wrappee: host::Interleaved, const NORMALS: bool, const TANGENTS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> host::Interleaved for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{}
impl<
	Wrappee: host::NonInterleaved, const NORMALS: bool, const TANGENTS: bool, const RADII: bool,
	const RADIUS_DERIVS: bool, const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> host::NonInterleaved for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{}

// Guarantee normals
impl<
	Wrappee: host::CanHaveNormals, const TANGENTS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> host::CanHaveNormals for GuaranteeAttributes<
	Wrappee, true, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{
	type NormalIterator<'data> = Wrappee::NormalIterator<'data> where Wrappee: 'data;

	#[inline(always)]
	fn hasNormals (&self) -> bool { true }
	#[inline(always)]
	fn normals (&self) -> Self::NormalIterator<'_> { self.0.normals() }
	#[inline(always)]
	fn normal (&self, index: u32) -> &glm::Vec3 { self.0.normal(index) }
}
impl<
	Wrappee: host::CanHaveNormals, const TANGENTS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> host::HasNormals for GuaranteeAttributes<
	Wrappee, true, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{}
impl<
	Wrappee: host::CanHaveNormals, const TANGENTS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> host::CanHaveNormals for GuaranteeAttributes<
	Wrappee, false, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{
	type NormalIterator<'data> = Wrappee::NormalIterator<'data> where Wrappee: 'data;

	#[inline(always)]
	fn hasNormals (&self) -> bool { self.0.hasNormals() }
	#[inline(always)]
	fn normals (&self) -> Self::NormalIterator<'_> { self.0.normals() }
	#[inline(always)]
	fn normal (&self, index: u32) -> &glm::Vec3 { self.0.normal(index) }
}
impl<
	Wrappee: host::HasNormals, const TANGENTS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> host::HasNormals for GuaranteeAttributes<
	Wrappee, false, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{}

// Guarantee tangents
impl<
	Wrappee: host::CanHaveTangents, const NORMALS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> host::CanHaveTangents for GuaranteeAttributes<
	Wrappee, NORMALS, true, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{
	type TangentIterator<'data> = Wrappee::TangentIterator<'data> where Wrappee: 'data;

	#[inline(always)]
	fn hasTangents (&self) -> bool { true }
	#[inline(always)]
	fn tangents (&self) -> Self::TangentIterator<'_> { self.0.tangents() }
	#[inline(always)]
	fn tangent (&self, index: u32) -> &glm::Vec3 { self.0.tangent(index) }
}
impl<
	Wrappee: host::CanHaveTangents, const NORMALS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> host::HasTangents for GuaranteeAttributes<
	Wrappee, NORMALS, true, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{}
impl<
	Wrappee: host::CanHaveTangents, const NORMALS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> host::CanHaveTangents for GuaranteeAttributes<
	Wrappee, NORMALS, false, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{
	type TangentIterator<'data> = Wrappee::TangentIterator<'data> where Wrappee: 'data;

	#[inline(always)]
	fn hasTangents (&self) -> bool { self.0.hasTangents() }
	#[inline(always)]
	fn tangents (&self) -> Self::TangentIterator<'_> { self.0.tangents() }
	#[inline(always)]
	fn tangent (&self, index: u32) -> &glm::Vec3 { self.0.tangent(index) }
}
impl<
	Wrappee: host::HasTangents, const NORMALS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> host::HasTangents for GuaranteeAttributes<
	Wrappee, NORMALS, false, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{}

// Guarantee radii/radius derivatives (needs to be treated together to handle Derivs => Radii requirement)
impl<
	Wrappee: host::CanHaveRadii, const NORMALS: bool, const TANGENTS: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> host::CanHaveRadii for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, true, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{
	type RadiusIterator<'data> = Wrappee::RadiusIterator<'data> where Wrappee: 'data;

	#[inline(always)]
	fn hasRadii (&self) -> bool { true }
	#[inline(always)]
	fn radii (&self) -> Self::RadiusIterator<'_> { self.0.radii() }
	#[inline(always)]
	fn radius (&self, index: u32) -> f32 { self.0.radius(index) }
}
impl<
	Wrappee: host::CanHaveRadii, const NORMALS: bool, const TANGENTS: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> host::HasRadii for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, true, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{}
impl<
	Wrappee: host::CanHaveRadiusDerivs, const NORMALS: bool, const TANGENTS: bool, const ORIENTATIONS: bool,
	const SCALINGS: bool, const COLORS: bool
> host::CanHaveRadii for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, false, true, ORIENTATIONS, SCALINGS, COLORS
>{
	type RadiusIterator<'data> = Wrappee::RadiusIterator<'data> where Wrappee: 'data;

	#[inline(always)]
	fn hasRadii (&self) -> bool { true }
	#[inline(always)]
	fn radii (&self) -> Self::RadiusIterator<'_> { self.0.radii() }
	#[inline(always)]
	fn radius (&self, index: u32) -> f32 { self.0.radius(index) }
}
impl<
	Wrappee: host::CanHaveRadiusDerivs, const NORMALS: bool, const TANGENTS: bool, const ORIENTATIONS: bool,
	const SCALINGS: bool, const COLORS: bool
> host::HasRadii for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, false, true, ORIENTATIONS, SCALINGS, COLORS
>{}
impl<
	Wrappee: host::CanHaveRadii, const NORMALS: bool, const TANGENTS: bool, const ORIENTATIONS: bool,
	const SCALINGS: bool, const COLORS: bool
> host::CanHaveRadii for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, false, false, ORIENTATIONS, SCALINGS, COLORS
>{
	type RadiusIterator<'data> = Wrappee::RadiusIterator<'data> where Wrappee: 'data;

	#[inline(always)]
	fn hasRadii (&self) -> bool { self.0.hasRadii() }
	#[inline(always)]
	fn radii (&self) -> Self::RadiusIterator<'_> { self.0.radii() }
	#[inline(always)]
	fn radius (&self, index: u32) -> f32 { self.0.radius(index) }
}
impl<
	Wrappee: host::HasRadii, const NORMALS: bool, const TANGENTS: bool, const ORIENTATIONS: bool, const SCALINGS: bool,
	const COLORS: bool
> host::HasRadii for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, false, false, ORIENTATIONS, SCALINGS, COLORS
>{}
impl<
	Wrappee: host::CanHaveRadiusDerivs, const NORMALS: bool, const TANGENTS: bool, const ORIENTATIONS: bool,
	const SCALINGS: bool, const COLORS: bool
> host::CanHaveRadiusDerivs for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, true, true, ORIENTATIONS, SCALINGS, COLORS
>{
	#[inline(always)]
	fn hasRadiusDerivs (&self) -> bool { true }
	#[inline(always)]
	fn radiusDerivs (&self) -> Self::RadiusIterator<'_> { self.0.radiusDerivs() }
	#[inline(always)]
	fn radiusDeriv (&self, index: u32) -> f32 { self.0.radiusDeriv(index) }
}
impl<
	Wrappee: host::CanHaveRadiusDerivs, const NORMALS: bool, const TANGENTS: bool, const ORIENTATIONS: bool,
	const SCALINGS: bool, const COLORS: bool
> host::HasRadiusDerivs for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, true, true, ORIENTATIONS, SCALINGS, COLORS
>{}
impl<
	Wrappee: host::CanHaveRadiusDerivs, const NORMALS: bool, const TANGENTS: bool, const ORIENTATIONS: bool,
	const SCALINGS: bool, const COLORS: bool
> host::CanHaveRadiusDerivs for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, true, false, ORIENTATIONS, SCALINGS, COLORS
>{
	#[inline(always)]
	fn hasRadiusDerivs (&self) -> bool { self.0.hasRadiusDerivs() }
	#[inline(always)]
	fn radiusDerivs (&self) -> Self::RadiusIterator<'_> { self.0.radiusDerivs() }
	#[inline(always)]
	fn radiusDeriv (&self, index: u32) -> f32 { self.0.radiusDeriv(index) }
}
impl<
	Wrappee: host::HasRadiusDerivs, const NORMALS: bool, const TANGENTS: bool, const ORIENTATIONS: bool,
	const SCALINGS: bool, const COLORS: bool
> host::HasRadiusDerivs for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, true, false, ORIENTATIONS, SCALINGS, COLORS
>{}
impl<
	Wrappee: host::CanHaveRadiusDerivs, const NORMALS: bool, const TANGENTS: bool, const ORIENTATIONS: bool,
	const SCALINGS: bool, const COLORS: bool
> host::CanHaveRadiusDerivs for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, false, false, ORIENTATIONS, SCALINGS, COLORS
>{
	#[inline(always)]
	fn hasRadiusDerivs (&self) -> bool { self.0.hasRadiusDerivs() }
	#[inline(always)]
	fn radiusDerivs (&self) -> Self::RadiusIterator<'_> { self.0.radiusDerivs() }
	#[inline(always)]
	fn radiusDeriv (&self, index: u32) -> f32 { self.0.radiusDeriv(index) }
}
impl<
	Wrappee: host::HasRadiusDerivs, const NORMALS: bool, const TANGENTS: bool, const ORIENTATIONS: bool,
	const SCALINGS: bool, const COLORS: bool
> host::HasRadiusDerivs for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, false, false, ORIENTATIONS, SCALINGS, COLORS
>{}

// Guarantee orientations
impl<
	Wrappee: host::CanHaveOrientations, const NORMALS: bool, const TANGENTS: bool, const RADII: bool,
	const RADIUS_DERIVS: bool, const SCALINGS: bool, const COLORS: bool
> host::CanHaveOrientations for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, true, SCALINGS, COLORS
>{
	type OrientationIterator<'data> = Wrappee::OrientationIterator<'data> where Wrappee: 'data;

	#[inline(always)]
	fn hasOrientations (&self) -> bool { true }
	#[inline(always)]
	fn orientations (&self) -> Self::OrientationIterator<'_> { self.0.orientations() }
	#[inline(always)]
	fn orientation (&self, index: u32) -> &glm::Quat { self.0.orientation(index) }
}
impl<
	Wrappee: host::CanHaveOrientations, const NORMALS: bool, const TANGENTS: bool, const RADII: bool,
	const RADIUS_DERIVS: bool, const SCALINGS: bool, const COLORS: bool
> host::HasOrientations for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, true, SCALINGS, COLORS
>{}
impl<
	Wrappee: host::CanHaveOrientations, const NORMALS: bool, const TANGENTS: bool, const RADII: bool,
	const RADIUS_DERIVS: bool, const SCALINGS: bool, const COLORS: bool
> host::CanHaveOrientations for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, false, SCALINGS, COLORS
>{
	type OrientationIterator<'data> = Wrappee::OrientationIterator<'data> where Wrappee: 'data;

	#[inline(always)]
	fn hasOrientations (&self) -> bool { self.0.hasOrientations() }
	#[inline(always)]
	fn orientations (&self) -> Self::OrientationIterator<'_> { self.0.orientations() }
	#[inline(always)]
	fn orientation (&self, index: u32) -> &glm::Quat { self.0.orientation(index) }
}
impl<
	Wrappee: host::HasOrientations, const NORMALS: bool, const TANGENTS: bool, const RADII: bool,
	const RADIUS_DERIVS: bool, const SCALINGS: bool, const COLORS: bool
> host::HasOrientations for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, false, SCALINGS, COLORS
>{}

// Guarantee scaling vectors
impl<
	Wrappee: host::CanHaveScalings, const NORMALS: bool, const TANGENTS: bool, const RADII: bool,
	const RADIUS_DERIVS: bool, const ORIENTATIONS: bool, const COLORS: bool
> host::CanHaveScalings for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, true, COLORS
>{
	type ScaleIterator<'data> = Wrappee::ScaleIterator<'data> where Wrappee: 'data;

	#[inline(always)]
	fn hasScalings (&self) -> bool { true }
	#[inline(always)]
	fn scalings (&self) -> Self::ScaleIterator<'_> { self.0.scalings() }
	#[inline(always)]
	fn scaling (&self, index: u32) -> &glm::Vec3 { self.0.scaling(index) }
}
impl<
	Wrappee: host::CanHaveScalings, const NORMALS: bool, const TANGENTS: bool, const RADII: bool,
	const RADIUS_DERIVS: bool, const ORIENTATIONS: bool, const COLORS: bool
> host::HasScalings for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, true, COLORS
>{}
impl<
	Wrappee: host::CanHaveScalings, const NORMALS: bool, const TANGENTS: bool, const RADII: bool,
	const RADIUS_DERIVS: bool, const ORIENTATIONS: bool, const COLORS: bool
> host::CanHaveScalings for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, false, COLORS
>{
	type ScaleIterator<'data> = Wrappee::ScaleIterator<'data> where Wrappee: 'data;

	#[inline(always)]
	fn hasScalings (&self) -> bool { self.0.hasScalings() }
	#[inline(always)]
	fn scalings (&self) -> Self::ScaleIterator<'_> { self.0.scalings() }
	#[inline(always)]
	fn scaling (&self, index: u32) -> &glm::Vec3 { self.0.scaling(index) }
}
impl<
	Wrappee: host::HasScalings, const NORMALS: bool, const TANGENTS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const COLORS: bool
> host::HasScalings for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, false, COLORS
>{}

// Guarantee colors
impl<
	Wrappee: host::CanHaveColors, const NORMALS: bool, const TANGENTS: bool, const RADII: bool,
	const RADIUS_DERIVS: bool, const ORIENTATIONS: bool, const SCALINGS: bool
> host::CanHaveColors for	GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, true
>{
	type ColorIterator<'data> = Wrappee::ColorIterator<'data> where Wrappee: 'data;

	#[inline(always)]
	fn hasColors (&self) -> bool { true }
	#[inline(always)]
	fn colors (&self) -> Self::ColorIterator<'_> { self.0.colors() }
	#[inline(always)]
	fn color (&self, index: u32) -> &cgv::RGBA { self.0.color(index) }
}
impl<
	Wrappee: host::CanHaveColors, const NORMALS: bool, const TANGENTS: bool, const RADII: bool,
	const RADIUS_DERIVS: bool, const ORIENTATIONS: bool, const SCALINGS: bool
> host::HasColors for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, true
>{}
impl<
	Wrappee: host::CanHaveColors, const NORMALS: bool, const TANGENTS: bool, const RADII: bool,
	const RADIUS_DERIVS: bool, const ORIENTATIONS: bool, const SCALINGS: bool
> host::CanHaveColors for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, false
>{
	type ColorIterator<'data> = Wrappee::ColorIterator<'data> where Wrappee: 'data;

	#[inline(always)]
	fn hasColors (&self) -> bool { self.0.hasColors() }
	#[inline(always)]
	fn colors (&self) -> Self::ColorIterator<'_> { self.0.colors() }
	#[inline(always)]
	fn color (&self, index: u32) -> &cgv::RGBA { self.0.color(index) }
}
impl<
	Wrappee: host::HasColors, const NORMALS: bool, const TANGENTS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool
> host::HasColors for	GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, false
>{}

// Add ability to Deref (and DerefMut) to wrapped type
impl<
	Wrappee: HostData, const NORMALS: bool, const TANGENTS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> Deref for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{
	type Target = Wrappee;

	#[inline(always)]
	fn deref (&self) -> &Self::Target { &self.0 }
}
impl<
	Wrappee: HostData, const NORMALS: bool, const TANGENTS: bool, const RADII: bool, const RADIUS_DERIVS: bool,
	const ORIENTATIONS: bool, const SCALINGS: bool, const COLORS: bool
> DerefMut for GuaranteeAttributes<
	Wrappee, NORMALS, TANGENTS, RADII, RADIUS_DERIVS, ORIENTATIONS, SCALINGS, COLORS
>{
	#[inline(always)]
	fn deref_mut (&mut self) -> &mut Self::Target { &mut self.0 }
}



//////
//
// Combination aliases
//

// Canonical flat aliases for all attribute-name combinations. Aliases mentioning `RadiusDerivs` imply radii as well,
// so those combinations set both radius-related guarantees on the shared combination wrapper.
defineGuaranteeAliases!(
	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals are present.
	GuaranteeNormals = GuaranteeAttributes<Wrappee, true, false, false, false, false, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents are present.
	GuaranteeTangents = GuaranteeAttributes<Wrappee, false, true, false, false, false, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] a compile-time guarantee that radii are present.
	GuaranteeRadii = GuaranteeAttributes<Wrappee, false, false, true, false, false, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radius derivatives are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiusDerivs = GuaranteeAttributes<Wrappee, false, false, true, true, false, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that orientations are present.
	GuaranteeOrientations = GuaranteeAttributes<Wrappee, false, false, false, false, true, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that scaling vectors are present.
	GuaranteeScalings = GuaranteeAttributes<Wrappee, false, false, false, false, false, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that colors are present.
	GuaranteeColors = GuaranteeAttributes<Wrappee, false, false, false, false, false, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals and tangents are present.
	GuaranteeNormalsTangents = GuaranteeAttributes<Wrappee, true, true, false, false, false, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals and radii are present.
	GuaranteeNormalsRadii = GuaranteeAttributes<Wrappee, true, false, true, false, false, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals and radius derivatives are
	/// present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiusDerivs = GuaranteeAttributes<Wrappee, true, false, true, true, false, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals and orientations are
	/// present.
	GuaranteeNormalsOrientations = GuaranteeAttributes<Wrappee, true, false, false, false, true, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals and scaling vectors are
	/// present.
	GuaranteeNormalsScalings = GuaranteeAttributes<Wrappee, true, false, false, false, false, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals and colors are present.
	GuaranteeNormalsColors = GuaranteeAttributes<Wrappee, true, false, false, false, false, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents and radii are present.
	GuaranteeTangentsRadii = GuaranteeAttributes<Wrappee, false, true, true, false, false, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents and radius derivatives are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiusDerivs = GuaranteeAttributes<Wrappee, false, true, true, true, false, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents and orientations are
	/// present.
	GuaranteeTangentsOrientations = GuaranteeAttributes<Wrappee, false, true, false, false, true, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents and scaling vectors are
	/// present.
	GuaranteeTangentsScalings = GuaranteeAttributes<Wrappee, false, true, false, false, false, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents and colors are present.
	GuaranteeTangentsColors = GuaranteeAttributes<Wrappee, false, true, false, false, false, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radii and radius derivatives are
	/// present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiiRadiusDerivs = GuaranteeAttributes<Wrappee, false, false, true, true, false, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radii and orientations are present.
	GuaranteeRadiiOrientations = GuaranteeAttributes<Wrappee, false, false, true, false, true, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radii and scaling vectors are
	/// present.
	GuaranteeRadiiScalings = GuaranteeAttributes<Wrappee, false, false, true, false, false, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radii and colors are present.
	GuaranteeRadiiColors = GuaranteeAttributes<Wrappee, false, false, true, false, false, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radius derivatives and orientations
	/// are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiusDerivsOrientations = GuaranteeAttributes<Wrappee, false, false, true, true, true, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radius derivatives and scaling
	/// vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiusDerivsScalings = GuaranteeAttributes<Wrappee, false, false, true, true, false, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radius derivatives and colors are
	/// present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiusDerivsColors = GuaranteeAttributes<Wrappee, false, false, true, true, false, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that orientations and scaling vectors are
	/// present.
	GuaranteeOrientationsScalings = GuaranteeAttributes<Wrappee, false, false, false, false, true, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that orientations and colors are present.
	GuaranteeOrientationsColors = GuaranteeAttributes<Wrappee, false, false, false, false, true, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that scaling vectors and colors are
	/// present.
	GuaranteeScalingsColors = GuaranteeAttributes<Wrappee, false, false, false, false, false, true, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, and radii are
	/// present.
	GuaranteeNormalsTangentsRadii = GuaranteeAttributes<Wrappee, true, true, true, false, false, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, and radius
	/// derivatives are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiusDerivs = GuaranteeAttributes<Wrappee, true, true, true, true, false, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, and orientations
	/// are present.
	GuaranteeNormalsTangentsOrientations = GuaranteeAttributes<Wrappee, true, true, false, false, true, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, and scaling
	/// vectors are present.
	GuaranteeNormalsTangentsScalings = GuaranteeAttributes<Wrappee, true, true, false, false, false, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, and colors are
	/// present.
	GuaranteeNormalsTangentsColors = GuaranteeAttributes<Wrappee, true, true, false, false, false, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radii, and radius
	/// derivatives are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiiRadiusDerivs = GuaranteeAttributes<Wrappee, true, false, true, true, false, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radii, and orientations are
	/// present.
	GuaranteeNormalsRadiiOrientations = GuaranteeAttributes<Wrappee, true, false, true, false, true, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radii, and scaling vectors
	/// are present.
	GuaranteeNormalsRadiiScalings = GuaranteeAttributes<Wrappee, true, false, true, false, false, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radii, and colors are
	/// present.
	GuaranteeNormalsRadiiColors = GuaranteeAttributes<Wrappee, true, false, true, false, false, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radius derivatives, and
	/// orientations are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiusDerivsOrientations = GuaranteeAttributes<
		Wrappee, true, false, true, true, true, false, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radius derivatives, and
	/// scaling vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiusDerivsScalings = GuaranteeAttributes<Wrappee, true, false, true, true, false, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radius derivatives, and
	/// colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiusDerivsColors = GuaranteeAttributes<Wrappee, true, false, true, true, false, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, orientations, and scaling
	/// vectors are present.
	GuaranteeNormalsOrientationsScalings = GuaranteeAttributes<Wrappee, true, false, false, false, true, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, orientations, and colors
	/// are present.
	GuaranteeNormalsOrientationsColors = GuaranteeAttributes<Wrappee, true, false, false, false, true, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, scaling vectors, and colors
	/// are present.
	GuaranteeNormalsScalingsColors = GuaranteeAttributes<Wrappee, true, false, false, false, false, true, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radii, and radius
	/// derivatives are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiiRadiusDerivs = GuaranteeAttributes<Wrappee, false, true, true, true, false, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radii, and orientations
	/// are present.
	GuaranteeTangentsRadiiOrientations = GuaranteeAttributes<Wrappee, false, true, true, false, true, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radii, and scaling vectors
	/// are present.
	GuaranteeTangentsRadiiScalings = GuaranteeAttributes<Wrappee, false, true, true, false, false, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radii, and colors are
	/// present.
	GuaranteeTangentsRadiiColors = GuaranteeAttributes<Wrappee, false, true, true, false, false, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radius derivatives, and
	/// orientations are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiusDerivsOrientations = GuaranteeAttributes<
		Wrappee, false, true, true, true, true, false, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radius derivatives, and
	/// scaling vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiusDerivsScalings = GuaranteeAttributes<Wrappee, false, true, true, true, false, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radius derivatives, and
	/// colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiusDerivsColors = GuaranteeAttributes<Wrappee, false, true, true, true, false, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, orientations, and scaling
	/// vectors are present.
	GuaranteeTangentsOrientationsScalings = GuaranteeAttributes<Wrappee, false, true, false, false, true, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, orientations, and colors
	/// are present.
	GuaranteeTangentsOrientationsColors = GuaranteeAttributes<Wrappee, false, true, false, false, true, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, scaling vectors, and colors are present.
	GuaranteeTangentsScalingsColors = GuaranteeAttributes<Wrappee, false, true, false, false, false, true, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radii, radius derivatives, and
	/// orientations are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiiRadiusDerivsOrientations = GuaranteeAttributes<Wrappee, false, false, true, true, true, false, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radii, radius derivatives, and
	/// scaling vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiiRadiusDerivsScalings = GuaranteeAttributes<Wrappee, false, false, true, true, false, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radii, radius derivatives, and
	/// colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiiRadiusDerivsColors = GuaranteeAttributes<Wrappee, false, false, true, true, false, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radii, orientations, and scaling
	/// vectors are present.
	GuaranteeRadiiOrientationsScalings = GuaranteeAttributes<Wrappee, false, false, true, false, true, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radii, orientations, and colors are
	/// present.
	GuaranteeRadiiOrientationsColors = GuaranteeAttributes<Wrappee, false, false, true, false, true, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radii, scaling vectors, and colors
	/// are present.
	GuaranteeRadiiScalingsColors = GuaranteeAttributes<Wrappee, false, false, true, false, false, true, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radius derivatives, orientations,
	/// and scaling vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiusDerivsOrientationsScalings = GuaranteeAttributes<
		Wrappee, false, false, true, true, true, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radius derivatives, orientations,
	/// and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiusDerivsOrientationsColors = GuaranteeAttributes<Wrappee, false, false, true, true, true, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radius derivatives, scaling vectors,
	/// and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiusDerivsScalingsColors = GuaranteeAttributes<Wrappee, false, false, true, true, false, true, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that orientations, scaling vectors, and
	/// colors are present.
	GuaranteeOrientationsScalingsColors = GuaranteeAttributes<Wrappee, false, false, false, false, true, true, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radii, and radius
	/// derivatives are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiiRadiusDerivs = GuaranteeAttributes<
		Wrappee, true, true, true, true, false, false, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radii, and
	/// orientations are present.
	GuaranteeNormalsTangentsRadiiOrientations = GuaranteeAttributes<
		Wrappee, true, true, true, false, true, false, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radii, and
	/// scaling vectors are present.
	GuaranteeNormalsTangentsRadiiScalings = GuaranteeAttributes<Wrappee, true, true, true, false, false, true, false>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radii, and colors
	/// are present.
	GuaranteeNormalsTangentsRadiiColors = GuaranteeAttributes<Wrappee, true, true, true, false, false, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radius
	/// derivatives, and orientations are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiusDerivsOrientations = GuaranteeAttributes<
		Wrappee, true, true, true, true, true, false, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radius
	/// derivatives, and scaling vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiusDerivsScalings = GuaranteeAttributes<
		Wrappee, true, true, true, true, false, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radius
	/// derivatives, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiusDerivsColors = GuaranteeAttributes<
		Wrappee, true, true, true, true, false, false, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, orientations, and
	/// scaling vectors are present.
	GuaranteeNormalsTangentsOrientationsScalings = GuaranteeAttributes<
		Wrappee, true, true, false, false, true, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, orientations, and
	/// colors are present.
	GuaranteeNormalsTangentsOrientationsColors = GuaranteeAttributes<
		Wrappee, true, true, false, false, true, false, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, scaling vectors,
	/// and colors are present.
	GuaranteeNormalsTangentsScalingsColors = GuaranteeAttributes<Wrappee, true, true, false, false, false, true, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radii, radius derivatives,
	/// and orientations are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiiRadiusDerivsOrientations = GuaranteeAttributes<
		Wrappee, true, false, true, true, true, false, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radii, radius derivatives,
	/// and scaling vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiiRadiusDerivsScalings = GuaranteeAttributes<
		Wrappee, true, false, true, true, false, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radii, radius derivatives,
	/// and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiiRadiusDerivsColors = GuaranteeAttributes<Wrappee, true, false, true, true, false, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radii, orientations, and
	/// scaling vectors are present.
	GuaranteeNormalsRadiiOrientationsScalings = GuaranteeAttributes<
		Wrappee, true, false, true, false, true, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radii, orientations, and
	/// colors are present.
	GuaranteeNormalsRadiiOrientationsColors = GuaranteeAttributes<Wrappee, true, false, true, false, true, false, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radii, scaling vectors, and
	/// colors are present.
	GuaranteeNormalsRadiiScalingsColors = GuaranteeAttributes<Wrappee, true, false, true, false, false, true, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radius derivatives,
	/// orientations, and scaling vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiusDerivsOrientationsScalings = GuaranteeAttributes<
		Wrappee, true, false, true, true, true, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radius derivatives,
	/// orientations, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiusDerivsOrientationsColors = GuaranteeAttributes<
		Wrappee, true, false, true, true, true, false, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radius derivatives, scaling
	/// vectors, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiusDerivsScalingsColors = GuaranteeAttributes<
		Wrappee, true, false, true, true, false, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, orientations, scaling
	/// vectors, and colors are present.
	GuaranteeNormalsOrientationsScalingsColors = GuaranteeAttributes<
		Wrappee, true, false, false, false, true, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radii, radius derivatives,
	/// and orientations are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiiRadiusDerivsOrientations = GuaranteeAttributes<
		Wrappee, false, true, true, true, true, false, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radii, radius derivatives,
	/// and scaling vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiiRadiusDerivsScalings = GuaranteeAttributes<
		Wrappee, false, true, true, true, false, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radii, radius derivatives,
	/// and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiiRadiusDerivsColors = GuaranteeAttributes<
		Wrappee, false, true, true, true, false, false, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radii, orientations, and
	/// scaling vectors are present.
	GuaranteeTangentsRadiiOrientationsScalings = GuaranteeAttributes<
		Wrappee, false, true, true, false, true, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radii, orientations, and
	/// colors are present.
	GuaranteeTangentsRadiiOrientationsColors = GuaranteeAttributes<
		Wrappee, false, true, true, false, true, false, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radii, scaling vectors,
	/// and colors are present.
	GuaranteeTangentsRadiiScalingsColors = GuaranteeAttributes<Wrappee, false, true, true, false, false, true, true>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radius derivatives,
	/// orientations, and scaling vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiusDerivsOrientationsScalings = GuaranteeAttributes<
		Wrappee, false, true, true, true, true, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radius derivatives,
	/// orientations, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiusDerivsOrientationsColors = GuaranteeAttributes<
		Wrappee, false, true, true, true, true, false, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radius derivatives,
	/// scaling vectors, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiusDerivsScalingsColors = GuaranteeAttributes<
		Wrappee, false, true, true, true, false, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, orientations, scaling
	/// vectors, and colors are present.
	GuaranteeTangentsOrientationsScalingsColors = GuaranteeAttributes<
		Wrappee, false, true, false, false, true, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radii, radius derivatives,
	/// orientations, and scaling vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiiRadiusDerivsOrientationsScalings = GuaranteeAttributes<
		Wrappee, false, false, true, true, true, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radii, radius derivatives,
	/// orientations, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiiRadiusDerivsOrientationsColors = GuaranteeAttributes<
		Wrappee, false, false, true, true, true, false, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radii, radius derivatives, scaling
	/// vectors, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiiRadiusDerivsScalingsColors = GuaranteeAttributes<
		Wrappee, false, false, true, true, false, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radii, orientations, scaling
	/// vectors, and colors are present.
	GuaranteeRadiiOrientationsScalingsColors = GuaranteeAttributes<
		Wrappee, false, false, true, false, true, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radius derivatives, orientations,
	/// scaling vectors, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiusDerivsOrientationsScalingsColors = GuaranteeAttributes<
		Wrappee, false, false, true, true, true, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radii, radius
	/// derivatives, and orientations are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiiRadiusDerivsOrientations = GuaranteeAttributes<
		Wrappee, true, true, true, true, true, false, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radii, radius
	/// derivatives, and scaling vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiiRadiusDerivsScalings = GuaranteeAttributes<
		Wrappee, true, true, true, true, false, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radii, radius
	/// derivatives, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiiRadiusDerivsColors = GuaranteeAttributes<
		Wrappee, true, true, true, true, false, false, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radii,
	/// orientations, and scaling vectors are present.
	GuaranteeNormalsTangentsRadiiOrientationsScalings = GuaranteeAttributes<
		Wrappee, true, true, true, false, true, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radii,
	/// orientations, and colors are present.
	GuaranteeNormalsTangentsRadiiOrientationsColors = GuaranteeAttributes<
		Wrappee, true, true, true, false, true, false, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radii,
	/// scaling vectors, and colors are present.
	GuaranteeNormalsTangentsRadiiScalingsColors = GuaranteeAttributes<
		Wrappee, true, true, true, false, false, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radius
	/// derivatives, orientations, and scaling vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiusDerivsOrientationsScalings = GuaranteeAttributes<
		Wrappee, true, true, true, true, true, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radius
	/// derivatives, orientations, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiusDerivsOrientationsColors = GuaranteeAttributes<
		Wrappee, true, true, true, true, true, false, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radius
	/// derivatives, scaling vectors, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiusDerivsScalingsColors = GuaranteeAttributes<
		Wrappee, true, true, true, true, false, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, orientations,
	/// scaling vectors, and colors are present.
	GuaranteeNormalsTangentsOrientationsScalingsColors = GuaranteeAttributes<
		Wrappee, true, true, false, false, true, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radii, radius derivatives,
	/// orientations, and scaling vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiiRadiusDerivsOrientationsScalings = GuaranteeAttributes<
		Wrappee, true, false, true, true, true, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radii, radius derivatives,
	/// orientations, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiiRadiusDerivsOrientationsColors = GuaranteeAttributes<
		Wrappee, true, false, true, true, true, false, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radii, radius derivatives,
	/// scaling vectors, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiiRadiusDerivsScalingsColors = GuaranteeAttributes<
		Wrappee, true, false, true, true, false, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radii, orientations,
	/// scaling vectors, and colors are present.
	GuaranteeNormalsRadiiOrientationsScalingsColors = GuaranteeAttributes<
		Wrappee, true, false, true, false, true, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radius derivatives,
	/// orientations, scaling vectors, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiusDerivsOrientationsScalingsColors = GuaranteeAttributes<
		Wrappee, true, false, true, true, true, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radii, radius derivatives,
	/// orientations, and scaling vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiiRadiusDerivsOrientationsScalings = GuaranteeAttributes<
		Wrappee, false, true, true, true, true, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radii, radius derivatives,
	/// orientations, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiiRadiusDerivsOrientationsColors = GuaranteeAttributes<
		Wrappee, false, true, true, true, true, false, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radii, radius derivatives,
	/// scaling vectors, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiiRadiusDerivsScalingsColors = GuaranteeAttributes<
		Wrappee, false, true, true, true, false, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radii, orientations,
	/// scaling vectors, and colors are present.
	GuaranteeTangentsRadiiOrientationsScalingsColors = GuaranteeAttributes<
		Wrappee, false, true, true, false, true, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radius derivatives,
	/// orientations, scaling vectors, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiusDerivsOrientationsScalingsColors = GuaranteeAttributes<
		Wrappee, false, true, true, true, true, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that radii, radius derivatives,
	/// orientations, scaling vectors, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeRadiiRadiusDerivsOrientationsScalingsColors = GuaranteeAttributes<
		Wrappee, false, false, true, true, true, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radii, radius
	/// derivatives, orientations, and scaling vectors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiiRadiusDerivsOrientationsScalings = GuaranteeAttributes<
		Wrappee, true, true, true, true, true, true, false
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radii, radius
	/// derivatives, orientations, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiiRadiusDerivsOrientationsColors = GuaranteeAttributes<
		Wrappee, true, true, true, true, true, false, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radii, radius
	/// derivatives, scaling vectors, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiiRadiusDerivsScalingsColors = GuaranteeAttributes<
		Wrappee, true, true, true, true, false, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radii,
	/// orientations, scaling vectors, and colors are present.
	GuaranteeNormalsTangentsRadiiOrientationsScalingsColors = GuaranteeAttributes<
		Wrappee, true, true, true, false, true, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radius
	/// derivatives, orientations, scaling vectors, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiusDerivsOrientationsScalingsColors = GuaranteeAttributes<
		Wrappee, true, true, true, true, true, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, radii, radius derivatives,
	/// orientations, scaling vectors, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsRadiiRadiusDerivsOrientationsScalingsColors = GuaranteeAttributes<
		Wrappee, true, false, true, true, true, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that tangents, radii, radius derivatives,
	/// orientations, scaling vectors, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeTangentsRadiiRadiusDerivsOrientationsScalingsColors = GuaranteeAttributes<
		Wrappee, false, true, true, true, true, true, true
	>;

	/// Wrapper for arbitrary [`renderer::Data`] with a compile-time guarantee that normals, tangents, radii, radius
	/// derivatives, orientations, scaling vectors, and colors are present.<br/>
	/// **NOTE**: Guaranteeing radius derivatives implies guaranteeing radii as well.
	GuaranteeNormalsTangentsRadiiRadiusDerivsOrientationsScalingsColors = GuaranteeAttributes<
		Wrappee, true, true, true, true, true, true, true
	>;
);
