
//////
//
// Imports
//

// CGV-rs core shader library
import "cgv/api/uniforms.slang";          // interface with managed uniforms handled by CGV-rs
import "cgv/math/lin/common.slang";       // enables the cgv::math::lin::IElement interface for all float-based builtins
import "cgv/math/lin/operators.slang";    // enable GLSL-style linear algebra mathematical operators
import "cgv/geom/billboard.slang";        // access the pre-defined billboards
import "cgv/geom/intersect/common.slang"; // test common intersection facilities

// Use CGV namespace by default
using namespace cgv;



//////
//
// Streams
//

/* no custom structs - for now we just use builtin primitives (float4 in our case). */



//////
//
// Common uniforms
//

/// The viewing parameters. Change often, thus group(0).
[vk::binding(0, 0)] // <- start at binding(0) of group(0)
let viewing: ParameterBlock<cgv::api::Viewing>;

/// The set of constant sphere attributes. Changed infrequently by the user, thus group(1).
struct ConstantAttribs {
	/// The user-defined default color for all spheres.
	var radius: float;

	/// The user-defined default radius of all spheres.
	var color: float4;
};
[vk::binding(0, 1)] // <- start at binding(0) of group(1)
ParameterBlock<ConstantAttribs> constAttribs;



//////
//
// Vertex shader
//

////
// Entry points

/// Common vertex shader entry point for all sphere renderer code paths.
[shader("vertex")]
float4 vertexMain_posOnly (uint vertexID: SV_VertexID) : SV_Position {
	return viewing.projection*viewing.modelview * geom::Billboards<float>::quad[vertexID];
}



//////
//
// Fragment shader
//

////
// Helper functions

/// Tests the `IHit` interface.
func evaluateHit (hit: geom::intersect::IHit<float3>) -> float4 {
	return constAttribs.color * float4(hit.point, 1.);
}


////
// Entry points

/// Our sole fragment shader entry point.
[shader("fragment")]
func fragmentMain_posOnly (pos: float4) -> float4
{
	// Test intersect and billboard module
	var hit: geom::intersect::Hit<float3>;
	hit.point = geom::Billboards<float>::quad[1].xyz + pos.xyz / pos.w;
	hit.l = pos.w;

	// Just emit unlit default color for whole billboard
	return evaluateHit(hit);
}
