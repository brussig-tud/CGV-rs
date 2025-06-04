
//////
//
// Module definition
//

// A module providing the common uniforms defined by the CGV-rs rendering API.
module cgv_api_uniforms;

// Insert into cgv::lin namespace
namespace cgv::api {



//////
//
// Structs
//

// The uniform group containing viewing information. Corresponds to cgv::RenderState::viewingUniforms.
public struct Viewing
{
	// The modelview matrix.
	public float4x4 modelview;

	// The projection matrix.
	public float4x4 projection;
}



//////
//
// Module finalization
//

// Module namespace close
}
