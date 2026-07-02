
//////
//
// Module setup
//

/// Facilities for using billboard geometry.
module billboard;


////
// Imports

// Shaderlib prelude
import "cgv/prelude.slang";

// Media library
import "cgv/media/geom.slang";


////
// Module namespace

// Insert into cgv namespace
public namespace cgv::geom {

// Import parent namespace
using cgv;



//////
//
// Structs
//

/// An axis-aligned box in `N`-dimensional space.
public struct AABox<Real: __BuiltinFloatingPointType, int N>
{
	/// Our vector type.
	public typealias Vec = vector<Real, N>;

	/// The minimum corner of the box.
	public var pmin: Vec;

	/// The maximum corner of the box.
	public var pmax: Vec;

	/// Construct an empty (invalid) box ready for receiving points.
	[ForceInline]
	public static func invalid () -> This {
		return This(Constants<Real>::inf, Constants<Real>::negInf);
	}

	/// Construct an empty (invalid) box ready for receiving points.
	[ForceInline][mutating]
	public func invalidate () {
		this.pmin = Constants<Real>::inf;
		this.pmax = Constants<Real>::negInf;
	}

	/// Add minimal amount of volume to the box such that the given point is covered.
	/// # Arguments
	/// * `point` – The point to include in the box.
	[ForceInline][mutating]
	public func add (in const Vec point) {
		this.pmin = min(this.pmin, point);
		this.pmax = max(this.pmax, point);
	}

	/// Compute the center of the box.
	[ForceInline]
	public func center () -> Vec {
		return (this.pmin+this.pmax)*Real(.5);
	}

	/// Compute the extent of the box.
	[ForceInline]
	public func extent () -> Vec {
		return this.pmax - this.pmin;
	}

	/// Compute the half-extent of the box.
	[ForceInline]
	public func halfExtent () -> Vec {
		return Real(.5)*extent();
	}

	/// Find the projection interval of the given axis-aligned box onto the given direction, returning an ordered pair of scalars corresponding to the projections of the frontmost and backmost (relative to the given direction) points on the box boundary, respectively.
	/// # Arguments
	/// * `dir` – The direction to project onto.
	/// # Returns
	/// The projection interval between the frontmost and backmost points on the box boundary.
	public func projectionOnDir (const Vec dir) -> Vec2<Real>
	{
		// Preamble
		let he = halfExtent();
		let center = center();
		var center_p = dot(dir, center);

		// Compute radius by adding the contributions of each half-extent vector when projected onto the direction.
		var radius = he[0]*abs(dir[0]);
		[ForceUnroll]
		for (int i=1; i<N; ++i)
		      radius += he[i]*abs(dir[i]);

		// Build interval
		return Vec2<Real>(center_p-radius, center_p+radius);
	}
}



//////
//
// Module finalization
//

// Module namespace close
}
