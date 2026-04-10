
//////
//
// Module setup
//

/// Facilities for using billboard geometry.
module billboard;


//////
// Imports

// Shaderlib prelude
import "cgv/prelude.slang";

// Media library
import "cgv/media/geom.slang";


/////
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

	/// A unit quad with no additional vertex attributes.
	public constexpr static PosVec quad[] = {
		PosVec(-_1,-_1, _0, _1), PosVec( _1,-_1, _0, _1), PosVec(-_1, _1, _0, _1), PosVec( _1, _1, _0, _1)
	};

	/// A unit quad with normal facing in the negative z direction.
	public constexpr static media::VertexN<PosScalar> quadN[] = {
		media::VertexN<PosScalar>(PosVec(-_1,-_1, _0, _1), float4(0,0,0,-1)),
		media::VertexN<PosScalar>(PosVec( _1,-_1, _0, _1), float4(0,0,0,-1)),
		media::VertexN<PosScalar>(PosVec(-_1, _1, _0, _1), float4(0,0,0,-1)),
		media::VertexN<PosScalar>(PosVec( _1, _1, _0, _1), float4(0,0,0,-1))
	};
}



//////
//
// Module finalization
//

// Module namespace close
}
