
////
// Library

fn filterMipmap1D (
	targetTexture: texture_storage_1d<rgba8unorm,write>, targetTexel: u32, srcTexture: texture_1d<f32>,
	srcRes: u32
) -> vec4<f32> {
	return vec4(1, 0, 0, 1);
}

fn filterMipmap2D (
	targetTexture: texture_storage_2d<rgba8unorm,write>, targetTexel: vec2<u32>, srcTexture: texture_2d<f32>,
	srcRes: vec2<u32>
) -> vec4<f32>
{
	let offset = vec2<u32>(0, 1);
	let color = (
		textureLoad(srcTexture, 2*id.xy + offset.xx, 0) +
		textureLoad(srcTexture, 2*id.xy + offset.xy, 0) +
		textureLoad(srcTexture, 2*id.xy + offset.yx, 0) +
		textureLoad(srcTexture, 2*id.xy + offset.yy, 0)
	) * 0.25;
	textureStore(curMipmap, id.xy, color);
}

fn filterMipmap3D (
	targetTexture: texture_storage_3d<rgba8unorm,write>, targetVoxel: vec3<u32>, srcTexture: texture_3d<f32>,
	srcRes: vec3<u32>
) -> vec4<f32> {
	return vec4(1, 0, 0, 1);
}
