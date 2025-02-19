
////
// Compute shader

// Uniforms
// - textures
@group(0) @binding(0)
var prevMipmap: texture_2d<f32>;
@group(0) @binding(1)
var curMipmap: texture_storage_2d<rgba8unorm,write>;

// Shader entry point
@compute @workgroup_size(8, 8)
fn kernel (@builtin(global_invocation_id) id: vec3<u32>)
{
	let offset = vec2<u32>(0, 1);
	let color = (
		textureLoad(prevMipmap, 2*id.xy + offset.xx, 0) +
		textureLoad(prevMipmap, 2*id.xy + offset.xy, 0) +
		textureLoad(prevMipmap, 2*id.xy + offset.yx, 0) +
		textureLoad(prevMipmap, 2*id.xy + offset.yy, 0)
	) * 0.25;
	textureStore(curMipmap, id.xy, color);
}
