
//////
//
// Imports
//

// CGV-rs core shader library
import "cgv/api/uniforms.slang";   // interface with managed uniforms handled by CGV-rs
import "cgv/lin/operators.slang";  // enable GLSL-style linear algebra mathematical operators
import "cgv/color/blending.slang"; // alpha blending operators
import "cgv/math/field.slang";     // contains the checkerboard pattern



//////
//
// Streams
//

// Vertex shader input attributes.
struct VSInput
{
	/// Model-space position of the vertex.
	float4 pos;

	/// Base color of the vertex.
	float4 color;

	/// Texture coordinates of the vertex.
	float2 texcoord;
}

// Vertex shader output -> fragment shader input attributes.
struct VSOutput
{
	/// Clip-space position of the vertex.
	float4 pos_clip : SV_Position;

	/// Same as `VSInput::color`.
	float4 color;

	/// Same as `VSInput::texcoord`.
	float2 texcoord;

	/// Slang initializer for constructing new instances of this struct.
	[ForceInline]
	__init (float4 pos_clip, float4 color, float2 texcoord) {
		this.pos_clip = pos_clip; this.color = color; this.texcoord = texcoord;
	}
}



//////
//
// Common uniforms
//

/// The viewing parameters. Change often, thus group(0).
[vk::binding(0, 0)] // <- start at binding(0) of group(0)
ParameterBlock<cgv::api::Viewing> viewing;



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
		/* color:    */ vert.color,
		/* texcoord: */ vert.texcoord
	);
}



//////
//
// Fragment shader
//

////
// Uniforms

/// Data layout of the visualization parameters uniform block.
struct Appearance
{
	/// The user-defined color of the logo shape (which can be transparent, thus revealing the checkerboard pattern).
	float4 logoColor;

	/// The user-defined background color (which can also be transparent, thus revealing the checkerboard pattern).
	float4 bgColor;

	/// The user-defined color of odd checkers in the pattern.
	float4 oddCheckersColor;

	/// The user-defined color of even checkers in the pattern.
	float4 evenCheckersColor;

	/// The user-defined number of checkers in each dimension across the quad.
	float2 checkersCount;
};
/// Visualization parameters. Adjustable from the GUI and thus change from time to time, hence group(1).
[vk::binding(0, 1)] // <- start at binding(0) of group(1)
ParameterBlock<Appearance> appearance;

/// Data layout of the uniform block providing texturing information (just color currently).
struct Texture {
	Texture2D color;
	SamplerState sampler;
};
/// The texture to shader with. Won't change, thus group(2).
[vk::binding(0, 2)] // <- start at binding(0) of group(2)
ParameterBlock<Texture> texture;


////
// Entry points

/// Our sole fragment shader entry point.
[shader("fragment")]
func fragmentMain (vsOut: VSOutput) -> float4
{
	// Checkerboard background
	let checkers = cgv::math::field::Checkers<float, /* dimensions: */2>::withInversePeriod(appearance.checkersCount);
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
