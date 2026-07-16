
//////
//
// Module setup
//

/// A module provide utilities for working with polynomials.
module polynomial;


////
// Imports

// Local imports
import "cgv/prelude.slang";
import "cgv/math/functions.slang";
import "cgv/math/lin/common.slang";
import "cgv/meta.slang";


////
// Module namespace

// Insert into cgv::math namespace
public namespace cgv::math {



//////
//
// Interfaces
//

/// Common interface for all polynomial representations.
public interface IPolynomial<Real: __BuiltinFloatingPointType, uint D>
{
	/// The polynomial coefficients.
	property coeffs : vector<Real, D+1>;

	/// The degree of the polynomial.
	constexpr func degree () -> uint {
		return D;
	}

	/// Evaluate the polynomial at the given point.
	/// # Arguments
	/// * `x` – The point at which to evaluate the polynomial.
	/// # Returns
	/// The value of the polynomial at the given point.
	func eval (in const x: Real) -> Real;

	/// Call operator, evaluates the polynomial at the given point.
	[ForceInline]
	Real operator() (in const Real x) {
		return eval(x);
	}
}

/// Common interface for all polynomials that can be transformed to the monomial basis.
public interface IToMonomial<Real : __BuiltinFloatingPointType, uint D> : IPolynomial<Real, D>
{
	/// The type of the polynomial after transforming to the monomial basis.
	associatedtype MonomialType: IPolynomial<Real, D>;

	/// Transform the polynomial to the monomial basis.
	/// # Returns
	/// The polynomial in the monomial basis.
	//func toMonomial () -> MonomialType;
}

/// Common interface for all polynomials that can be transformed to the Bézier basis.
public interface IToBezier<Real : __BuiltinFloatingPointType, uint D> : IPolynomial<Real, D>
{
	/// The type of the polynomial after transforming to the Bézier basis.
	associatedtype BezierType: IPolynomial<Real, D>;

	/// Transform the polynomial to the Bézier basis.
	/// # Returns
	/// The polynomial in the Bézier basis.
	//func toBezier () -> BezierType;
}

/// Common interface for all polynomials that can be transformed to the Hermite basis.
public interface IToHermite<Real : __BuiltinFloatingPointType> : IPolynomial<Real, 3>
{
	/// The type of the polynomial after transforming to the Hermite basis.
	associatedtype HermiteType : IPolynomial<Real, 3>;

	/// Transform the polynomial to the Hermite basis.
	/// # Returns
	/// The polynomial in the Hermite basis.
	//func toHermite  () -> HermiteType;
}



//////
//
// Structs
//

////
// Sub-namespaces

/// Namespace providing some pre-defined polynomial representations as well as utility functions.
public namespace poly {

/// Collection of basis transformation matrices between common polynomial bases of a given degree (excluding the Hermite basis as that is defined for degree 3 only).
public struct BasisTrans<Real: __BuiltinFloatingPointType, uint D>
{
	////
	// Exported constants

	/// The number of rows/columns in the (always square) transformation matrices.
	public constexpr static const int N = D+1;


	////
	// Private helpers

	/// Loop through the given row vector of a to-be-filled M2B matrix via compile-time recursion
	[ForceInline]
	private constexpr static const void m2b_forRow<int col, int curRow> (inout matrix<Real,N,N> M) {
		if (curRow < N) {
			M[col][curRow] = m2b_entry<col, curRow>();
			m2b_forRow<col, curRow+1>(M);
		}
	}

	/// Loop through the column vectors of a to-be-filled M2B matrix via compile-time recursion
	[ForceInline]
	private constexpr static const void m2b_forCol<int curCol> (inout matrix<Real,N,N> M) {
		if (curCol < N) {
			m2b_forRow<curCol, 0>(M);
			m2b_forCol<curCol+1>(M);
		}
	}

	/// Loop through the given row vector of a to-be-filled B2M matrix via compile-time recursion
	[ForceInline]
	private constexpr static const void b2m_forRow<int col, int curRow>(inout matrix<Real, N, N> M) {
		if (curRow < N) {
			M[col][curRow] = b2m_entry<col, curRow>();
			b2m_forRow<col, curRow+1>(M);
		}
	}

