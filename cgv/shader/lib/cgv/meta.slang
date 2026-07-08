
//////
//
// Module setup
//

/// Meta-programming utilites useful for statically more complicated expressions at compile time.
module meta;


////
// Module namespace

// Insert into cgv::meta namespace
public namespace cgv::meta {



//////
//
// Functions
//

/// Evaluate the power of a base to an exponent as a compile-time constant.
[ForceInline]
constexpr public func pow<int Base, uint Exp> () -> int {
	return (Exp > 2 ? pow<Base, Exp-1>() : Base) * Base;
}

/// Evaluate the given power of 2 as a compile-time constant.
[ForceInline]
constexpr public func pow2<uint Exp> () -> int {
	return pow<2, Exp>();
}

/// Evaluate the binomial coefficient *N*-choose-*K* as a compile-time constant.
[ForceInline]
constexpr public func binomial<int N, int K>() -> int {
	return K==0 || K==N ? 1 : binomial<N-1, K-1>() + binomial<N-1, K>();
}



//////
//
// Module finalization
//

// Module namespace close
}
