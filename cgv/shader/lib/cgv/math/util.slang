
//////
//
// Module definition
//

// A module providing small assorted mathematical utilities.
module util;


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

/// Check if the given integer value is a power of 2.
/// # Arguments
/// `val` – The value to check.
/// # Returns
/// `true` if `val` is a power of 2, `false` otherwise.
[ForceInline]
public func isPow2<T: __BuiltinIntegerType> (in const val: T) -> bool {
	return (val & (val - T(1))) == T(0);
};

/// Checks for is  if the given integer value is a power of 2.
/// # Arguments
/// `val` – The value to check.
/// # Returns
/// `true` if `val` is a power of 2, `false` otherwise.
public func isPow2<T: __BuiltinIntegerType, int N> (in const val: vector<T, N>) -> vector<bool, N> {
	vector<bool, N> result;
	[ForceUnroll]
	for (int d=0; d<N; ++d)
		result[d] = isPow2(val[d]);
	return result;
};



//////
//
// Module finalization
//

// Module namespace close
}
