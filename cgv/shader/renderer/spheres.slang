
//////
//
// Imports
//

// CGV-rs core shader library
import "cgv/prelude.slang";               // enable GLSL-style linear algebra mathematical operators
import "cgv/api/uniforms.slang";          // interface with managed uniforms handled by CGV-rs
import "cgv/math/lin/common.slang";       // enables the cgv::math::lin::IElement interface for all float-based builtins
import "cgv/geom/splat/sphere.slang";     // sphere splatting library
import "cgv/geom/intersect/common.slang"; // test common intersection facilities

// Use CGV namespace by default
using namespace cgv;



//////
//
// Structs
//

/// Representation of a sphere.
struct Sphere
{
	/// The position of the sphere center in camera space.
	var pos_rad: float4;

	/// The color of the sphere.
	var color: float4;

	/// Slang initializer for constructing a new instances of this struct.
	[ForceInline]
	__init (in const float4 pos_rad, in const float4 color) {
		this.pos_rad = pos_rad; this.color = color;
	}
};



//////
//
// Streams
//

////
// Vertex shader input

/// Sphere attributes input stream for the positions/co-located radius plus colors case.
struct VSInput_posRadColor
{
	/// The position of the sphere center in world space, and potentially the radius in the *w*-component.
	[vk::location(0)] var pos_rad: float4;

	/// The color of the sphere.
	[vk::location(1)] var color: float4;
};

/// Sphere attributes input stream for the positions with separate radius case.
struct VSInput_posSepRad
{
	/// The position of the sphere center in world space.
	[vk::location(0)] var pos: float4;

	/// The radius of the sphere.
	[vk::location(1)] var radius: float;
};

/// Sphere attributes input stream for the positions with separate radius and color case.
struct VSInput_posSepRadColor
{
	/// The position of the sphere center in world space.
	[vk::location(0)] var pos: float4;

	/// The radius of the sphere.
	[vk::location(1)] var radius: float;

	/// The color of the sphere.
	[vk::location(2)] var color: float4;
};


////
// Vertex shader output / fragment shader input

/// Splat vertex attributes for the positions-only/co-located radius case.
struct SphereSplat_posRad
{
	/// Clip-space position of the splat vertex.
	float4 pos_clip: SV_Position;

	/// Camera-space position of the sphere center (and optionally radius).
	nointerpolation float4 pos_rad;

	/// Precomputed splat info for the fragment shader.
	//nointerpolation geom::splat::sphere::SplatInfo<float> info;

	/// Slang initializer for constructing a new instances of this struct.
	[ForceInline]
	__init (in const float4 pos_clip, in const float4 pos_rad/*, in const geom::splat::sphere::SplatInfo<float> info*/) {
		this.pos_clip = pos_clip; this.pos_rad = pos_rad; //this.info = info;
	}
};

/// Splat vertex attributes for the positions/co-located radius plus colors case.
struct SphereSplat_posRadColor
{
	/// Clip-space position of the splat vertex.
	float4 pos_clip: SV_Position;

	/// Camera-space position of the sphere center (and optionally radius).
	nointerpolation float4 pos_rad;

	/// The color of the sphere.
	nointerpolation float4 color;

	/// Precomputed splat info for the fragment shader.
	//nointerpolation geom::splat::sphere::SplatInfo<float> info;

	/// Slang initializer for constructing a new instances of this struct.
	[ForceInline]
	__init (
		in const float4 pos_clip, in const float4 pos_rad, in const float4 color/*,
		in const geom::splat::sphere::SplatInfo<float> info*/
	){
		this.pos_clip = pos_clip; this.pos_rad = pos_rad; this.color = color; //this.info = info;
	}
};




//////
//
// Common uniforms
//

/// The viewing parameters. Change often, thus group(0).
[vk::binding(0, 0)] // <- start at binding(0) of group(0)
let viewing: ParameterBlock<cgv::api::Viewing>;

/// The set of constant sphere attributes. Changed infrequently by the user, thus group(1).
struct ConstantAttribs
{
	/// The user-defined default color for all spheres.
	var color: float4;

	/// The user-defined default radius of all spheres.
	var radius: float;
};
[vk::binding(0, 1)] // <- start at binding(0) of group(1)
ParameterBlock<ConstantAttribs> constAttribs;



//////
//
// Vertex shader
//

////
// Entry points

/// Vertex shader entry point for the positions-only case.
[shader("vertex")]
func vertexMain_pos (in pos: float4, uint vertexID: SV_VertexID) -> SphereSplat_posRad
{
	var sphere = Sphere(float4(pos.xyz, constAttribs.radius), constAttribs.color);
	var ssi: geom::splat::sphere::SplatInfo<float>;
	return SphereSplat_posRad(
		/* pos_clip: */ splatSphere(sphere, vertexID, ssi),
		/* pos_rad:  */ sphere.pos_rad,
		/* info:     */// ssi
	);
}

/// Vertex shader entry point for the combined positions/radii case.
[shader("vertex")]
func vertexMain_posRad (in pos_rad: float4, uint vertexID: SV_VertexID) -> SphereSplat_posRad
{
	var sphere = Sphere(pos_rad, constAttribs.color);
	var ssi: geom::splat::sphere::SplatInfo<float>;
	return SphereSplat_posRad(
		/* pos_clip: */ splatSphere(sphere, vertexID, ssi),
		/* pos_rad:  */ sphere.pos_rad,
		/* info:	 */// ssi
	);
}

