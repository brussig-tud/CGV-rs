
//////
//
// Imports
//

// CGV-rs core shader library
import "cgv/prelude.slang";               // enable GLSL-style linear algebra mathematical operators
import "cgv/api/uniforms.slang";          // interface with managed uniforms handled by CGV-rs
import "cgv/math/lin/common.slang";       // enables the cgv::math::lin::IElement interface for all float-based builtins
import "cgv/geom/splat/hermite.slang";    // Hermite spline splatting library
import "cgv/geom/intersect/common.slang"; // test common intersection facilities

// Use CGV namespace by default
using namespace cgv;



//////
//
// Structs
//

/// Representation of a Hermite node.
struct HermiteNode: geom::splat::hermite::INode<float>
{
	/// The position of the sphere center in camera space.
	var pos_rad: float4;

	/// The color of the sphere.
	var tan_drad: float4;

	/// Slang initializer for constructing a new instances of this struct.
	[ForceInline]
	__init (in const float4 pos_rad, in const float4 tan_drad) {
		this.pos_rad = pos_rad; this.tan_drad = tan_drad;
	}
};

/// Representation of a Hermite curve.
struct HermiteCurve: geom::splat::IHermiteCurve<float, HermiteNode>
{
	/// The start node of the curve.
	var n0: HermiteNode;

	/// The color of the sphere.
	var n1: HermiteNode;

