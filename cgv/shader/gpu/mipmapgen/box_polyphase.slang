
//////
//
// Imports
//

// CGV-rs core shader library
import "cgv/media/util.slang";        // for the texture utility functions
import "cgv/lin/common.slang";        // enables the cgv::lin::IElement interface for all float-based builtins
import "cgv/gpu/filter/kernel.slang"; // various filter kernels

// Use CGV namespace by default
using namespace cgv;



//////
//
// Uniforms
//

/// The input/output set of textures (2D variant).
struct TextureInOut2D
{
	/// The previous mip level.
	Texture2D prevMipmap;

	/// The current (to be calculated) mip level.
	[format("rgba8")]
	WTexture2D curMipmap;
};
[vk::binding(0,0)]
ParameterBlock<TextureInOut2D> tex2d;

/// The input/output set of textures (3D variant).
struct TextureInOut3D
{
	/// The previous mip level.
	Texture3D prevMipmap;

	/// The current (to be calculated) mip level.
	[format("rgba8")]
	WTexture3D curMipmap;
};
[vk::binding(0,0)]
ParameterBlock<TextureInOut3D> tex3d;



//////
//
// Compute shader
//

/// Entry point for the 2D generator.
[shader("compute")]
[numthreads(8, 8, 1)]
func boxPolyphase2D (int3 id : SV_DispatchThreadID)
{
	let tgtSize = media::getTexResolution(tex2d.curMipmap);
	let origSize = media::getTexResolution(tex2d.prevMipmap);
	let pow2 = media::areTexDimsPow2(tex2d.prevMipmap);
	let filtered = gpu::filter::kernel::polyphaseBox3(
		tgtSize, origSize, 2*id.xy, (int2 q) => tex2d.prevMipmap.Load(int3(q, 0))
	);
	tex2d.curMipmap.Store(id.xy, filtered);
}

/// Entry point for the 3D generator.
[shader("compute")]
[numthreads(8, 8, 1)]
func boxPolyphase3D (int3 id : SV_DispatchThreadID)
{
	tex3d.curMipmap.Store(
		id, gpu::filter::kernel::polyphaseBox3(
			media::getTexResolution(tex3d.curMipmap), media::getTexResolution(tex3d.prevMipmap), 2*id,
			(int3 q) => tex3d.prevMipmap.Load(int4(q, 0))
		)
	);
}
