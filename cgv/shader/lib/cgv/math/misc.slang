
//////
//
// Module definition
//

// A module providing coordinate system transformation utilities.
module misc;


/////
// Module namespace

// Insert into cgv::math namespace
namespace cgv::math {



//////
//
// Functions
//

/// Perform the operation `abs(floor())` on the input number.
/// # Arguments
/// `x` – The value to floor robustly.
/// # Returns
/// The result of the expression `abs(floor(x))`.
[ForceInline]
public func absFloor<Real: __BuiltinFloatingPointType> (in const x: Real) -> Real {
	return abs(floor(x));//x >= Real(0) ? floor(x) : ceil(-x);
};



//////
//
// Module finalization
//

// Module namespace close
}
