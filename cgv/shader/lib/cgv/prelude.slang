
//////
//
// Module setup
//

/// Commonly used assorted functionality that is relevant across problem domains.
module common;


/////
// Module namespace

// Insert into cgv::geom namespace
public namespace cgv {



//////
//
// Type aliases and enums
//

/// A convenience type alias for generic 2-dimensional vectors.
public typealias Vec2<Number: IArithmetic> = vector<Number, 2>;

/// A convenience type alias for generic 3-dimensional vectors.
public typealias Vec3<Number: IArithmetic> = vector<Number, 3>;

/// A convenience type alias for generic 3-dimensional vectors.
public typealias Vec4<Number: IArithmetic> = vector<Number, 4>;



//////
//
// Structs
//

/// A struct providing assorted mathematical and meta (compile-time) constants.
public struct Constants<Number: IFloat> {
	public static constexpr Number inf = Number(1)/Number(0);
	public static constexpr Number negInf = -inf;
}



//////
//
// Module finalization
//

// Module namespace close
}
