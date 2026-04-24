
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Local imports
//use crate::{self as cgv, *};



//////
//
// Traits
//

/// Trait of a collection of renderable data, ready for consumption by a [`Renderer`].
pub trait Data {
	/// Return the number of elements in the underlying data series.
	fn num (&self) -> u32;
}
