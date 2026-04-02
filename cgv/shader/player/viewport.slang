
//////
//
// Streams
//

/// Vertex data of the viewport quad.
struct ViewportGeometry {
	/// The clip-space position.
	float4 pos_clip : SV_Position;

	/// The texture coordinates.
	float2 uv;
}



//////
//
// Constants
//

/// Viewport-covering clip-space vertices.
constexpr static let vpVertices: ViewportGeometry[] = {
	{float4(-1,-1, 0, 1), float2(0, 1)}, {float4(1,-1, 0, 1), float2(1, 1)},
	{float4(-1, 1, 0, 1), float2(0, 0)}, {float4(1, 1, 0, 1), float2(1, 0)}
};




//////
//
// Vertex shader
//

////
// Entry points

/// Our sole vertex shader entry point.
[shader("vertex")]
func vertexMain (uint vertexID: SV_VertexID) -> ViewportGeometry {
	return vpVertices[vertexID];
}



//////
//
// Fragment shader
//

////
// Uniforms

/// Data layout of the uniform block providing the texture with the viewport contents.
struct ViewportTexture {
	/// The color buffer.
	Texture2D color;

	/// The sampler for reading the color buffer.
	SamplerState sampler;
};
/// The texture with the viewport contents.
[vk::binding(0, 0)] // <- start at binding(0) of group(0)
ParameterBlock<ViewportTexture> vpTex;

/// The global gamma values.
[vk::binding(0, 1)] // <- start at binding(0) of group(1)
ParameterBlock<float3> gamma;


////
// Entry points

/// Our sole fragment shader entry point.
[shader("fragment")]
func fragmentMain (vpIn: ViewportGeometry) -> float4 {
	let unpremultiplied: float4 = vpTex.color.Sample(vpTex.sampler, vpIn.uv);
	return float4(
		pow(unpremultiplied.rgb * unpremultiplied.a, gamma), // pre-multiply and transform to sRGB
		unpremultiplied.a
	);
}
