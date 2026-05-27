
//////
//
// Module definition
//

/// A module providing the common uniforms defined by the CGV-rs rendering API.
module uniforms;


////
// Imports

// Local imports
import "cgv/media/texture.slang";


////
// Module namespace

// Insert into cgv::api namespace
namespace cgv::api {



//////
//
// Structs
//

/// The uniform group containing viewing information. Corresponds to `cgv::RenderState::viewingUniforms`.
public struct Viewing
{
	/// The modelview matrix.
	public float4x4 modelview;

	/// The projection matrix.
	public float4x4 projection;

	/// The combined `projection`×`modelview` matrix
	public float4x4 projModelview;

	/// The inverse modelview transformation matrix.
	public float4x4 modelview_inv;

	/// The inverse projection matrix.
	public float4x4 projection_inv;

	/// The inverse combined `projection`×`modelview` matrix (i.e. `modelview_inv`×`projection_inv`).
	public float4x4 projModelview_inv;

	/// The normal matrix.
	public float4x4 normal;

	/// The inverse normal matrix.
	public float4x4 normal_inv;
}

/// The uniform group containing input/output texture for mipmap generation. Used by all `cgv::gpu::mipmap::ShaderFilter`s.
public struct TextureInOut<
	Shape: __ITextureShape, media::tex::Format format, TexelContent: ITexelElement=float4, int sampleCount=0
>{
	/// The input texture.
	public _Texture<TexelContent, Shape, 0, 0, sampleCount, /* access: */0, 0, 0, format> input;

	/// The output texture.
	public _Texture<TexelContent, Shape, 0, 0, sampleCount, /* access: */2, 0, 0, format> output;
}



//////
//
// Module finalization
//

// Module namespace close
}
