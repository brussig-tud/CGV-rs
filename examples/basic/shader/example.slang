
//////
//
// Imports
//

// CGV-rs core shader library
import "cgv/api/uniforms.slang";   // interface with managed uniforms handled by CGV-rs.
import "cgv/lin/operators.slang";  // enable GLSL-style linear algebra mathematical operators
import "cgv/color/blending.slang"; // alpha blending operators
import "cgv/math/field.slang";     // contains the checkerboard pattern



//////
//
// Uniforms
//

// group(0): Viewing parameters. Change often and are thus group(0).
ParameterBlock<cgv::api::Viewing> viewing;

// group(1): Visualization parameters. Are adjustable from the GUI and thus change from time to time, hence group(1).
struct Appearance
{
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
	let checkers = cgv::math::field::Checkers<float, 2>::withInversePeriod(appearance.checkersCount);
	let canvas = lerp(
		/* dark: */appearance.oddCheckersColor*vsOut.color, /* light: */appearance.evenCheckersColor*vsOut.color,
		/* selector: */checkers.queryPreFiltered(vsOut.texcoord, (float mask) => fwidth(mask))
	);

	// Fetch logo texture samples and color them according to user setting
	let logo = lerp(
		appearance.bgColor, appearance.logoColor, /* selector: */texture.color.Sample(texture.sampler, vsOut.texcoord).a
    );

	// Blend logo over the canvas
	return cgv::color::blend::over(canvas, logo);
}