	/// Slang initializer for constructing a new instances of this struct.
	[ForceInline]
	__init (in const HermiteNode n0, in const HermiteNode n1) {
		this.n0 = n0; this.n1 = n1;
	}
};
/*



//////
//
// Streams
//

////
// Vertex shader input

/// Sphere attributes input stream for the positions/co-located radius plus colors case.
struct VSInput_posRadColor {
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
struct SphereSplat_posRad: geom::splat::sphere::ISplatInfo<float>
{
	/// Clip-space position of the splat vertex.
	float4 pos_clip: SV_Position;

	/// Camera-space position of the sphere center (and optionally radius).
	nointerpolation float4 pos_rad;

	/// Precomputed splat info for the fragment shader.
	var q_tilde: float2;
	var inv_T_square_v: float3;
	var inv_T_square_e_c: float3;
	var inv_e: float;
	var e_zw_clip: float2;
	var v_zw_clip: float2;
	var v_eye: float3;
};

/// Splat vertex attributes for the positions/co-located radius plus colors case.
struct SphereSplat_posRadColor: geom::splat::sphere::ISplatInfo<float>
{
	/// Clip-space position of the splat vertex.
	float4 pos_clip: SV_Position;

	/// Camera-space position of the sphere center (and optionally radius).
	nointerpolation float4 pos_rad;

	/// The color of the sphere.
	nointerpolation float4 color;

	/// Precomputed splat info for the fragment shader.
	var q_tilde: float2;
	var inv_T_square_v: float3;
	var inv_T_square_e_c: float3;
	var inv_e: float;
	var e_zw_clip: float2;
	var v_zw_clip: float2;
	var v_eye: float3;
};


////
// Fragment shader output

struct FSOutput
{
	/// The final fragment color.
	float4 color: SV_Target;

	/// The fragment depth corrected for the splatting result.
	float  depth: SV_Depth;

	/// Slang initializer.
	[ForceInline]
	__init (in const float4 color, in const float depth) {
		this.color = color; this.depth = depth;
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
struct Style {
	/// The user-defined default color for all spheres.
	var color: float4;

	/// The user-defined default radius of all spheres.
	var radius: float;

    /// The user-defined global radius multiplier.
    var radiusScale: float;
};
[vk::binding(0, 1)] // <- start at binding(0) of group(1)
ParameterBlock<Style> style;



//////
//
// Vertex shader
//

////
// Entry points

*/
[shader("vertex")]
func vertexMain (uint vertexID: SV_VertexID) -> float4 {
	return float4(0,0,0,1);
}/*
/// Vertex shader entry point for the positions-only case.
[shader("vertex")]
func vertexMain_pos (in pos: float4, uint vertexID: SV_VertexID) -> SphereSplat_posRad {
	var sphere = Sphere(float4(pos.xyz, style.radius), style.color);
	var vertex: SphereSplat_posRad;
	vertex.pos_clip = splatSphere(sphere, vertexID, vertex);
	vertex.pos_rad = sphere.pos_rad;
	return vertex;
}

/// Vertex shader entry point for the combined positions/radii case.
[shader("vertex")]
func vertexMain_posRad (in pos_rad: float4, uint vertexID: SV_VertexID) -> SphereSplat_posRad {
	var sphere = Sphere(pos_rad, style.color);
	var vertex: SphereSplat_posRad;
	vertex.pos_clip = splatSphere(sphere, vertexID, vertex);
	vertex.pos_rad = sphere.pos_rad;
	return vertex;
}

/// Vertex shader entry point for the separate positions/radii case.
[shader("vertex")]
func vertexMain_posSepRad (in sphere: VSInput_posSepRad, uint vertexID: SV_VertexID) -> SphereSplat_posRad {
	var sphere = Sphere(float4(sphere.pos.xyz, sphere.radius), style.color);
	var vertex: SphereSplat_posRad;
	vertex.pos_clip = splatSphere(sphere, vertexID, vertex);
	vertex.pos_rad = sphere.pos_rad;
	return vertex;
}

/// Vertex shader entry point for the positions and colors case.
[shader("vertex")]
func vertexMain_posColor (in sphere: VSInput_posRadColor, uint vertexID: SV_VertexID) -> SphereSplat_posRadColor
{
	var sphere = Sphere(float4(sphere.pos_rad.xyz, style.radius), sphere.color);
	var vertex: SphereSplat_posRadColor;
	vertex.pos_clip = splatSphere(sphere, vertexID, vertex);
	vertex.pos_rad = sphere.pos_rad;
	vertex.color = sphere.color;
	return vertex;
}

/// Vertex shader entry point for the combined positions/radii and colors case.
[shader("vertex")]
func vertexMain_posRadColor (in sphere: VSInput_posRadColor, uint vertexID: SV_VertexID) -> SphereSplat_posRadColor
{
	var sphere = Sphere(sphere.pos_rad, sphere.color);
	var vertex: SphereSplat_posRadColor;
	vertex.pos_clip = splatSphere(sphere, vertexID, vertex);
	vertex.pos_rad = sphere.pos_rad;
	vertex.color = sphere.color;
	return vertex;
}

/// Vertex shader entry point for the separate positions/radii and colors case.
[shader("vertex")]
func vertexMain_posSepRadColor (in sphere: VSInput_posRadColor, uint vertexID: SV_VertexID) -> SphereSplat_posRadColor
{
	var sphere = Sphere(float4(sphere.pos_rad.xyz, style.radius), style.color);
	var vertex: SphereSplat_posRadColor;
	vertex.pos_clip = splatSphere(sphere, vertexID, vertex);
	vertex.pos_rad = sphere.pos_rad;
	vertex.color = sphere.color;
	return vertex;
}


////
// Functions

/// The splatting shading logic.
[ForceInline]
func splatSphere<Sphere: geom::splat::ISphere<float>, SplatInfo: geom::splat::sphere::ISplatInfo<float>> (
	in const sphere: Sphere, in const uint vertexID, out ssi: SplatInfo
) -> float4 {
	return geom::splat::sphere::splatVertex(
		float4(sphere.pos_rad.xyz, sphere.pos_rad.w*style.radiusScale), vertexID, viewing.modelview,
		viewing.modelview_inv, viewing.projModelview, float3x3(viewing.normal), ssi
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
func fragmentMain_pos (splat: SphereSplat_posRad) -> FSOutput {
	// Delegate to actual shading routine
	return shadeSphere(Sphere(float4(splat.pos_rad.xyz, style.radius*style.radiusScale), style.color), splat);
}

/// Fragment shader entry point for the positions and radii case.
[shader("fragment")]
func fragmentMain_posRad (splat: SphereSplat_posRad) -> FSOutput {
	// Delegate to actual shading routine
	return shadeSphere(Sphere(splat.pos_rad, style.color), splat);
}

/// Fragment shader entry point for the positions and colors case.
[shader("fragment")]
func fragmentMain_posColor (splat: SphereSplat_posRadColor) -> FSOutput {
	// Delegate to actual shading routine
	return shadeSphere(Sphere(float4(splat.pos_rad.xyz, style.radius*style.radiusScale), splat.color), splat);
}

/// Fragment shader entry point for the positions, radii and colors case.
[shader("fragment")]
func fragmentMain_posRadColor (splat: SphereSplat_posRadColor) -> FSOutput {
	// Delegate to actual shading routine
	return shadeSphere(Sphere(splat.pos_rad, splat.color), splat);
}


////
// Functions

/// The actual shading logic.
func shadeSphere<SplatInfo: geom::splat::sphere::ISplatInfo<float>> (in const sphere: Sphere, in const ssi: SplatInfo)
	-> FSOutput
{
	// Intersect the sphere
	var splat_tc: float2; // <- not currently used
	var clipDepth: float;
	let hit = geom::splat::sphere::intersectRay(ssi, splat_tc, clipDepth);
	if (hit == none) {
		discard;
	}

	// Shade according to sphere surface hit
	return FSOutput(evaluateHit(hit.value, sphere.color), clipDepth);
}

/// Tests the `IHit` interface.
func evaluateHit (in const hit: geom::intersect::IHit<float3>, in const materialColor: float4) -> float4
{
	// TODO: implement proper shading that integrates with managed lighting and material system, once it exists
	// - define a point light slightly to the left above the camera
	let dirLight = normalize(float3(-.75, 1, 0) - hit.point);
	let lightColor = float3(.5, .5, .5);
	let ambient = 0.0625;
	let shininess = 32.;
	// - compute diffuse terms
	let diffuse = max(0., dot(hit.normal, dirLight));
	// - compute Blinn-Phong specular terms
	let dirEye = normalize(-hit.point);
	let halfDir = normalize(dirLight + dirEye);
	let specular = pow(max(0., dot(hit.normal, halfDir)), shininess);
	// - shade
	let litColor = float4(materialColor.rgb * lightColor*(diffuse+specular+ambient), materialColor.a);

	// Done!
	return litColor;
}
*/
