
//////
//
// Module setup
//

/// Commonly used assorted functionality for intersection tests.
module common;


////
// Imports

// Shaderlib prelude
import "cgv/prelude.slang";

// Linear algebra module
import "cgv/math/lin/common.slang";
import "cgv/math/lin/inverse.slang";

// Geometry module
import "cgv/geom/billboard.slang";


////
// Module namespace

// Insert into cgv::geom::splat::sphere namespace
public namespace cgv::geom::splat::sphere {

// Import root namespace
using cgv;



//////
//
// Structs
//

/// **S**phere **P**arameter **S**pace or *SPS* for short. **TODO: investigate.**
/// Stupidly 1-to-1 ported from [https://github.com/sgumhold/cgv/blob/master/libs/cgv_gl/glsl/sphere_lib.glvs](https://github.com/sgumhold/cgv/blob/master/libs/cgv_gl/glsl/sphere_lib.glvs), pending an informed and educated *Slang* re-implementation properly suited for *CGV-rs*.
public struct ParameterSpace<Scalar: __BuiltinFloatingPointType>
{
	/// Our 2D vector type.
	typealias Vec2 = cgv::Vec2<Scalar>;

	/// Our 3D vector type.
	typealias Vec3 = cgv::Vec3<Scalar>;

	/// Our 4D vector type.
	typealias Vec4 = cgv::Vec4<Scalar>;

	/// Our linear transformation matrix type.
	typealias Mat3 = cgv::Mat3<Scalar>;

	/// Our affine transformation matrix type.
	typealias Mat4 = cgv::Mat4<Scalar>;

	/// The sphere center in *SPS*.
	public var m_tilde: Vec3;

	/// The *SPS* *x*-axis.
	public var x_tilde: Vec3;

	/// The *SPS* *y*-axis.
	public var y_tilde: Vec3;

	/// The eye point in *SPS*.
	public var e_tilde: Vec3;

	/// Something with the eye point. **TODO: investigate.**
	public var inv_e_vs: Scalar;

	/// Something to do with eye-space normals. **TODO: investigate.**
	public var inv_T_square_e_c_vs: Vec3;

	// Something to do with pre-computing "constant parts of the depth value". **TODO: investigate.**
	public var e_zw_clip_vs: Vec2;

	/// Construct the *SPS* for the given model-view-projection chain.
	/// # Arguments
	/// * `pos_rad` – The position (*xyz*) and radius (*w*) of the sphere to be splatted.
	/// * `MV` – The model-view matrix.
	/// * `iMV` – The inverse model-view matrix.
	/// * `MVP` – The model-view-projection matrix.
	/// * `NM` – The normal matrix.
	/// # Returns
	/// The constructed *SPS*.
	public __init (in const Vec4 pos_rad, in const Mat4 MV, in const Mat4 iMV, in const Mat4 MVP, in const Mat3 NM)
	{
		// Convenience helpers
		constexpr static const Scalar _1 = Scalar(1);

		// Compute radius and reciprocal radius
		let inv_R = _1 / pos_rad.w;

		// Determine eye point in parameter space
		let e = iMV[3].xyz;
		this.e_tilde = inv_R * (e - pos_rad.xyz);

		// Compute helper
		let inv_e_square = _1 / dot(this.e_tilde, this.e_tilde);

		// Determine silhoutte center in parameter space
		this.m_tilde = inv_e_square * this.e_tilde;

		// Determine radius of silhouette in parameter space
		let r = sqrt(_1 - inv_e_square);

		// Compute vector x of length r orthogonal to e in parameter space
		this.x_tilde = r * normalize(cross(iMV[1].xyz, this.e_tilde));

		// compute vector y of length r orthogonal to x and e in parameter space
		this.y_tilde = r * normalize(cross(this.e_tilde, this.x_tilde));

		this.inv_e_vs = Scalar(sqrt(inv_e_square));

		// compute components to compute normal in eye space
		this.inv_T_square_e_c_vs = NM * this.e_tilde;

		// constant part of depth value
		this.e_zw_clip_vs = (MVP * iMV[3]).zw;
	}

