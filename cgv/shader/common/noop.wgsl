
////
// Streams

struct VertexOutput {
	@builtin(position) dummy: vec4f,
};


////
// Vertex shader

// Shader entry point
@vertex
fn vs_main () -> VertexOutput
{
	var out: VertexOutput;
	out.dummy = vec4f(0, 0, 0, 0);
	return out;
}


////
// Fragment shader

// Shader entry point
@fragment
fn fs_main (in: VertexOutput) -> @location(0) vec4f {
	return vec4f(0, 0, 0, 0);
}
