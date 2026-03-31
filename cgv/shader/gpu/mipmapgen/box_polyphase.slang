
//////
//
// Imports
//

// CGV-rs core shader library
import "cgv/api/uniforms.slang";      // for the standardized texture in-/ouput uniform block
import "cgv/media/texture.slang";     // for the texture utility functions
import "cgv/lin/common.slang";        // enables the cgv::lin::IElement interface for all float-based builtins
import "cgv/gpu/filter/kernel.slang"; // various filter kernels

// Use CGV namespace by default
using namespace cgv;



//////
//
// Uniforms
//

/// Common input output type, for now only parametrized over texture shape.
typealias TextureInOut<Shape: __ITextureShape> = api::TextureInOut<Shape, media::tex::Format::RGBA8>;

/// The input/output set of textures (2D variant).
[vk::binding(0, 0)] // <- start at binding(0) of group(0)
ParameterBlock<TextureInOut<__Shape2D>> mip2d;

/// The input/output set of textures (3D variant).
[vk::binding(0, 0)] // <- start at binding(0) of group(0)
ParameterBlock<TextureInOut<__Shape3D>> mip3d;



//////
//
// Compute shader
//

/// Entry point for the 2D generator.
[shader("compute")]
[numthreads(8, 8, 1)]
func boxPolyphase2D (int3 id: SV_DispatchThreadID)
{
	mip2d.output.Store(
		id.xy, gpu::filter::kernel::polyphaseBox3(
			/* orig: */ media::tex::getResolution(mip2d.input),  /* sampleLocation: */ 2*id.xy,
			/* dataAccessor: */ (int2 q) => mip2d.input.Load(int3(q, 0))
		)
	);
}

/// Entry point for the 3D generator.
[shader("compute")]
[numthreads(8, 8, 1)]
func boxPolyphase3D (int3 id: SV_DispatchThreadID)
{
	mip3d.output.Store(
		id, gpu::filter::kernel::polyphaseBox3(
			/* orig: */ media::tex::getResolution(mip3d.input),  /* sampleLocation: */ 2*id,
			/* dataAccessor: */ (int3 q) => mip3d.input.Load(int4(q, 0))
		)
	);
}
