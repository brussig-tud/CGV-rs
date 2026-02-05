
//////
//
// Module setup
//

/// A module providing signed-distance field-related (SDF) functionality.
module sdf;


//////
// Imports

// CGV-rs core shader library
import "cgv/common.slang";
import "cgv/geom/common.slang";


/////
// Module namespace

// The common example namespace.
public namespace ex {



//////
//
// Interfaces
//

/// The interface of an arbitrary 2-dimensional *signed-distance field* (SDF).
public interface ISdf<Real: __BuiltinFloatingPointType>
{
	/// Provide (and potentially on-the-fly compute) the tight-fitting axis-aligned bounding box around the SDF's 0-contour.
	/// # Returns
	/// The directional extents.
	func extents () -> cgv::geom::Extents<Real>;

	/// Evaluate the SDF at the given query point `q`.
	/// # Parameters
	/// * `q` – The query point.
	/// # Returns
	/// The value of the distance field at the provided query point.
	func eval (q: vector<Real, 2>) -> Real;
}



//////
//
// Structs
//

////
// Sub-namespaces

/// Namespace providing some pre-defined SDFs.
public namespace sdf {

/// The interface of an arbitrary 2-dimensional *signed-distance field* (SDF).
public struct Circle<Real: __BuiltinFloatingPointType>: ISdf<Real>
{
	/// The radius of the circle
	public Real radius;

	/// Provide (and potentially on-the-fly compute) the tight-fitting axis-aligned bounding box around the SDF's 0-contour.
	/// # Returns
	/// The directional extents.
	public func extents () -> cgv::geom::Extents<Real> {
		return cgv::geom::Extents<Real>::biSymmetric(radius);
	}

	/// Evaluate the SDF at the given query point `q`.
	/// # Parameters
	/// * `q` – The query point.
	/// # Returns
	/// The value of the distance field at the provided query point.
	public func eval (q: cgv::Vec2<Real>) -> Real {
		return length(q) - radius;
	}

	/// Initializer
	[ForceInline]
	public __init (Real radius) {
		this.radius = radius;
	}

	/// Construct the unit circle
	[ForceInline]
	public static func unit () -> Circle<Real> {
		return Circle<Real>(1);
	}

	/// Construct the unit diameter circle (i.e. a radius of .5).
	[ForceInline]
	public static func halfUnit () -> Circle<Real> {
		return Circle<Real>(.5);
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
