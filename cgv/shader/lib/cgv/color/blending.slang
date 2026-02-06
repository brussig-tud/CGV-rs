
//////
//
// Module setup
//

/// Functionality that facilitates alpha blending.
module blending;


/////
// Module namespace

// Insert into cgv::color namespace
public namespace cgv::color {



//////
//
// Constants
//

////
// Sub-namespaces

/// Namespace providing some pre-defined SDFs.
public namespace blend {

/// The *Over* blending operator.
public static constexpr Over over;


// Sub-namespace close
}



//////
//
// Interfaces
//

///
public interface IBlendOperator: IFunc<float4, float4, float4>
{
	///
	public func blend (in back: float4, in fore: float4) -> float4;

	///
	public float4 operator() (float4 back, float4 fore) {
		return blend(back, fore);
	}
}



//////
//
// Structs
//

////
// Sub-namespaces

/// Namespace providing some pre-defined SDFs.
namespace blend {

///
public struct Over: IFunc<float4, float4, float4>, IBlendOperator {
	override func blend (in back: float4, in fore: float4) -> float4 {
		return float4(/* color: */back.rgb*(1-fore.a) + fore.rgb, /* alpha: */back.a *(1-fore.a) + fore.a);
	}

	override float4 operator() (float4 back, float4 fore) {
		return blend(back, fore);
	}
}

///
public struct Over_noPreMult: IFunc<float4, float4, float4>, IBlendOperator {
	override func blend (in back: float4, in fore: float4) -> float4 {
		let a_out = back.a*(1-fore.a) + fore.a;
		return float4(/* color: */fore.rgb*fore.a + back.rgb*back.a*(1-fore.a) / a_out, /* alpha: */a_out);
	}

	override float4 operator() (float4 back, float4 fore) {
		return blend(back, fore);
	}
}

///
public struct Over_foreNotPreMult: IFunc<float4, float4, float4>, IBlendOperator {
	override func blend (in back: float4, in fore: float4) -> float4 {
		return over(back, float4(fore.rgb*fore.a, fore.a));
	}

	override float4 operator() (float4 back, float4 fore) {
		return blend(back, fore);
	}
}


// Sub-namespace close
}



//////
//
// Module finalization
//

// Module namespace close
}
