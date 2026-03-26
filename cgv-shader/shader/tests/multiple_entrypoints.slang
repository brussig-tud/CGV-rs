
//////
//
// Imports
//

// Enable GLSL-style linear algebra mathematical operators
import "lib/operators.slang";



//////
//
// Uniforms
//

// group(0): Viewing parameters. Change often and are thus group(0).
struct Viewing {
	float4x4 projection;
	float4x4 modelview;
}
ParameterBlock<Viewing> viewing;

// group(1): Visualization parameters. Are adjustable from the GUI and thus change from time to time, hence group(1).
struct Appearance {
	float4 logoColor;
	float4 bgColor;
	float4 oddCheckersColor;
	float4 evenCheckersColor;
	float2 checkersCount;
};
ParameterBlock<Appearance> appearance;

// group(2): Material parameters (just a texture and its sampler in this simple example). Won't change, thus group(2).
struct Texture {
	Texture2D color;
	SamplerState sampler;
};
ParameterBlock<Texture> texture;

// group(3): Compute shader output. Won't change unless `texture` changes, so group(3).
[format("rgba8")]
ParameterBlock<WTexture2D> outputTexture;



//////
//
// Streams
//

// Vertex shader input attributes.
struct VSInput {
	float4 pos;
	float4 color;
	float2 texcoord;
}

// Vertex shader output attributes.
struct VSOutput
{
	float4 pos_clip : SV_Position;
	float4 color;
	float2 texcoord;

	__init (float4 _pos_clip, float4 _color, float2 _texcoord) {
		pos_clip = _pos_clip; color = _color; texcoord = _texcoord;
	}
}



//////
//
// Functions
//

func uvwFromIndex (int2 index) -> float2 {
	var width: float; var height: float;
	outputTexture.GetDimensions(width, height);
	return float2(index) / float2(width, height);
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
		/* color:    */ vert.color,
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
	// Determine output color by sampling input texture
	return texture.color.Sample(texture.sampler, vsOut.texcoord);
}



//////
//
// Compute shader
//

// Compute shader variant 1
[shader("compute")]
[numthreads(8, 8, 1)]
func computeMain1 (int3 id : SV_DispatchThreadID) {
	let uv = uvwFromIndex(id.xy);
	outputTexture.Store(id.xy, texture.color.Sample(texture.sampler, uv));
}

// Compute shader variant 2
[shader("compute")]
[numthreads(16, 16, 1)]
func computeMain2 (int3 id : SV_DispatchThreadID) {
	let uv = uvwFromIndex(id.xy);
	outputTexture.Store(id.xy, float4(uv.x, uv.y, 0, 1));
}