	/// Loop through the column vectors of a to-be-filled B2M matrix via compile-time recursion
	[ForceInline]
	private constexpr static const void b2m_forCol<int curCol>(inout matrix<Real, N, N> M) {
		if (curCol < N) {
			b2m_forRow<curCol, 0>(M);
			b2m_forCol<curCol+1>(M);
		}
	}

	/// Returns the (*row*, *col*)-th entry of the monomial-to-Bézier basis transformation matrix for degree `D` polynomials.
	[ForceInline]
	private constexpr static const func m2b_entry<int col, int row>() -> Real {
		// Note reversal of column order `D-col` compared to typical variants of the formula in the literature, since we
		// are given monomial coefficients in descending order of degree.
		static constexpr const int coeff = D-col;
		return row < coeff ? Real(0) : Real(meta::binomial<row, coeff>()) / Real(meta::binomial<D, coeff>());
	}

	/// Returns the (*row*, *col*)-th entry of the Bézier-to-monomial basis transformation matrix for degree `D` polynomials.
	[ForceInline]
	private constexpr static const func b2m_entry<int col, int row>() -> Real {
		// Note reversal of row order `D-row` compared to typical variants of the formula in the literature, since we
		// will store the resulting monomial coefficients in descending order of degree.
		static constexpr const int coeff = D-row;
		return coeff < col ? Real(0) : Real((((coeff-col) & 1) ? -1 : 1) * meta::binomial<D, coeff>()*meta::binomial<coeff, col>());
	}


	////
	// Methods

	/// Returns the (*row*, *col*)-th entry of the monomial-to-Bézier basis transformation matrix for degree `D` polynomials.
	[ForceInline]
	public constexpr static const func m2b_entry (in const col: int, in const row: int) -> Real {
		// Note reversal of column order `D-col` compared to typical variants of the formula in the literature, since we
		// are given monomial coefficients in descending order of degree.
		let coeff = D-col;
		return row < coeff ? Real(0) : Real(binomial(row, coeff)) / Real(binomial(D, coeff));
	}

	/// Returns the (*row*, *col*)-th entry of the Bézier-to-monomial basis transformation matrix for degree `D` polynomials.
	[ForceInline]
	public constexpr static const func b2m_entry (in const col: int, in const row: int) -> Real {
		// Note reversal of row order `D-row` compared to typical variants of the formula in the literature, since we
		// will store the resulting monomial coefficients in descending order of degree.
		let coeff = D-row;
		return coeff < col ? Real(0) : Real((((coeff-col) & 1) ? -1 : 1) * binomial(D, coeff)*binomial(coeff, col));
	}

	/// Return the monomial to Bézier basis transformation matrix for the given polynomial degree.
	[ForceInline]
	public constexpr static const func m2b() -> matrix<Real, D+1, D+1> {
		var M: matrix<Real, N, N>;
		m2b_forCol<0>(M);
		return M;
	}

	/// Return the Bézier to monomial basis transformation matrix for the given polynomial degree.
	[ForceInline]
	public constexpr static const func b2m() -> matrix<Real, D+1, D+1> {
		var M: matrix<Real, N, N>;
		b2m_forCol<0>(M);
		return M;
	}
};

/// Collection of basis transformation matrices between common polynomial bases.
public struct BasisTransFixed<Real: __BuiltinFloatingPointType>
{
	////
	// Scalar constants

	// Private shorthand for Real(0).
	private constexpr static const Real _0 = Real(0);

	// Private shorthand for Real(1).
	private constexpr static const Real _1 = Real(1);

	// Private shorthand for Real(2).
	private constexpr static const Real _2 = Real(2);

	// Private shorthand for Real(3).
	private constexpr static const Real _3 = Real(3);

	// Private shorthand for Real(6).
	private constexpr static const Real _6 = Real(6);

	// Private shorthand for Real(.5).
	private constexpr static const Real _1o2 = Real(.5);

