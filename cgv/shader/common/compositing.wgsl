
////
// Streams

struct VertexOutput {
	@builtin(position) pos_clip: vec4f,
	@location(0) uv: vec2f,
};


////
// Vertex shader

// Shader entry point
@vertex
fn vs_main (@builtin(vertex_index) vertexID: u32) -> VertexOutput
{
	var out: VertexOutput;
	if vertexID == 0 {
		out.pos_clip = vec4f(-1, -1, 0, 1);
		out.uv = vec2f(0, 1);
	}
	else if vertexID == 1 {
		out.pos_clip = vec4f(1, -1, 0, 1);
		out.uv = vec2f(1, 1);
	}
	else if vertexID == 2 {
		out.pos_clip = vec4f(-1, 1, 0, 1);
		out.uv = vec2f(0, 0);
	}
	else {
		out.pos_clip = vec4f(1, 1, 0, 1);
		out.uv = vec2f(1, 0);
	}
	return out;
}


////
// Fragment shader

// Uniforms
// - textures
@group(0) @binding(0)
var source: texture_2d<f32>;
@group(0) @binding(1)
var smpler: sampler;

// Shader entry point: non-premultiplied
@fragment
fn fs_non_premultiplied (in: VertexOutput) -> @location(0) vec4f {
	var srcUnpremultiplied = textureSample(source, smpler, in.uv);
	return vec4f(srcUnpremultiplied.rgb * srcUnpremultiplied.a, srcUnpremultiplied.a);
}
