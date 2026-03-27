
//////
//
// Imports
//

// CGV-rs core shader library
import "cgv/lin/common.slang"; // enables the cgv::lin::IElement interface for all float-based builtins
import "cgv/gpu/filter/kernel.slang"; // various filter kernels

// Use CGV namespace by default
using namespace cgv;



//////
//
// Uniforms
//

/// Input/output set of textures (2D variant)
struct TextureInOut2D
{
	/// Previous mip level
	Texture2D prevMipmap;

	/// Current mip level
	[format("rgba8")]
	WTexture2D curMipmap;
};
[vk::binding(0,0)]
ParameterBlock<TextureInOut2D> tex2d;

/// Input/output set of textures (3D variant)
struct TextureInOut3D
{
	/// Previous mip level
	Texture3D prevMipmap;

	/// Current mip level
	[format("rgba8")]
	WTexture3D curMipmap;
};
[vk::binding(0,0)]
ParameterBlock<TextureInOut3D> tex3d;



//////
//
// Compute shader
//

[shader("compute")]
[numthreads(8, 8, 1)]
func boxPolyphase2D (int3 id : SV_DispatchThreadID) {
	tex2d.curMipmap.Store(
		id.xy, gpu::filter::kernel::box2(2*id.xy, (int2 q) => tex2d.prevMipmap.Load(int3(q, 0)))
	);
}

[shader("compute")]
[numthreads(8, 8, 1)]
func boxPolyphase3D (int3 id : SV_DispatchThreadID) {
	tex3d.curMipmap.Store(
		id, gpu::filter::kernel::box2(2*id, (int3 q) => tex3d.prevMipmap.Load(int4(q, 0)))
	);
}
