
//////
//
// Module definition
//

// A module providing utilities for representing an manipulating (scalar-)fields
module field;


//////
// Imports

// Local imports
import "cgv/common.slang";


/////
// Module namespace

// Insert into cgv::math namespace
namespace cgv::math {



//////
//
// Interfaces
//

///
public interface IField<Real: __BuiltinFloatingPointType, int Dims>
{
	///
	public func query (in q: vector<Real, Dims>)->Real;

	///
	public func queryPreFiltered (in q: vector<Real, Dims>, filter: IFunc<Real, Real>) -> Real {
		return query(q);
	}
}
typealias IField1<Real: __BuiltinFloatingPointType> = IField<Real, 1>;
typealias IField2<Real: __BuiltinFloatingPointType> = IField<Real, 2>;
typealias IField3<Real: __BuiltinFloatingPointType> = IField<Real, 3>;
typealias IField1f = IField1<float>;
typealias IField2f = IField2<float>;
typealias IField3f = IField3<float>;

///
public interface IPeriodicField<Real : __BuiltinFloatingPointType, int Dims> : IField<Real, Dims>
{
	///
	associatedtype Self: IPeriodicField<Real, Dims>;

	///
	public static func withPeriod (in const period: vector<Real, Dims>) -> Self;

	///
	public static func withInversePeriod (in const invPeriod: vector<Real, Dims>) -> Self {
		return withPeriod(Real(1)/invPeriod);
	}
}
typealias IPeriodicField1<Real: __BuiltinFloatingPointType> = IPeriodicField<Real, 1>;
typealias IPeriodicField2<Real: __BuiltinFloatingPointType> = IPeriodicField<Real, 2>;
typealias IPeriodicField3<Real: __BuiltinFloatingPointType> = IPeriodicField<Real, 3>;
typealias IPeriodicField1f = IPeriodicField1<float>;
typealias IPeriodicField2f = IPeriodicField2<float>;
typealias IPeriodicField3f = IPeriodicField3<float>;



//////
//
// Structs
//

////
// Sub-namespaces

/// Namespace providing some pre-defined SDFs.
public namespace field {

/// Helper function to evaluate a unit checkerboard mask (unit means each cell is a hypercube of unit side length).
func checkerboard<Real : __BuiltinFloatingPointType, uint Dims> (in q: vector<Real, Dims>) -> Real
{
	// Calculate if inside a "black" square by counting the number of odd components. The logic behind it is that to be
	// in a "black" cell on a discrete binary grid, an odd number of components in the grid index must be odd(1), the
	// others must be even(0). For example (crude 3-dimensional visualization):
	//
	//      (0,0,1)black ———— (1,0,1)white
	//        ⁄|                   ⁄|             This generalizes to arbitrary odd and even integers, not just 0
	//      ⁄  |                 ⁄  |             and 1. The key insight is that we can quickly find out if an
	//    ⁄    |               ⁄    |             integer is odd or even by taking it modulo 2.
	// (0,0,0)white ———— (1,0,0)black
	//   |     |              |     |             This implies we can find out for arbitrary N-dimensional integer
	//   |     |              |     |             indices if they refer to "black" or "white" cells by just summing
	//   |  (0,1,1)white ———— |1,1,1)black        their components and checking if the sum modulo 2 evaluates to
	//   |    ⁄               |    ⁄              zero or one!
	//   |  ⁄                 |  ⁄
	//   |⁄                   |⁄
	// (0,1,0)black ———— (1,1,0)white

	// Determine black/white
	var sum = abs(floor(q[0])); // <- have to integer-ize the components for this to work (see explanation above)
	[ForceUnroll]
	for (uint c=1; c<Dims; ++c)
		sum += abs(floor(q[c]));

	// Done! Black or white is decided by sum being odd or even (i.e. modulo 2)
	return sum % Real(2);
}

/// A field assigning to each point in space the signed distance to an alternating grid of hypercuboids, in the 2D case commonly referred to as a checkerboard. *Odd* cells are considered to be on the "inside" (i.e. negative distances), while *even* cells are "outside".
public struct CheckersSDF<Real : __BuiltinFloatingPointType, uint Dims> : IPeriodicField<Real, Dims>
{
	public typealias Self = CheckersSDF<Real, Dims>;
	public typealias Vec = vector<Real, Dims>;

	public Vec invPeriod;

	override static func withPeriod (in const period: vector<Real, Dims>) -> Self {
		return withInversePeriod(Real(1)/period);
	}

	override static func withInversePeriod (in const invPeriod: vector<Real, Dims>) -> Self {
		return CheckersSDF<Real, Dims>(invPeriod);
	}

	override func query (in q: Vec) -> Real
	{
		// Scale query inversely to period
		let q = q*invPeriod;

		// Determine if "odd" or "even"
		let odd = checkerboard(q) > Real(0);

		// Determine distance to cell border
		let fr = fract(q);
		var minDist = cgv::Constants<Real>::inf;
		[ForceUnroll]
		for (uint c=0; c<Dims; ++c) {
			let dist = fr[c] < Real(.5) ? fr[c] : Real(1)-fr[c];
			minDist = min(dist, minDist);
		}

		// Turn into signed distance to odd cells (i.e. even is positive)
		return odd ? -minDist : minDist;
	}

	__init (Vec invPeriod) {
		this.invPeriod = invPeriod;
	}
}

/// A field that assigns a value of 1 or 0 to every point depending on whether it is in an *odd* or *even* cell,
/// respectively, of a checkerboard pattern.
public struct Checkers<Real : __BuiltinFloatingPointType, uint Dims> : IPeriodicField<Real, Dims>
{
	public typealias Self = Checkers<Real, Dims>;
	public typealias Vec = vector<Real, Dims>;

	CheckersSDF<Real, Dims> sdf;
	property invPeriod : Vec {
		get { return sdf.invPeriod; }
		set { sdf.invPeriod = newValue; }
	}

	override static func withPeriod (in const period: vector<Real, Dims>) -> Self {
		return withInversePeriod(Real(1)/period);
	}

	override static func withInversePeriod (in const invPeriod: vector<Real, Dims>) -> Self {
		return Checkers<Real, Dims>(invPeriod);
	}

	override func query (in q: Vec) -> Real {
		// Scale query inversely to period
		return checkerboard(q*invPeriod);
	}

	override func queryPreFiltered (in q: Vec, filter: IFunc<Real, Real>) -> Real
	{
		// Query checkerboard SDF as basis for pre-filtering
		let dist = sdf.query(q);

		// Pre-filter the 0<->1 transition
		let radius = Real(.5) * filter(dist);
		let mask = smoothstep(-radius, radius, dist);

		// Done!
		return mask;
	}

	__init (Vec invPeriod) {
		this.sdf = CheckersSDF<Real, Dims>(invPeriod);
	}
}

// Sub-namespace close
}



//////
//
// Module finalization
//

// Module namespace close
}
