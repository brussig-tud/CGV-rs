
struct VertexInput {
    @location(0) pos: vec4<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
	@builtin(position) pos_clip: vec4<f32>,
	@location(0) color: vec4<f32>,
};

@vertex
fn vs_main (vert: VertexInput) -> VertexOutput
{
	var out: VertexOutput;
	out.color = vert.color;
	out.pos_clip = vert.pos;
	return out;
}

@fragment
fn fs_main (in: VertexOutput) -> @location(0) vec4<f32> {
	return in.color;
}