	// Private shorthand for Real(1/3.).
	private constexpr static const Real _1o3 = Real(1/3.);

	// Private shorthand for Real(2/3.).
	private constexpr static const Real _2o3 = Real(2/3.);


	////
	// Monomial -> Bézier

	/// The transformation matrix from monomial to Bézier basis for linear polynomials.
	///
	/// **DEPRECATED**: Use `BasisTrans<Real, /*degree:*/1>::m2b()` instead.
	public constexpr static const Mat2<Real> M2B_1 = Mat2<Real>(
		Vec2<Real>( _0,  _1),
		Vec2<Real>( _1,  _1)
	);

	/// The transformation matrix from monomial to Bézier basis for quadratic polynomials.
	///
	/// **DEPRECATED**: Use `BasisTrans<Real, /*degree:*/2>::m2b()` instead.
	public constexpr static const Mat3<Real> M2B_2 = Mat3<Real>(
		Vec3<Real>( _0,  _0,   _1),
		Vec3<Real>( _0,  _1o2, _1),
		Vec3<Real>( _1,  _1,   _1)
	);

	/// The transformation matrix from monomial to Bézier basis for cubic polynomials.
	///
	/// **DEPRECATED**: Use `BasisTrans<Real, /*degree:*/3>::m2b()` instead.
	public constexpr static const Mat4<Real> M2B_3 = Mat4<Real>(
		Vec4<Real>( _0,   _0,   _0,   _1),
		Vec4<Real>( _0,   _0,   _1o3, _1),
		Vec4<Real>( _0,   _1o3, _2o3, _1),
		Vec4<Real>( _1,   _1,   _1,   _1)
	);


	////
	// Bézier -> Monomial

	/// The transformation matrix from Bézier to monomial basis for linear polynomials.
	///
	/// **DEPRECATED**: Use `BasisTrans<Real, /*degree:*/1>::b2m()` instead.
	public constexpr static const Mat2<Real> B2M_1 = Mat2<Real>(
		Vec2<Real>(-_1,  _1),
		Vec2<Real>( _1,  _0)
	);

	/// The transformation matrix from Bézier to monomial basis for quadratic polynomials.
	///
	/// **DEPRECATED**: Use `BasisTrans<Real, /*degree:*/2>::b2m()` instead.
	public constexpr static const Mat3<Real> B2M_2 = Mat3<Real>(
		Vec3<Real>( _1,  -_2,   _1),
		Vec3<Real>(-_2,   _2,   _0),
		Vec3<Real>( _1,   _0,   _0)
	);

	/// The transformation matrix from Bézier to monomial basis for cubic polynomials.
	///
	/// **DEPRECATED**: Use `BasisTrans<Real, /*degree:*/3>::b2m()` instead.
	public constexpr static const Mat4<Real> B2M_3 = Mat4<Real>(
		Vec4<Real>(-_1,   _3,  -_3,   _1),
		Vec4<Real>( _3,  -_6,   _3,   _0),
		Vec4<Real>(-_3,   _3,   _0,   _0),
		Vec4<Real>( _1,   _0,   _0,   _0)
	);


	////
	// Bézier <-> Hermite

	/// The transformation matrix from Bézier to Hermite basis (cubic polynomials only).
	public constexpr static const Mat4<Real> B2H = Mat4<Real>(
		Vec4<Real>( _1,   _0,   _0,   _0),
		Vec4<Real>(-_3,   _3,   _0,   _0),
		Vec4<Real>( _0,   _0,  -_3,   _3),
		Vec4<Real>( _0,   _0,   _0,   _1)
	);

	/// The transformation matrix from Hermite to Bézier basis (cubic polynomials only).
	public constexpr static const Mat4<Real> H2B = Mat4<Real>(
		Vec4<Real>( _1,   _0,   _0,   _0),
		Vec4<Real>( _1,   _1o3, _0,   _0),
		Vec4<Real>( _0,   _0,  -_1o3, _1),
		Vec4<Real>( _0,   _0,   _0,   _1)
	);
}

/// A polynomial of degree `D` that is represented by a vector of coefficients for the monomial basis.
/// The coefficients are stored in descending order of degree, that is, `coeffs[0]` is the coefficient of the highest degree term, and `coeffs[D]` is the constant term.
///
/// This coefficient ordering is chosen for performance reasons as it directly mirrors the evaluation order of terms in Horner's method.
public struct Monomial<Real: __BuiltinFloatingPointType, uint D> : IPolynomial<Real, D>, IToMonomial<Real, D>, IToBezier<Real, D>
{
	public typealias MonomialType = This;
	public typealias BezierType = Bezier<Real, D>;

