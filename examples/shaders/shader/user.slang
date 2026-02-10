
// CGV-rs core shader lib
import "cgv/geom/common.slang";

// Local imports
import "lib/sdf.slang";
import "lib/glyph.slang";

/// A custom circle glyph.
public struct MyCircle: ex::IGlyph
{
	/// Our `Self`.
	public typealias Self = MyCircle;

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
		return MyCircle(.5, .125);
	}

	/// Initializer
	[ForceInline]
	public __init (float radius, float smear) {
		this.circle = ex::sdf::Circle<float>(radius);
		this.smear = smear;
		this.invSmear = 1/smear;
	}
}

/// Bind to our specialization point
export struct Glyph: ex::IGlyph = MyCircle;
