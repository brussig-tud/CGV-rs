
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

/// A collection of billboards.
public struct Billboards<PosScalar: __BuiltinFloatingPointType>
{
	// Private shorthand for PosScalar(0).
	private constexpr static const PosScalar _0 = PosScalar(0);

	// Private shorthand for PosScalar(1).
	private constexpr static const PosScalar _1 = PosScalar(1);

	/// Our position vector type.
	public typealias PosVec = Vec4<PosScalar>;

	/// The *c*-th corner of a unit quad with no additional vertex attributes.
	[ForceInline]
	public constexpr static const PosVec quadCorner (in const uint c) {
		return {PosScalar(((c&1)<<1)-1), PosScalar(((c>>1)<<1)-1), _0, _1};
	}

	/// A unit quad with no additional vertex attributes. If you just need a single corner, consider using [`quadCorner`] instead.
	[ForceInline]
	public constexpr static const PosVec[4] quad() {
		return { quadCorner(0), quadCorner(1), quadCorner(2), quadCorner(3) };
	}


	/// The *c*-th corner of a unit quad with normal facing in the negative z direction.
	[ForceInline]
	public constexpr static const media::VertexN<PosScalar> quadNCorner (in const uint c) {
		 return media::VertexN<PosScalar>(quadCorner(c), float4(0,0,-1,0));
	}

	/// A unit quad with normal facing in the negative z direction. If you just need a single corner, consider using [`quadNCorner`] instead.
	[ForceInline]
	public constexpr static const media::VertexN<PosScalar>[4] quadN () {
		return { quadNCorner(0), quadNCorner(1), quadNCorner(2), quadNCorner(3) };
	}

	/// The *c*-th corner of a unit quad with 2D unit texture coordinates (the *U* refers to the coordinate tuple *uv*).
	[ForceInline]
	public constexpr static const media::VertexU<PosScalar> quadUCorner (in const uint c) {
		constexpr const int2 uv = int2(c&1, c>>1);
		return media::VertexU<PosScalar>(PosVec(PosScalar((uv.x<<1) - 1), PosScalar((uv.y<<1) - 1), _0, _1), float2(uv.x, 1-uv.y));
	}

	/// A unit quad with 2D unit texture coordinates (the *U* refers to the coordinate tuple *uv*). If you just need a single corner, consider using [`quadUCorner`] instead.
	[ForceInline]
	public constexpr static const media::VertexU<PosScalar>[4] quadU () {
		return { quadUCorner(0), quadUCorner(1), quadUCorner(2), quadUCorner(3) };
	}
}



//////
//
// Module finalization
//

// Module namespace close
}
