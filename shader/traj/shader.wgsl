
////
// Interface

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

@vertex
fn vs_main (vert: VertexInput) -> VertexOutput
{
	var out: VertexOutput;
	out.pos_clip = vert.pos;
	out.color = vert.color;
	out.texcoord = vert.texcoord;
	return out;
}


////
// Fragment shader

@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(0) @binding(1)
var smpler: sampler;

@fragment
fn fs_main (in: VertexOutput) -> @location(0) vec4f {
    var texColor = textureSample(tex, smpler, in.texcoord);
	return vec4f(mix(in.color.rgb, texColor.rgb, texColor.a), 1);
}
