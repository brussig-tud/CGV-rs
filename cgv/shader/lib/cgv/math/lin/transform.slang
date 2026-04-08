
//////
//
// Module setup
//

// A module providing coordinate system transformation utilities.
module transform;


//////
// Imports

// Local imports
/* nothing here yet */


/////
// Module namespace

// Insert into cgv::math::lin namespace
public namespace cgv::math::lin {



//////
//
// Functions
//

// Transform a position with a homogenous matrix, dropping the .w component from the result.
public func transformPos (m: float4x4, pos: float3) -> float3 {
	return mul(m, float4(pos, 1)).xyz;
}

// Transform a position with a homogenous matrix, keeping the resulting .w component.
public func transformPos_w (m: float4x4, pos: float3) -> float4 {
	return mul(m, float4(pos, 1));
}

// Transform a direction with a homogenous matrix, dropping the .w component from the result.
public func transformDir (m: float4x4, dir: float3) -> float3 {
	return mul(m, float4(dir, 0)).xyz;
}

// Transform a direction with a homogenous matrix, keeping the resulting .w component.
public func transformDir_w (m: float4x4, dir: float3) -> float4 {
	return mul(m, float4(dir, 0));
}



//////
//
// Module finalization
//

// Module namespace close
}
