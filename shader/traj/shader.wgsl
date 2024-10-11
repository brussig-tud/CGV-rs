// Vertex shader

struct VertexOutput {
	@builtin(position) pos_clip: vec4<f32>,
};

@vertex
fn vs_main (@builtin(vertex_index) idx: u32) -> VertexOutput
{
	var out: VertexOutput;
	let x = f32(1 - i32(idx)) * .5;
	let y = f32(i32(idx & 1u)*2 - 1) * .5;
	out.pos_clip = vec4<f32>(x, y, .0, 1.);
	return out;
}

@fragment
fn fs_main (in: VertexOutput) -> @location(0) vec4<f32> {
	return vec4<f32>(.7, .5, .3, 1.);
}