	///
	public __init(in const Vec4 pos_rad, in const Mat4 MV, in const Mat4 MVP, in const Mat3 NM) {
		This(pos_rad, MV, MV.inv(), MVP, NM);
	}
}

/// **S**phere **S**plat **I**nfo or *SSI* for short. **TODO: investigate.**
/// Stupidly 1-to-1 ported from [https://github.com/sgumhold/cgv/blob/master/libs/cgv_gl/glsl/sphere_lib.glvs](https://github.com/sgumhold/cgv/blob/master/libs/cgv_gl/glsl/sphere_lib.glgs), pending an informed and educated *Slang* re-implementation properly suited for *CGV-rs*.
public struct SplatInfo<Scalar: __BuiltinFloatingPointType>
{
	/// Our 2D vector type.
	typealias Vec2 = cgv::Vec2<Scalar>;

	/// Our 3D vector type.
	typealias Vec3 = cgv::Vec3<Scalar>;

	public var q_tilde: Vec2;
	public var inv_T_square_v: Vec3;
	public var inv_T_square_e_c: Vec3;
	public var inv_e: Scalar;
	public var e_zw_clip: Vec2;
	public var v_zw_clip: Vec2;
	public var v_eye: Vec3;
};



//////
//
// Functions
//

/// **TODO: investigate.**
/// Stupidly 1-to-1 ported from [https://github.com/sgumhold/cgv/blob/master/libs/cgv_gl/glsl/sphere_lib.glvs](https://github.com/sgumhold/cgv/blob/master/libs/cgv_gl/glsl/sphere_lib.glgs), pending an informed and educated *Slang* re-implementation properly suited for *CGV-rs*.
public func splatVertexWithSPS<Scalar: __BuiltinFloatingPointType> (
	in const pos_rad: cgv::Vec4<Scalar>, in const corner: uint, in sps: ParameterSpace<Scalar>,
	in MV: cgv::Mat4<Scalar>, in MVP: cgv::Mat4<Scalar>, in NM: cgv::Mat3<Scalar>, inout ssi: SplatInfo<Scalar>
) -> cgv::Vec4<Scalar>
{
	// Our vector types
	typealias Vec3 = cgv::Vec3<Scalar>;
	typealias Vec4 = cgv::Vec4<Scalar>;

	// compute the corner point in homogeneous object coordinates
	ssi.q_tilde = geom::Billboards<Scalar>::quad[corner].xy;
	let V_tilde: Vec3 = sps.m_tilde + ssi.q_tilde.x*sps.x_tilde + ssi.q_tilde.y*sps.y_tilde;
	let v_tilde: Vec3 = V_tilde - sps.e_tilde;

	// Vertex output
	var V: Vec4;
	V.xyz = pos_rad.w*V_tilde + pos_rad.xyz;
	V.w = Scalar(1);

	// compute vector from eye to vertex in eye space
	var v4d: Vec4 = MV * V;
	ssi.v_eye = v4d.xyz / v4d.w;

	// compute components to compute normal in eye space
	ssi.inv_T_square_v = NM * (v_tilde);

	//ssi.v = -normalize(ssi.v_eye);
	let clipPos = MVP * V;
	ssi.v_zw_clip = clipPos.zw - ssi.e_zw_clip;
	return clipPos;
}

/// **TODO: investigate.**
public func splatVertex<Scalar: __BuiltinFloatingPointType> (
	in const pos_rad: cgv::Vec4<Scalar>, in const corner: uint, in const modelview: cgv::Mat4<Scalar>,
	in const modelview_inv: cgv::Mat4<Scalar>, in projModelview: cgv::Mat4<Scalar>, in normalMat: cgv::Mat3<Scalar>,
	inout ssi: SplatInfo<Scalar>
) -> cgv::Vec4<Scalar>
{
	let sps = geom::splat::sphere::ParameterSpace<Scalar>(
		pos_rad, modelview,  modelview_inv, projModelview, normalMat
	);
	ssi.inv_e = sps.inv_e_vs;
	ssi.inv_T_square_e_c = sps.inv_T_square_e_c_vs;
	ssi.e_zw_clip = sps.e_zw_clip_vs;
	return splatVertexWithSPS(pos_rad, corner, sps, modelview, projModelview, normalMat, ssi);
}



//////
//
// Module finalization
//

// Module namespace close
}
