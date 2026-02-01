
//////
//
// Imports
//

// Interface with managed uniforms handled by CGV-rs.
import "cgv/api/uniforms.slang";

// Enable linear algebra mathematical operators (allows using `*` for matrix multiplication like in GLSL)
import "cgv/lin/operators.slang";



//////
//
// Uniforms
//

// group(0): Viewing parameters.
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

// Vertex shader entry point
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

// Fragment shader entry point
[shader("fragment")]
func fragmentMain (vsOut: VSOutput) -> float4 {
	var shadedColor = float4(1, 1, 0, 1);
	return shadedColor;
}
