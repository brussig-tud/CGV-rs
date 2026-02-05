
//////
//
// Imports
//

// Interface with managed uniforms handled by CGV-rs.
import "cgv/api/uniforms.slang";

// Enable linear algebra mathematical operators (allows using `*` for matrix multiplication like in GLSL)
import "cgv/lin/operators.slang";

// Pull in convenience helpers
import "cgv/common.slang";
import "cgv/math/field.slang";



//////
//
// Uniforms
//

// group(0): Viewing parameters. Change often and are thus group(0).
ParameterBlock<cgv::api::Viewing> viewing;

// group(1): Visualization parameters. Are adjustable from the GUI and thus change from time to time, hence group(1).
struct Colors {
	float4 logoColor;
	float4 bgColor;
	float4 checkerColor_dark;
	float4 checkerColor_light;
};
ParameterBlock<Colors> colors;

// group(2): Material parameters (just a texture and its sampler in this simple example). Won't change, thus group(2).
struct Texture {
	Texture2D color;
	SamplerState sampler;
};
ParameterBlock<Texture> texture;



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
func fragmentMain (vsOut: VSOutput) -> float4
{
	// Checkerboard background
	let checkers = cgv::math::field::Checkers<float, 2>::withInversePeriod(float2(16));
	let bgColor = lerp(
		/* dark: */colors.checkerColor_dark*vsOut.color, /* light: */colors.checkerColor_light*vsOut.color,
		/* selector: */checkers.queryPreFiltered(vsOut.texcoord, (float mask) => fwidth(mask))
	);

	// Fetch and color texture samples
	let texSample = texture.color.Sample(texture.sampler, vsOut.texcoord);
	let texColor = lerp(colors.bgColor, colors.logoColor*texSample, texSample.a);

	// Mix solid part of color according to texture alpha
	return float4(lerp(bgColor.rgb, texColor.rgb, texColor.a), bgColor.a);
}
