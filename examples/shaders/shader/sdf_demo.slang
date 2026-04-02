
//////
//
// Imports
//

// CGV-rs core shader library
import "cgv/api/uniforms.slang";   // interface with managed uniforms handled by CGV-rs
import "cgv/lin/operators.slang";  // enable GLSL-style linear algebra mathematical operators

// Local imports
import "lib/sdf.slang";
import "lib/glyph.slang";



//////
//
// Common uniforms
//

// Viewing parameters.
[vk::binding(0, 0)] // <- start at binding(0) of group(0)
ParameterBlock<cgv::api::Viewing> viewing;



//////
//
// Streams
//

// The vertex attributes of our target quad.
struct Vertex
{
	float4 pos : SV_Position;
	float2 texcoord;

	__init (float4 pos, float2 texcoord) {
		this.pos = pos; this.texcoord = texcoord;
	}
}



//////
//
// Constants
//

/// Our quad vertices in model space.
constexpr static let qVerts: Vertex[] = {
	Vertex(float4(-1.,-1., 0., 1.), float2(-1., 1.)),  Vertex(float4(1.,-1., 0., 1.), float2(1., 1.)),
	Vertex(float4(-1., 1., 0., 1.), float2(-1.,-1.)),  Vertex(float4(1., 1., 0., 1.), float2(1.,-1.))
};



//////
//
// Vertex shader
//

////
// Entry points

/// Our sole vertex shader entry point.
[shader("vertex")]
func vertexMain (int vertexID: SV_VertexID) -> Vertex {
	return Vertex(
		/*      pos: */ viewing.projection*viewing.modelview * qVerts[vertexID].pos,
		/* texcoord: */ qVerts[vertexID].texcoord
	);
}



//////
//
// Fragment shader
//

////
// Specialization points

/// The Glyph type to draw.
extern struct Glyph: ex::IGlyph;


////
// Entry points

/// Our sole fragment shader entry point.
[shader("fragment")]
func fragmentMain (fsInput: Vertex) -> float4 {
	let glyph = Glyph::default(); // construct user glyph
	return glyph.colorAt(fsInput.texcoord); // shade according to user glyph
}
