
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

/// Checks if the given integers is divisible by 2.
/// # Arguments
/// `val` – The value to check.
/// # Returns
/// `true`if the number was divisible by 2, `false` otherwise.
[ForceInline]
public func isDiv2<T: __BuiltinIntegerType> (in const val: T) -> bool {
	return (val & T(1)) == T(0);
};

/// Checks for each component of the given vector of integers if they are divisible by 2.
/// # Arguments
/// `val` – The vector to check.
/// # Returns
/// A boolean vector containing `true` at each component that was divisible by 2, and `false` at the others.
public func isDiv2<T: __BuiltinIntegerType, int N> (in const val: vector<T, N>) -> vector<bool, N> {
	vector<bool, N> result;
	[ForceUnroll]
	for (int d=0; d<N; ++d)
		result[d] = isDiv2(val[d]);
	return result;
};

/// Checks for each component of the given vector of integers if they are either the given number or divisible by 2.
/// # Arguments
/// `val` – The vector to check.
/// `num` – The special number that will override the divisibility check.
/// # Returns
/// A boolean vector containing `true` at each component that was divisible by 2 or the given number, and `false` at the others.
public func isNumOrDiv2<T: __BuiltinIntegerType, int N> (in const val: vector<T, N>, in const num: T)
	-> vector<bool, N>
{
	vector<bool, N> result;
	[ForceUnroll]
	for (int d=0; d<N; ++d)
		result[d] = val[d]==num || isDiv2(val[d]);
	return result;
};

/// Checks for each component of the given vector of integers if they are either the given compile-time number or divisible by 2.
/// # Arguments
/// `Num` – The special (compile-time) number that will override the divisibility check.
/// `val` – The vector to check.
/// # Returns
/// A boolean vector containing `true` at each component that was divisible by 2 or the given number, and `false` at the others.
public func isNumOrDiv2<int Num, T: __BuiltinIntegerType, int N> (in const val: vector<T, N>)
	-> vector<bool, N>
{
	vector<bool, N> result;
	[ForceUnroll]
	for (int d=0; d<N; ++d)
		result[d] = val[d]==T(Num) || isDiv2(val[d]);
	return result;
};

/// Check if the given integer value is a power of 2.
/// # Arguments
/// `val` – The value to check.
/// # Returns
/// `true` if `val` is a power of 2, `false` otherwise.
[ForceInline]
public func isPow2<T: __BuiltinIntegerType> (in const val: T) -> bool {
	return (val & (val-T(1))) == T(0);
};

/// Checks for each component of the given vector of integers if they are a power of 2.
/// # Arguments
/// `val` – The vector to check.
/// # Returns
/// A boolean vector containing `true` at each component that was a power of 2, and `false` at the others.
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
