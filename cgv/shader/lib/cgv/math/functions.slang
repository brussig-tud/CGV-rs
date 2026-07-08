
//////
//
// Module setup
//

/// A module providing mathematical functions that are not provided by the Slang/HLSL standard library.
module functions;


////
// Imports

// Local imports
import "cgv/prelude.slang";


////
// Module namespace

// Insert into cgv::math namespace
public namespace cgv::math {



//////
//
// Functions
//

/// Evaluate the binomial coefficient *n*-choose-*k*.
[ForceInline]
constexpr public func binomial<Int: __BuiltinIntegerType>(in const n: Int, in const k: Int) -> Int
{
	if (k > n)
		return Int(0);
	let k_new = k > n-k ? n-k : k;

	var result = Int(1);
	for (Int i=Int(1); i<=k_new; ++i)
		result = result * (n-k_new + i)/i;
	return result;
}



//////
//
// Module finalization
//

// Module namespace close
}
