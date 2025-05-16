
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

// Shader entry point: premultiplied
@fragment
fn fs_premultiplied (in: VertexOutput) -> @location(0) vec4f {
	return vec4f(textureSample(source, smpler, in.uv));
}

// Shader entry point: non-premultiplied
@fragment
fn fs_non_premultiplied (in: VertexOutput) -> @location(0) vec4f {
	var srcUnpremultiplied = textureSample(source, smpler, in.uv);
	return vec4f(srcUnpremultiplied.rgb * srcUnpremultiplied.a, srcUnpremultiplied.a);
}

// Our fragment shader is almost trivial, with the most interesting
// thing being how it uses the `TMaterial` type parameter (through the
// value stored in the `gMaterial` parameter block) to dispatch to
// the correct implementation of the `getDiffuseColor()` method
// in the `IMaterial` interface.
//
// The `gMaterial` parameter block declaration thus serves not only
// to group certain shader parameters for efficient CPU-to-GPU
// communication, but also to select the code that will execute
// in specialized versions of the `fragmentMain` entry point.
//
[shader("fragment")]
float4 fs_nonPremultiplied(in: VertexOutput) : SV_Target
{
    // We start by using our interpolated vertex attributes
    // to construct the local surface geometry that we will
    // use for material evaluation.
    //
    SurfaceGeometry g;
    g.position = coarseVertex.worldPosition;
    g.normal = normalize(coarseVertex.worldNormal);
    g.uv = coarseVertex.uv;

    float3 V = normalize(gViewParams.eyePosition - g.position);

    // Next we prepare the material, which involves running
    // any "pattern generation" logic of the material (e.g.,
    // sampling and blending texture layers), to produce
    // a BRDF suitable for evaluating under illumination
    // from different light sources.
    //
    // Note that the return type here is `gMaterial.BRDF`,
    // which is the `BRDF` type *associated* with the (unknown)
    // `TMaterial` type. When `TMaterial` gets substituted for
    // a concrete type later (e.g., `SimpleMaterial`) this
    // will resolve to a concrete type too (e.g., `SimpleMaterial.BRDF`
    // which is an alias for `BlinnPhong`).
    //
    let brdf = gMaterial.prepare(g);

    // Now that we've done the first step of material evaluation
    // and sampled texture maps, etc., it is time to start
    // integrating incident light at our surface point.
    //
    // Because we've wrapped up the lighting environment as
    // a single (composite) object, this is as simple as calling
    // its `illuminate()` method. Our particular fragment shader
    // is thus abstracted from how the renderer chooses to structure
    // this integration step, somewhat similar to how an
    // `illuminance` loop in RenderMan Shading Language works.
    //

    float3 color = saturate(gLightEnv.illuminate(g, brdf, V) + float3(0.3));
    return float4(color, 1);
}