/// Vertex shader entry point for the separate positions/radii case.
[shader("vertex")]
func vertexMain_posSepRad (in sphere: VSInput_posSepRad, uint vertexID: SV_VertexID) -> SphereSplat_posRad
{
	var sphere = Sphere(float4(sphere.pos.xyz, sphere.radius), constAttribs.color);
	var ssi: geom::splat::sphere::SplatInfo<float>;
	return SphereSplat_posRad(
		/* pos_clip: */ splatSphere(sphere, vertexID, ssi),
		/* pos_rad:  */ sphere.pos_rad,
		/* info:	 */// ssi
	);
}

/// Vertex shader entry point for the positions and colors case.
[shader("vertex")]
func vertexMain_posColor (in sphere: VSInput_posRadColor, uint vertexID: SV_VertexID) -> SphereSplat_posRadColor
{
	var sphere = Sphere(float4(sphere.pos_rad.xyz, constAttribs.radius), sphere.color);
	var ssi: geom::splat::sphere::SplatInfo<float>;
	return SphereSplat_posRadColor(
		/* pos_clip: */ splatSphere(sphere, vertexID, ssi),
		/* pos_rad:  */ sphere.pos_rad,
		/* color:    */ sphere.color,
		/* info:	 */// ssi
	);
}

/// Vertex shader entry point for the combined positions/radii and colors case.
[shader("vertex")]
func vertexMain_posRadColor (in sphere: VSInput_posRadColor, uint vertexID: SV_VertexID) -> SphereSplat_posRadColor
{
	var sphere = Sphere(sphere.pos_rad, sphere.color);
	var ssi: geom::splat::sphere::SplatInfo<float>;
	return SphereSplat_posRadColor(
		/* pos_clip: */ splatSphere(sphere, vertexID, ssi),
		/* pos_rad:  */ sphere.pos_rad,
		/* color:    */ sphere.color,
		/* info:	 */// ssi
	);
}

/// Vertex shader entry point for the separate positions/radii and colors case.
[shader("vertex")]
func vertexMain_posSepRadColor (in sphere: VSInput_posRadColor, uint vertexID: SV_VertexID) -> SphereSplat_posRadColor
{
	var sphere = Sphere(float4(sphere.pos_rad.xyz, constAttribs.radius), constAttribs.color);
	var ssi: geom::splat::sphere::SplatInfo<float>;
	return SphereSplat_posRadColor(
		/* pos_clip: */ splatSphere(sphere, vertexID, ssi),
		/* pos_rad:  */ sphere.pos_rad,
		/* color:    */ sphere.color,
		/* info:	 */// ssi
	);
}


////
// Functions

/// The splatting shading logic.
[ForceInline]
func splatSphere (
	in const sphere: Sphere, in const uint vertexID, out ssi: geom::splat::sphere::SplatInfo<float>
) -> float4 {
	return geom::splat::sphere::splatVertex(
		sphere.pos_rad, vertexID, viewing.modelview, viewing.modelview_inv, viewing.projModelview,
		float3x3(viewing.normal), ssi
	);
}



//////
//
// Fragment shader
//

////
// Entry points

/// Fragment shader entry point for the positions-only case.
[shader("fragment")]
func fragmentMain_pos (splat: SphereSplat_posRad) -> float4 {
	// Delegate to actual shading routine
	return shadeSphere(Sphere(float4(splat.pos_rad.xyz, constAttribs.radius), constAttribs.color)/*, splat.info*/);
}

/// Fragment shader entry point for the positions and radii case.
[shader("fragment")]
func fragmentMain_posRad (splat: SphereSplat_posRad) -> float4 {
	// Delegate to actual shading routine
	return shadeSphere(Sphere(splat.pos_rad, constAttribs.color)/*, splat.info*/);
}

/// Fragment shader entry point for the positions and colors case.
[shader("fragment")]
func fragmentMain_posColor (splat: SphereSplat_posRadColor) -> float4 {
	// Delegate to actual shading routine
	return shadeSphere(Sphere(float4(splat.pos_rad.xyz, constAttribs.radius), splat.color)/*, splat.info*/);
}

/// Fragment shader entry point for the positions, radii and colors case.
[shader("fragment")]
func fragmentMain_posRadColor (splat: SphereSplat_posRadColor) -> float4 {
	// Delegate to actual shading routine
	return shadeSphere(Sphere(splat.pos_rad, splat.color)/*, splat.info*/);
}


////
// Functions

/// The actual shading logic.
func shadeSphere (in const sphere: Sphere/*, in const ssi: geom::splat::sphere::SplatInfo<float>*/) -> float4
{
	// Intersect the sphere
	/*var splat_tc: float2;
	var clipDepth: float;
	let ssi: geom::splat::sphere::SplatInfo<float>;
	let hit = geom::splat::sphere::intersectRay(ssi, splat_tc, clipDepth);
	if (hit == none) {
		discard;
	}*/

	// Just emit unlit default color for whole billboard
	return evaluateHit(/*hit.value, */sphere.color);
}

/// Tests the `IHit` interface.
func evaluateHit (/*in const hit: geom::intersect::IHit<float3>, */in const materialColor: float4) -> float4 {
	return materialColor;
}
