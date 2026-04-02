
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

// Vertex shader input attributes.
struct VSInput {
	float4 pos;
	float2 texcoord;
}

// Vertex shader output attributes.
struct VSOutput
{
	float4 pos_clip : SV_Position;
	float2 texcoord;

	__init (float4 _pos_clip, float2 _texcoord) {
		pos_clip = _pos_clip; texcoord = _texcoord;
	}
}



//////
//
// Vertex shader
//

////
// Entry points

/// Our sole vertex shader entry point.
[shader("vertex")]
func vertexMain (vert: VSInput) -> VSOutput {
	return VSOutput(
		/* pos_clip: */ viewing.projection*viewing.modelview * vert.pos,
		/* texcoord: */ vert.texcoord
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
func fragmentMain (fsIn: VSOutput) -> float4 {
	let glyph = Glyph::default();
	let shadedColor = glyph.colorAt(fsIn.texcoord);
	return shadedColor;
}
