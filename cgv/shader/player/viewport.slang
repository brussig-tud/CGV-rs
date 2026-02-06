
////
// Uniforms

struct ViewportSource {
	Texture2D color;
	SamplerState sampler;
};
ParameterBlock<ViewportSource> viewportSource;


////
// Streams

struct ViewportGeometry {
	float4 pos_clip : SV_Position;
	float2 uv;
}



////
// Vertex shader

/// Viewport-covering clip-space vertices
static constexpr let vpVertices: ViewportGeometry[] = {
	{float4(-1,-1, 0, 1), float2(0, 1)}, {float4(1,-1, 0, 1), float2(1, 1)},
	{float4(-1, 1, 0, 1), float2(0, 0)}, {float4(1, 1, 0, 1), float2(1, 0)}
};

/// Vertex shader entry point
[shader("vertex")]
func vertexMain (uint vertexID: SV_VertexID) -> ViewportGeometry {
	return vpVertices[vertexID];
}


////
// Fragment shader

/// Fragment shader entry point
[shader("fragment")]
func fragmentMain (vpIn: ViewportGeometry) -> float4 {
	var srcUnpremultiplied: float4 = viewportSource.color.Sample(viewportSource.sampler, vpIn.uv);
	return pow(float4(srcUnpremultiplied.rgb * srcUnpremultiplied.a, srcUnpremultiplied.a), 1/2.2); // transform to sRGB
}
