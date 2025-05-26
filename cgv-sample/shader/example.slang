
////
// Uniforms

struct Viewing {
	float4x4 modelview;
	float4x4 projection;
}
ParameterBlock<Viewing> viewing;

struct Texture {
	Texture2D color;
	SamplerState sampler;
};
ParameterBlock<Texture> texture;


////
// Streams

struct VSInput {
	float4 pos : SV_Position;
	float4 color;
	float2 texcoord;
}

struct VSOutput {
	float4 pos_clip : SV_Position;
	float4 color;
	float2 texcoord;
}


////
// Vertex shader

/// Vertex shader entry point
[shader("vertex")]
func vertexMain (vert: VSInput) -> VSOutput {
	var out: VSOutput;
	out.pos_clip = mul((viewing.projection * viewing.modelview), vert.pos);
	out.color = vert.color;
	out.texcoord = vert.texcoord;
	return out;
}


////
// Fragment shader

/// Fragment shader entry point
[shader("fragment")]
func fragmentMain (vsOut: VSOutput) -> float4 {
	var texColor: float4 = texture.color.Sample(texture.sampler, vsOut.texcoord);
	return float4(lerp(vsOut.color.rgb, texColor.rgb, texColor.a), 1);
}
