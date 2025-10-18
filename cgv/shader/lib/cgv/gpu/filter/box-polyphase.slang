
//////
//
// Module definition
//

// A module (re-)defining mathematical operators for linear algebra.
module boxpolyphase;

// Insert into cgv::gpu::filter namespace
namespace cgv::gpu::filter {



//////
//
// Structs
//

struct BoxPolyphase {
	func evaluate (srcTexture: DescriptorHandle<Texture2D>, src: float2) -> float4 {
		return float4(1,0,0,1);
	}
};



//////
//
// Module finalization
//

// Module namespace close
}
