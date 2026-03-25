
//////
//
// Module setup
//

/// Meta-programming utilites useful for statically more complicated expressions at compile time.
module meta;


/////
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



//////
//
// Module finalization
//

// Module namespace close
}
