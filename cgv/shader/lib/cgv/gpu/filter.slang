
//////
//
// Module definition
//

// A module (re-)defining mathematical operators for linear algebra.
module filter;

// Insert into cgv::gpu namespace
namespace cgv::gpu {



//////
//
// Interfaces
//

interface IFilter {
	func evaluate (srcTexture: DescriptorHandle<Texture2D>, src: float2) -> float3;
};



//////
//
// Module finalization
//

// Module namespace close
}
