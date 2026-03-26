
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

/// Input/output set of textures
struct TextureInOut
{
	/// Previous mip level
	Texture2D prevMipmap;

	/// Current mip level
	[format("rgba8")]
	WTexture2D curMipmap;
};
ParameterBlock<TextureInOut> tex;



//////
//
// Compute shader
//

// Shader entry point
[numthreads(8, 8, 1)]
func main (int3 id : SV_DispatchThreadID) {
	tex.curMipmap.Store(
		id.xy, gpu::filter::kernel::box2(2*id.xy, (int2 q) => tex.prevMipmap.Load(int3(q, 0)))
	);
}
