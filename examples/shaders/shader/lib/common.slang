
//////
//
// Module setup
//

/// A module for common functionality most other modules depend on.
module common;


/////
// Module namespace

// The common example namespace.
public namespace ex {



//////
//
// Type aliases and enums
//

/// A convenience type alias for generic 2-dimensional vectors.
public typealias Vec2<Number: IArithmetic> = vector<Number, 2>;



//////
//
// Structs
//

/// A struct representing a 2-dimensional axis-aligned bounding box around a central point `(0,0)` as two distinct extents, one each along the *x*- and *y*-axes.
public struct Extents<Number: IArithmetic>
{
	/// The extents along the *x*-axis.
	public Vec2<Number> x;

	/// The extents along the *y*-axis.
	public Vec2<Number> y;

	/// Initializer
	[ForceInline]
	public __init (Vec2<Number> xExt, Vec2<Number> yExt) {
		x = xExt; y = yExt;
	}

	/// Create symmetric extents with the given radii per direction.
	/// # Arguments
	/// * `xRadius` – The radius along the *x*-axis.
	/// * `yRadius` – The radius along the *y*-axis.
	/// # Returns
	/// Symmetric `Extents` with the given radii.
	[ForceInline]
	public static func symmetric (xAxis: Number, yAxis: Number) -> Extents<Number> {
		return Extents<Number>(Vec2<Number>(-xAxis, xAxis), Vec2<Number>(-yAxis, yAxis));
	}

	/// Create bi-symmetric extents, ie. the same radius along all axes.
	/// # Arguments
	/// * `radius` – The symmetric radius to use.
	/// # Returns
	/// Bi-symmetric `Extents` with the given radius.
	[ForceInline]
	public static func biSymmetric (radius: Number) -> Extents<Number> {
		return Extents<Number>(Vec2<Number>(-radius, radius), Vec2<Number>(-radius, radius));
	}
}



//////
//
// Module finalization
//

// Module namespace close
}
