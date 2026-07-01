
//////
//
// Module setup
//

/// A module provide utilities for working with curves, that is, univariate, possibly vector-valued functions.
module curves;


////
// Imports

// Local imports
import "cgv/math/polynomial.slang";


////
// Module namespace

// Insert into cgv::math namespace
public namespace cgv::math {



//////
//
// Interfaces
//

/// Common interface for all curves.
public interface ICurve {
	/// The type of the internal number representation.
	associatedtype Repr: __BuiltinFloatingPointType;
}



//////
//
// Module finalization
//

// Module namespace close
}
