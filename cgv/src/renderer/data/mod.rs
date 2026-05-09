
//////
//
// Module definitions
//

/// Module defining the host-side data model.
pub mod host;

/// Module defining the GPU-side data model.
pub mod gpu;
pub use gpu::InterleavedBuffer; // re-export

/// Our derives
pub mod derives {
	pub use cgv_derive::{
		// Re-export our related procedural derive macros from cgv-derive
		InterleavedElem, ElemWithNormal, ElemWithTangent, ElemWithRadius, ElemWithRadiusDeriv, ElemWithOrientation,
		ElemWithScaling, ElemWithColor, NoNormal, NoTangent, NoRadius, NoRadiusDeriv, NoOrientation, NoScaling, NoColor
	};
}



//////
//
// Imports
//

// Bitflags library
use bitflags::bitflags;

// Local imports
pub use derives::*; // re-export our derives for easy access
use crate::{self as cgv, *};



//////
//
// Enums
//

/// Enum of the *optional* geometry attributes the renderer module explicitly knows about (positions are special and
/// always present).
#[repr(u8)]
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum GeometryAttribute {
	Normals = 0,
	Tangents = 1,
	Radii = 2,
	RadiusDerivs = 3,
	Orientations = 4,
	Scalings = 5,
	Colors = 6,
}
impl GeometryAttribute
{
	/// The highest slot any `GeometryAttribute` corresponds to.
	///
	/// The type is intentionally kept as `u8` (thus requiring an explicit cast to `usize` for most practical purposes)
	/// to emphasize that this number will always be quite small. Its value will always be one less than
	/// [`GeometryAttribute::NUM_SLOTS`].
	///
	/// # Examples
	///
	/// ```rust
	/// # use cgv::renderer::data::*;
	/// assert_eq!(GA::Colors.slot(), GeometryAttribute::MAX_SLOT as usize);
	/// ```
	pub const MAX_SLOT: u8 = {
		// Ensure we stay informed about the primitive representation used for `GeometryAttribute` in case it ever gets
		// changed
		util::assert_eq_size!(std::mem::Discriminant<GeometryAttribute>, u8);

		// Ensure we stay informed about the highest discriminant value whenever we change `GeometryAttribute`
		const MAX: u8 = GA::Colors.slot() as u8;
		util::const_assert_eq!(MAX, 6);
		MAX
	};

	/// The number of slots that an array storing one value per variant of the `GeometryAttribute` enum would have. Will
	/// always be one more than the related [`GeometryAttribute::MAX_SLOT`] constant.
	///
	/// The type is intentionally kept as `u8` (thus requiring an explicit cast to `usize` for most practical purposes)
	/// to emphasize that this number will always be quite small.
	pub const NUM_SLOTS: u8 = Self::MAX_SLOT + 1;

	/// The corresponding *slot* of a certain geometry attribute. This will always be one less than
	/// [`GeometryAttribute::NUM_SLOTS`] less-than-or-equal to [`GeometryAttribute::MAX_SLOT`].
	#[inline(always)]
	pub const fn slot (&self) -> usize
	{
		unsafe {
			// SAFETY:
			// `GeometryAttributes` is a `repr(u8)`, and the Rust specification states that the discriminants of enums
			// with primitive representation may be obtained via pointer casting even if the enum is complex:
			// https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
			*(self as *const Self as *const u8) as usize
		}
	}

	/// Construct the enum variant that corresponds to the least-signficant *set* of the provided mask.
	///
	/// # Arguments
	///
	/// * `mask` – The bitmask of the flag to turn into a `GeometryAttribute`.
	///
	/// # Returns
	///
	/// The `GeometryAttribute` that corresponds to the least-significant bit in the mask that is set.
	///
	/// # Examples
	///
	/// ```rust
	/// # use cgv::renderer::data::*;
	/// let tangents = GeometryAttribute::fromMask(GAF::RADII|GAF::TANGENTS);
	/// assert_eq!(tangents, GA::Tangents);  // mask is 0b0000110
	///
	/// let radii = GeometryAttribute::fromMask(GAF::SCALINGS|GAF::RADII);
	/// assert_eq!(radii, GA::Radii);        // mask is 0b0100100
	/// ```
	pub fn fromMask (mask: GeometryAttributeFlags) -> Self
	{
		if mask.contains(GAF::NORMALS) { return GA::Normals }
		if mask.contains(GAF::TANGENTS) { return GA::Tangents }
		if mask.contains(GAF::RADII) { return GA::Radii }
		if mask.contains(GAF::RADIUS_DERIVS) { return GA::RadiusDerivs }
		if mask.contains(GAF::ORIENTATIONS) { return GA::Orientations }
		if mask.contains(GAF::SCALINGS) { return GA::Scalings }
		if mask.contains(GAF::COLORS) { return GA::Colors }
		panic!("corrupt geometry attributes bitmask")
	}
}
/// Convenience shorthand for [`GeometryAttribute`].
pub type GA = GeometryAttribute;

