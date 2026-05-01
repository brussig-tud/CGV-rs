
//////
//
// Imports
//

// CGV-rs core shader library
import "cgv/geom/billboard.slang";        // access the pre-defined billboards

// Use CGV namespace by default
using namespace cgv;



//////
//
// Vertex shader
//

////
// Entry points

/// Our sole vertex shader entry point.
[shader("vertex")]
func vertexMain (uint vertexID: SV_VertexID) -> media::VertexU<float> {
	return geom::Billboards<float>::quadU[vertexID];
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
func fragmentMain (vpIn: media::VertexU<float>) -> float4 {
	let unpremultiplied: float4 = vpTex.color.Sample(vpTex.sampler, vpIn.uv);
	return float4(pow(unpremultiplied.rgb * unpremultiplied.a, gamma), unpremultiplied.a);
}
