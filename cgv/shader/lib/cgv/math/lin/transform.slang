
//////
//
// Module setup
//

// A module providing coordinate system transformation utilities.
module transform;


////
// Imports

// Local imports
import "cgv/prelude.slang";


////
// Module namespace

// Insert into cgv::math::lin namespace
public namespace cgv::math::lin {

// Import parent namespace
using cgv;





//////
//
// Functions
//

// Transform a position with a homogenous matrix, dropping the .w component from the result.
[ForceInline]
public func transformPos<Scalar: __BuiltinFloatingPointType> (m: Mat4<Scalar>, pos: Vec3<Scalar>) -> Vec3<Scalar> {
	return mul(m, Vec4<Scalar>(pos, Scalar(1))).xyz;
}

// Transform a position with a homogenous matrix, keeping the resulting .w component.
[ForceInline]
public func transformPos_w<Scalar: __BuiltinFloatingPointType> (m: Mat4<Scalar>, pos: Vec3<Scalar>) -> Vec4<Scalar> {
	return mul(m, Vec4<Scalar>(pos, Scalar(1)));
}

// Transform a direction with a homogenous matrix, dropping the .w component from the result.
[ForceInline]
public func transformDir<Scalar: __BuiltinFloatingPointType> (m: Mat4<Scalar>, dir: Vec3<Scalar>) -> Vec3<Scalar> {
	return mul(m, Vec4<Scalar>(dir, Scalar(0))).xyz;
}

// Transform a direction with a homogenous matrix, keeping the resulting .w component.
[ForceInline]
public func transformDir_w<Scalar: __BuiltinFloatingPointType> (m: Mat4<Scalar>, dir: Vec3<Scalar>) -> Vec4<Scalar> {
	return mul(m, Vec4<Scalar>(dir, Scalar(0)));
}

// Find some vector that is guaranteed to be orthogonal to the given 3D vector.
public func orthoVec<Scalar: __BuiltinFloatingPointType> (const v: Vec3<Scalar>) -> Vec3<Scalar> {
	let k = fract(abs(v.x) + Scalar(.5));
	return Vec3<Scalar>(-v.y, v.x - k*v.z, k*v.y);
}

// Finds two 3D unit vectors that are orthogonal to each other and the given 3D reference unit vector.
public func make_orthonormal_basis<Scalar: __BuiltinFloatingPointType> (
	out e0: Vec3<Scalar>, out e1: Vec3<Scalar>, in const ref: Vec3<Scalar>
){
	e1 = normalize(orthoVec(ref));
	e0 = cross(e1, ref);
}



//////
//
// Module finalization
//

// Module namespace close
}