// Bitflag definition: GeometryAttributes
bitflags! {
	/// Bitflags representing the various geometry attributes the renderer module explicitly knows about.
	pub struct GeometryAttributeFlags: u16 {
		const NORMALS       = 1 << 0;
		const TANGENTS      = 1 << 1;
		const RADII         = 1 << 2;
		const RADIUS_DERIVS = 1 << 3;
		const ORIENTATIONS  = 1 << 4;
		const SCALINGS      = 1 << 5;
		const COLORS        = 1 << 6;
    }
}
impl From<u16> for GeometryAttributeFlags {
	#[inline(always)]
	fn from (value: u16) -> Self {
		debug_assert!(value <= Self::all().bits());
		Self::from_bits(value).unwrap()
	}
}
impl From<GeometryAttribute> for GeometryAttributeFlags
{
	#[inline]
	fn from (value: GeometryAttribute) -> Self
	{
		match value {
			GA::Normals => Self::NORMALS,
			GA::Tangents => Self::TANGENTS,
			GA::Radii => Self::RADII,
			GA::RadiusDerivs => Self::RADIUS_DERIVS,
			GA::Orientations => Self::ORIENTATIONS,
			GA::Scalings => Self::SCALINGS,
			GA::Colors => Self::COLORS
		}
	}
}
/// Convenience shorthand for [`GeometryAttributeFlags`].
pub type GAF = GeometryAttributeFlags;



//////
//
// Traits
//

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the [`renderer::HostData`]
/// trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// use cgv::renderer::data::derives::*;
/// # fn assertHostData<D: cgv::renderer::HostData>() -> bool { true }
/// #[derive(
///    // generate InterleavedElem impl
///    InterleavedElem,
///    // empty impls for the InterleavedElem bounds that we don't have the attributes for
///    NoNormal,NoTangent,NoRadius,NoRadiusDeriv,NoOrientation,NoScaling,NoColor
/// )]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3
///     // ...
/// }
/// assertHostData::<&[MyVertex]>();
/// ```
pub trait InterleavedElem:
	ElemWithNormal+ElemWithTangent+ElemWithRadius+ElemWithRadiusDeriv+ElemWithOrientation+ElemWithScaling+ElemWithColor
{
	fn pos (&self) -> &glm::Vec3;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasNormals`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasNormals<D: cgv::renderer::data::host::HasNormals>() -> bool { true }
/// #[derive(cgv::renderer::data::ElemWithNormal)]
/// struct MyVertex {
///     #[cgv_renderAttr(normal)]
///     normal: glm::Vec3
///     // ...
/// }
/// assertHasNormals::<&[MyVertex]>();
/// ```
pub trait ElemWithNormal {
	fn normal (&self) -> &glm::Vec3;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasTangents`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasTangents<D: cgv::renderer::data::host::HasTangents>() -> bool { true }
/// #[derive(cgv::renderer::data::ElemWithTangent)]
/// struct MyVertex {
///     #[cgv_renderAttr(tangent)]
///     tangent: glm::Vec3
///     // ...
/// }
/// assertHasTangents::<&[MyVertex]>();
/// ```
pub trait ElemWithTangent {
	fn tangent (&self) -> &glm::Vec3;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasRadii`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasRadii<D: cgv::renderer::data::host::HasRadii>() -> bool { true }
/// #[derive(cgv::renderer::data::ElemWithRadius)]
/// struct MyVertex {
///     #[cgv_renderAttr(radius)]
///     radius: f32
///     // ...
/// }
/// assertHasRadii::<&[MyVertex]>();
/// ```
pub trait ElemWithRadius {
	fn radius (&self) -> &f32;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasRadiusDerivs`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasRadiusDerivs<D: cgv::renderer::data::host::HasRadiusDerivs>() -> bool { true }
/// #[derive(cgv::renderer::data::ElemWithRadiusDeriv)]
/// struct MyVertex {
///     #[cgv_renderAttr(radiusDeriv)]
///     radiusDeriv: f32
///     // ...
/// }
/// assertHasRadiusDerivs::<&[MyVertex]>();
/// ```
pub trait ElemWithRadiusDeriv {
	fn radiusDeriv (&self) -> &f32;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasOrientations`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasOrientations<D: cgv::renderer::data::host::HasOrientations>() -> bool { true }
/// #[derive(cgv::renderer::data::ElemWithOrientation)]
/// struct MyVertex {
///     #[cgv_renderAttr(orientation)]
///     orientation: glm::Quat
///     // ...
/// }
/// assertHasOrientations::<&[MyVertex]>();
/// ```
pub trait ElemWithOrientation {
	fn orientation (&self) -> &glm::Quat;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasScalings`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasScalings<D: cgv::renderer::data::host::HasScalings>() -> bool { true }
/// #[derive(cgv::renderer::data::ElemWithScaling)]
/// struct MyVertex {
///     #[cgv_renderAttr(scaling)]
///     scaling: glm::Vec3
///     // ...
/// }
/// assertHasScalings::<&[MyVertex]>();
/// ```
pub trait ElemWithScaling {
	fn scaling (&self) -> &glm::Vec3;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasColors`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasColors<D: cgv::renderer::data::host::HasColors>() -> bool { true }
/// #[derive(cgv::renderer::data::ElemWithColor)]
/// struct MyVertex {
///     #[cgv_renderAttr(color)]
///     color: cgv::RGBA
///     // ...
/// }
/// assertHasColors::<&[MyVertex]>();
/// ```
pub trait ElemWithColor {
	fn color (&self) -> &cgv::RGBA;
}
