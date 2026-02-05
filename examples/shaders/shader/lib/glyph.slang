
//////
//
// Module setup
//

/// A module providing functionality for defining 2-dimensional glyphs.
module glyph;


//////
// Imports

// CGV-rs core shader library
import "cgv/common.slang";
import "cgv/geom/common.slang";

// Local imports
import "lib/sdf.slang";


/////
// Module namespace

// The common example namespace
public namespace ex {



//////
//
// Interfaces
//

/// The interface of an arbitrary 2D glyph.
public interface IGlyph
{
    /// The concrete type of the implementer, need for the `default` constructor.
    associatedtype Self: IGlyph;

	/// Provide (and potentially on-the-fly compute) the tight-fitting axis-aligned bounding box around the glyph.
	/// # Returns
	/// The directional extents.
	func extents () -> cgv::geom::Extents<float>;

	/// Query the glyph color at the given query point `q`.
	/// # Arguments
	/// * `q` – The query point.
	/// # Returns
	/// The RGBA color of the glyph at the given query point.
	func colorAt (q: float2) -> float4;

	/// Create a default instance of the glyph.
	/// # Returns
	/// A valid glyph instance, its exact properties depend on the implementation.
	public static Self default () ;
}



//////
//
// Structs
//

////
// Sub-namespaces

/// Namespace providing some pre-defined glyphs.
public namespace glyphs {

/// A simple circle glyph.
public struct Circle: IGlyph
{
	/// Our `Self`.
	public typealias Self = Circle;

	/// The underlying circle SDF.
	ex::sdf::Circle<float> circle;

	/// The "smear" radius outside the circle.
	float smear, invSmear;

	/// Provide (and potentially on-the-fly compute) the tight-fitting axis-aligned bounding box around the glyph.
	/// # Returns
	/// The directional extents.
	public func extents () -> cgv::geom::Extents<float> {
		let smearOffset = float2(-smear, smear);
		let circleExtents = circle.extents();
		return cgv::geom::Extents<float>(circleExtents.x+smearOffset, circleExtents.y+smearOffset);
	}

	/// Query the glyph color at the given query point `q`.
	/// # Parameters
	/// * `q` – The query point.
	/// # Returns
	/// The RGBA color of the glyph at the given query point.
	public func colorAt (q: float2) -> float4 {
		const float sd = circle.eval(q),
		            val = 1-clamp(abs(sd*invSmear), 0, 1),
		            val_clamped = clamp(val, 0, 1);
		return float4(val_clamped, val_clamped, 1-val_clamped, 1);
	}

	/// Create a default instance of the glyph.
	/// # Returns
	/// A valid glyph instance, its exact properties depend on the implementation.
	[ForceInline]
	public static Self default () {
		return Circle(.5, .125);
	}

	/// Initializer
	[ForceInline]
	public __init (float radius, float smear) {
		this.circle = ex::sdf::Circle<float>(radius);
		this.smear = smear;
		this.invSmear = 1/smear;
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
