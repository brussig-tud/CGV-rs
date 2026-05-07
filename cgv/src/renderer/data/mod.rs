
//////
//
// Module definitions
//

/// Module defining the host-side data model.
pub mod host;

/// Module defining the GPU-side data model.
pub mod gpu;



//////
//
// Imports
//

// Bitflags library
use bitflags::bitflags;

// Local imports
pub use cgv_derive::{
	// Re-export the relevant procedural derive macros from cgv-derive
	InterleavedElem, ElemWithNormal, ElemWithTangent, ElemWithRadius, ElemWithRadiusDeriv, ElemWithOrientation,
	ElemWithScaling, ElemWithColor
};
use crate::{self as cgv, *};



//////
//
// Enums
//

// Bitflag definition: GeometryAttributes
bitflags! {
	/// Bitflags representing the various geometry attributes the renderer module knows about.
	struct GeometryAttributes: u16 {
		const NORMALS       = 1 << 1;
		const TANGENTS      = 1 << 2;
		const RADII         = 1 << 3;
		const RADIUS_DERIVS = 1 << 4;
		const ORIENTATIONS  = 1 << 5;
		const SCALINGS      = 1 << 6;
		const COLORS        = 1 << 7;
    }
}
/// Convenience shorthand for [`GeometryAttributes`].
type GA = GeometryAttributes;



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
/// # fn assertHostData<D: cgv::renderer::HostData>() -> bool { true }
/// #[derive(cgv::renderer::data::InterleavedElem)]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3
///     // ...
/// }
/// assertHostData::<&[MyVertex]>();
/// ```
pub trait InterleavedElem {
	fn pos (&self) -> &glm::Vec3;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasNormals`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasNormals<D: cgv::renderer::data::host::HasNormals>() -> bool { true }
/// #[derive(cgv::renderer::data::InterleavedElem,cgv::renderer::data::ElemWithNormal)]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3,
///     #[cgv_renderAttr(normal)]
///     normal: glm::Vec3
///     // ...
/// }
/// assertHasNormals::<&[MyVertex]>();
/// ```
pub trait ElemWithNormal: InterleavedElem {
	fn normal (&self) -> &glm::Vec3;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasTangents`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasTangents<D: cgv::renderer::data::host::HasTangents>() -> bool { true }
/// #[derive(cgv::renderer::data::InterleavedElem,cgv::renderer::data::ElemWithTangent)]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3,
///     #[cgv_renderAttr(tangent)]
///     tangent: glm::Vec3
///     // ...
/// }
/// assertHasTangents::<&[MyVertex]>();
/// ```
pub trait ElemWithTangent: InterleavedElem {
	fn tangent (&self) -> &glm::Vec3;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasRadii`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasRadii<D: cgv::renderer::data::host::HasRadii>() -> bool { true }
/// #[derive(cgv::renderer::data::InterleavedElem,cgv::renderer::data::ElemWithRadius)]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3,
///     #[cgv_renderAttr(radius)]
///     radius: f32
///     // ...
/// }
/// assertHasRadii::<&[MyVertex]>();
/// ```
pub trait ElemWithRadius: InterleavedElem {
	fn radius (&self) -> &f32;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasRadiusDerivs`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasRadiusDerivs<D: cgv::renderer::data::host::HasRadiusDerivs>() -> bool { true }
/// #[derive(
///     cgv::renderer::data::InterleavedElem,cgv::renderer::data::ElemWithRadiusDeriv
/// )]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3,
///     #[cgv_renderAttr(radius)]
///     radius: f32,
///     #[cgv_renderAttr(radiusDeriv)]
///     radiusDeriv: f32
///     // ...
/// }
/// assertHasRadiusDerivs::<&[MyVertex]>();
/// ```
pub trait ElemWithRadiusDeriv: InterleavedElem {
	fn radiusDeriv (&self) -> &f32;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasOrientations`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasOrientations<D: cgv::renderer::data::host::HasOrientations>() -> bool { true }
/// #[derive(cgv::renderer::data::InterleavedElem,cgv::renderer::data::ElemWithOrientation)]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3,
///     #[cgv_renderAttr(orientation)]
///     orientation: glm::Quat
///     // ...
/// }
/// assertHasOrientations::<&[MyVertex]>();
/// ```
pub trait ElemWithOrientation: InterleavedElem {
	fn orientation (&self) -> &glm::Quat;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasScalings`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasScalings<D: cgv::renderer::data::host::HasScalings>() -> bool { true }
/// #[derive(cgv::renderer::data::InterleavedElem,cgv::renderer::data::ElemWithScaling)]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3,
///     #[cgv_renderAttr(scaling)]
///     scaling: glm::Vec3
///     // ...
/// }
/// assertHasScalings::<&[MyVertex]>();
/// ```
pub trait ElemWithScaling: InterleavedElem {
	fn scaling (&self) -> &glm::Vec3;
}

/// Helper/convenience trait making a type eligible for receiving a blanket-implementation of the
/// [`data::host::HasColors`] trait on its slices.
///
/// This trait can be derived:
/// ```rust
/// # use cgv::glm as glm;
/// # fn assertHasColors<D: cgv::renderer::data::host::HasColors>() -> bool { true }
/// #[derive(cgv::renderer::data::InterleavedElem,cgv::renderer::data::ElemWithColor)]
/// struct MyVertex {
///     #[cgv_renderAttr(pos)]
///     position: glm::Vec3,
///     #[cgv_renderAttr(color)]
///     color: cgv::RGBA
///     // ...
/// }
/// assertHasColors::<&[MyVertex]>();
/// ```
pub trait ElemWithColor: InterleavedElem {
	fn color (&self) -> &cgv::RGBA;
}