	public var coeffs: vector<Real, D+1>;

	[ForceInline]
	override func eval (in const x: Real) -> Real {
		return evalMonomial(x, coeffs);
	}

	/*[ForceInline]
	override func toMonomial () -> MonomialType {
		return this;
	}

	[ForceInline]
	override func toBezier () -> BezierType {
		return BezierType(BasisTrans<Real>.M2B_D(coeffs));
	}*/

	[ForceInline]
	__init (vector<Real, D+1> coeffs) {
		this.coeffs = coeffs;
	}
}

/// A polynomial of degree `D` that is represented by a vector of coefficients for the Bézier basis.
public struct Bezier<Real: __BuiltinFloatingPointType, uint D> : IPolynomial<Real, D>
{
	public var coeffs: vector<Real, D+1>;

	override func eval (in const x: Real) -> Real {
		return evalBezier(x, coeffs);
	}

	[ForceInline]
	__init (vector<Real, D+1> coeffs) {
		this.coeffs = coeffs;
	}
}

// Sub-namespace close
}



//////
//
// Functions
//

////
// Sub-namespaces

/// Namespace providing some pre-defined polynomial representations as well as utility functions.
namespace poly {

/// Evaluate the polynomial given by the provided Monomial basis coefficients at a given `x`. Uses Horner's method internally.
/// # Arguments
/// * `x` – The point at which to evaluate the polynomial.
/// * `coeffs` – The monomial basis coefficients of the polynomial.
/// # Returns
/// The value of the polynomial at the given point.
public func evalMonomial<Elem: math::lin::IElement, int Dplus1, Coeffs: math::lin::IElementSeq<Elem, Dplus1>> (
	in const x: Elem::Repr, in const coeffs: Coeffs
) -> Elem
{
	let x = Elem::splat(x);
	var y = coeffs[0];
	[ForceUnroll]
	for (int i=1; i<Dplus1; ++i)
		y = y.fma(x, coeffs[i]); // y = y*x + coeffs[i];
	return y;
}

/// Evaluate the polynomial given by the provided Bézier basis coefficients at a given `x`. Uses De-Casteljau's algorithm internally. Consider transforming the Bézier coefficients to monomial basis for more efficient evaluation (if the same polynomial needs to be evaluated repeatedly).
/// # Arguments
/// * `x` – The point at which to evaluate the polynomial.
/// * `coeffs` – The Bézier basis coefficients of the polynomial.
/// # Returns
/// The value of the polynomial at the given point.
public func evalBezier<Elem: math::lin::IElement, int Dplus1, Coeffs: math::lin::IElementSeq<Elem, Dplus1>> (
	in const Elem::Repr x, in Coeffs coeffs
) -> Elem
{
	// Outer loop: successively reduce the degree of the polynomial by one
	[ForceUnroll]
	for (int l=1; l<Dplus1; ++l)
	{
		// Inner loop: evaluate the interpolated control points of the next lower degree.
		[ForceUnroll]
		for (int i=0; i<Dplus1-l; ++i) {
			coeffs[i] = coeffs[i].lerp(coeffs[i+1], x);
		}
	}
	// Final function value will sit in the very first coefficient after reduction to constant (degree 0) is complete.
	return coeffs[0];
}

/// Derive the polynomial given by the provided monomial basis coefficients.
/// # Arguments
/// * `coeffs` – The coefficients of the polynomial to derive.
/// # Returns
/// The coefficients of the 1st derivative of the input polynomial.
public func deriveMonomial<
	Elem: math::lin::IElement, int Dplus1, Coeffs: math::lin::IElementSeq<Elem, Dplus1>,
	CoeffsDer: math::lin::IElementSeq<Elem, Dplus1-1>
>(
	in const Elem::Repr x, in const coeffs: Coeffs
) -> CoeffsDer
{
	constexpr static const int D = Dplus1-1;
	var result: CoeffsDer;
	[ForceUnroll]
	for (int l=0; l<D; ++l)
		result[l] = coeffs[l].scale(D-l);
	return result;
}

/// Derive the polynomial given by the provided Bézier basis coefficients.
/// # Arguments
/// * `coeffs` – The Bézier basis coefficients of the polynomial to derive.
/// # Returns
/// The Bézier basis coefficients of the 1st derivative of the input polynomial.
public func deriveBezier<
	Elem: math::lin::IElement, int Dplus1, Coeffs: math::lin::IElementSeq<Elem, Dplus1>,
	CoeffsDer: math::lin::IElementSeq<Elem, Dplus1-1>
>(
	in const Elem::Repr x, in const coeffs: Coeffs
) -> CoeffsDer
{
	constexpr static const int D = Dplus1-1;
	var result: CoeffsDer;
	[ForceUnroll]
	for (int l=0; l<D; ++l)
		result[l] = (coeffs[l+1] - coeffs[l]).scale(D);
	return result;
}

/// Solve the linear equation *p*(*x*) = *ax* + *b* = 0.
/// # Arguments
/// * `linear` – The linear polynomial to solve.
/// # Returns
/// The root of the linear polynomial *x* = -*b*/*a*, if it exists. If the polynomial is constant (i.e., *a*=0), the function will return `NaN`.
public func solveLinear<Real: __BuiltinFloatingPointType, Poly: IPolynomial<Real, 1>> (in const Poly linear)
-> Real {
	return -linear.coeffs[1] / linear.coeffs[0];
}

/// Evaluate the real solution(s) *x* = (-*b* ± √*D*) / (2*a*) (where *D* = *b*² - 4*ac*) to the quadratic equation with precomputed terms.
/// # Arguments
/// * `negb` – The value of `-b`.
/// * `_2a` – The value of `2a`.
/// * `D` – The value of `D`.
/// * `_2c` – The value of `_2c`.
/// # Returns
/// The real solution(s) to the quadratic equation. The returned vector contains the number of real solutions in its first component, followed by the solutions themselves.
public func evalQuadraticRoots<Real: __BuiltinFloatingPointType> (
	const Real negb, const Real _2a, const Real D, const Real _2c
) -> Vec3<Real>
{
	if (_2a != Real(0))
	{
		// Quadratic
		if (D > Real(0)) {
			Real sqrtD = sqrt(D);
			return Vec3<Real>(Real(2), (negb-sqrtD)/_2a, (negb+sqrtD)/_2a);
		}
		else if (D == Real(0))
			return Vec3<Real>(Real(1), negb/_2a, Real(0));
	}
	else if (negb != Real(0))
		// Linear
		return Vec3<Real>(Real(1), _2c/(negb+negb), Real(0));

	// No solution
	return Vec3<Real>(Real(0));
}

/// Solve the quadratic equation *p*(*x*) = *ax*² + *bx* + *c* = 0.
/// # Arguments
/// `quadratic` – The quadratic polynomial to solve.
/// # Returns
/// Up to two real roots of the quadratic polynomial, if they exist. The returned vector contains the number of real roots in its first component, followed by the roots themselves.
[ForceInline]
public func solveQuadratic<Real: __BuiltinFloatingPointType> (in const Monomial<Real, 2> linear)
-> vector<Real, 3> {
	let _2a = linear.coeffs[0]+linear.coeffs[0];
	let _2c = linear.coeffs[2]+linear.coeffs[2];
	return evalQuadraticRoots(
		/* negb */-linear.coeffs[1],   _2a,   /* D: */linear.coeffs[1]*linear.coeffs[1] - _2a*_2c,    _2c
	);
}

// Sub-namespace close
}



//////
//
// Module finalization
//

// Module namespace close
}
