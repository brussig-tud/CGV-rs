
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
[ForceInline]
public func orthoVec<Scalar: __BuiltinFloatingPointType> (const v: Vec3<Scalar>) -> Vec3<Scalar> {
	let k = fract(abs(v.x) + Scalar(.5));
	return Vec3<Scalar>(-v.y, v.x - k*v.z, k*v.y);
}

// Find two 3D unit vectors that are orthogonal to each other and the given 3D reference unit vector.
[ForceInline]
public func makeOrthonormalBasis<Scalar: __BuiltinFloatingPointType> (
	out e0: Vec3<Scalar>, out e1: Vec3<Scalar>, in const ref: Vec3<Scalar>
){
	e1 = normalize(orthoVec(ref));
	e0 = cross(e1, ref);
}

// Finds two 3D unit vectors that are orthogonal to each other and the given 3D reference unit vector.
[ForceInline]
public func makeOrthonormalBasis<Scalar: __BuiltinFloatingPointType> (in const ref: Vec3<Scalar>) -> Mat2x3<Scalar> {
	let e1 = normalize(orthoVec(ref));
	return Mat2x3<Scalar>(e1, cross(e1, ref));
}

// Creates a system transformation into an oriented space spanned by the given 3D reference unit vector as x-axis and two to-be-calculated 3D unit vectors orthogonal to it as y- and z-axes.
public func makeOrthonormalSystemFromX<Scalar: __BuiltinFloatingPointType> (in const refX: Vec3<Scalar>) -> Mat3<Scalar>
{
	let e1 = normalize(orthoVec(refX));
	let e0 = cross(e1, refX);
	return Mat3<Scalar>(
		Vec3<Scalar>(refX.x, e0.x, e1.x), Vec3<Scalar>(refX.y, e0.y, e1.y), Vec3<Scalar>(refX.z, e0.z, e1.z)
	);
}

// Creates a system transformation into an oriented space spanned by the given 3D reference unit vector as *x*-axis and two to-be-calculated 3D unit vectors orthogonal to it as *y*- and *z*-axes.
[ForceInline]
public func makeOrthonormalSystemFromX<Scalar: __BuiltinFloatingPointType> (
	out M: Mat3<Scalar>, in const refX: Vec3<Scalar>
){
	M = makeOrthonormalSystemFromX(refX);
}

// Creates a system transformation into an oriented space spanned by the given 3D reference unit vector as *y*-axis and two to-be-calculated 3D unit vectors orthogonal to it as *x*- and *z*-axes.
public func makeOrthonormalSystemFromY<Scalar: __BuiltinFloatingPointType> (in const refY: Vec3<Scalar>) -> Mat3<Scalar>
{
	let e1 = normalize(orthoVec(refY));
	let e0 = cross(e1, refY);
	return Mat3<Scalar>(
		Vec3<Scalar>(e0.x, refY.x, e1.x), Vec3<Scalar>(e0.y, refY.y, e1.y), Vec3<Scalar>(e0.z, refY.z, e1.z)
	);
}

// Creates a system transformation into an oriented space spanned by the given 3D reference unit vector as *y*-axis and two to-be-calculated 3D unit vectors orthogonal to it as *x*- and *z*-axes.
[ForceInline]
public func makeOrthonormalSystemFromY<Scalar: __BuiltinFloatingPointType> (
	out M: Mat3<Scalar>, in const refY: Vec3<Scalar>
){
	M = makeOrthonormalSystemFromY(refY);
}

// Creates a system transformation into an oriented space spanned by the given 3D reference unit vector as *z*-axis and two to-be-calculated 3D unit vectors orthogonal to it as *x*- and *y*-axes.
public func makeOrthonormalSystemFromZ<Scalar: __BuiltinFloatingPointType> (in const refZ: Vec3<Scalar>) -> Mat3<Scalar>
{
	let e1 = normalize(orthoVec(refZ));
	let e0 = cross(e1, refZ);
	return Mat3<Scalar>(
		Vec3<Scalar>(e0.x, e1.x, refZ.x), Vec3<Scalar>(e0.y, e1.y, refZ.y), Vec3<Scalar>(e0.z, e1.z, refZ.z)
	);
}

// Creates a system transformation into an oriented space spanned by the given 3D reference unit vector as *z*-axis and two to-be-calculated 3D unit vectors orthogonal to it as *x*- and *y*-axes.
[ForceInline]
public func makeOrthonormalSystemFromZ<Scalar: __BuiltinFloatingPointType> (
	out M: Mat3<Scalar>, in const refZ: Vec3<Scalar>
){
	M = makeOrthonormalSystemFromY(refZ);
}

// Calculates a system transformation into the space defined by the given origin and basis vectors.
[ForceInline]
public func makeLocalFrame<Scalar: __BuiltinFloatingPointType> (
	in const x: Vec3<Scalar>, in const y: Vec3<Scalar>, in const z: Vec3<Scalar>, const o: Vec3<Scalar>
) -> Mat4<Scalar> {
	return Mat4<Scalar>(
		Vec4<Scalar>(x.x, y.x, z.x, Scalar(0)), Vec4<Scalar>(x.y, y.y, z.y, Scalar(0)),
		Vec4<Scalar>(x.z, y.z, z.z, Scalar(0)), Vec4<Scalar>(-dot(x, o), -dot(y, o), -dot(z, o), Scalar(1))
	);
}

// Calculates a system transformation into the space defined by the given origin and basis vectors.
[ForceInline]
public func makeLocalFrame<Scalar: __BuiltinFloatingPointType> (
	out M: Mat4<Scalar>, in const x: Vec3<Scalar>, in const y: Vec3<Scalar>, in const z: Vec3<Scalar>,
	const o: Vec3<Scalar>
){
	M = makeLocalFrame(x, y, z, o);
}



//////
//
// Module finalization
//

// Module namespace close
}
