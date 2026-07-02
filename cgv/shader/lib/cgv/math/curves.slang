
//////
//
// Module setup
//

/// A module provide utilities for working with curves, that is, univariate, possibly vector-valued functions.
module curves;


////
// Imports

// Local imports
import "cgv/prelude.slang";
import "cgv/math/lin/common.slang";
import "cgv/math/polynomial.slang";


////
// Module namespace

// Insert into cgv::math namespace
public namespace cgv::math {



//////
//
// Interfaces
//

/// Common interface for all polynomial curves of degree `D`in `N`-dimensional space.
public interface ICurve<Real: __BuiltinFloatingPointType, int N, int D> {
	/// The type of the curve's component polynomials.
	associatedtype ComponentPoly: math::IPolynomial<Real, D>;

	/// Evaluate the curve at parameter `t`.
	public func eval (in const t: Real) -> vector<Real, N>;
}



//////
//
// Structs
//

////
// Sub-namespaces

/// Namespace exposing curve handling facilities.
public namespace curve {

/// A Bézier curve of degree `D` in `N`-dimensional space.
public struct Bezier<Real: __BuiltinFloatingPointType, int N, int D>: ICurve<Real, N, D>
{
	public typealias ComponentPoly = poly::Bezier<Real, D>;

	/// Our *N*-dimensional vector type.
	public typealias Vec = vector<Real, N>;

	/// Our *N*-dimensional coefficient storage matrix for degree `D` curves.
	public typealias Coeffs = matrix<Real, D+1, N>;

	/// The curve control points.
	public var p: Coeffs;

	/// Evaluate the Bézier curve at parameter `t`.
	[ForceInline]
	public func eval (in const t: Real) -> Vec {
		return math::poly::evalBezier(t, p);
	}
};




// Sub-namespace close
}



//////
//
// Module finalization
//

// Module namespace close
}
