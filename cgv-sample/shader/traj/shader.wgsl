
////
// Streams

struct VertexInput {
	@location(0) pos: vec4f,
	@location(1) color: vec4f,
	@location(2) texcoord: vec2f
};

struct VertexOutput {
	@builtin(position) pos_clip: vec4f,
	@location(0) color: vec4f,
	@location(1) texcoord: vec2f,
};


////
// Vertex shader

// Uniforms
// - viewing uniform group definition
struct Viewing {
	modelview: mat4x4<f32>,
	projection: mat4x4<f32>
}
// - viewing uniform
@group(0) @binding(0)
var<uniform> viewing: Viewing;

// Shader entry point
@vertex
fn vs_main (vert: VertexInput) -> VertexOutput
{
	var out: VertexOutput;
	out.pos_clip = viewing.projection * viewing.modelview * vert.pos;
	out.color = vert.color;
	out.texcoord = vert.texcoord;
	return out;
}


////
// Fragment shader

// Uniforms
// - textures
@group(1) @binding(0)
var tex: texture_2d<f32>;
@group(1) @binding(1)
var smpler: sampler;

// Shader entry point
@fragment
fn fs_main (in: VertexOutput) -> @location(0) vec4f {
	var texColor = textureSample(tex, smpler, in.texcoord);
	return vec4f(mix(in.color.rgb, texColor.rgb, texColor.a), 1);
}
