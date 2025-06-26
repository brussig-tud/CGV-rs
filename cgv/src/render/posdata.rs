
//////
//
// Module definitions
//

/*/// Submodule implementing the [`SphereRenderer`](sphere::SphereRenderer).
pub mod sphere;*/



//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Local imports
use crate::*;



//////
//
// Structs and enums
//

/* nothing here yet */



//////
//
// Traits
//

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
