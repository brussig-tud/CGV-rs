
//////
//
// Module setup
//

/// Commonly used assorted functionality that is relevant across problem domains.
module prelude;


////
// Module namespace

// Insert into cgv namespace
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

/// A convenience type alias for generic 2x2 matrices.
public typealias Mat2<Number: IArithmetic> = matrix<Number, 2, 2>;

/// A convenience type alias for generic 3x3 matrices.
public typealias Mat3<Number: IArithmetic> = matrix<Number, 3, 3>;

/// A convenience type alias for generic 3x3 matrices.
public typealias Mat4<Number: IArithmetic> = matrix<Number, 4, 4>;



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



//////
//
// Operator overloads
//

/// Enable GLSL-style multiplication for floating-point matrix-matrix operands.
/*[ForceInline]
public matrix<T, M, M> operator*<T: __BuiltinFloatingPointType, int N, int M> (matrix<T, N, M> m0, matrix<T, M, N> m1) {
	let _N = N;
	let _M = M;
	return mul(m1, m0);
}

/// Enable GLSL-style multiplication for integer matrix-matrix operands.
[ForceInline]
public matrix<T, M, M> operator*<T: __BuiltinIntegerType, int N, int M> (matrix<T, N, M> m0, matrix<T, M, N> m1) {
	let _N = N;
	let _M = M;
	return mul(m1, m0);
}*/
[ForceInline]
public float2x2 operator* (float2x2 m0, float2x2 m1) {
	return mul(m1, m0);
}
[ForceInline]
public float3x3 operator* (float3x3 m0, float3x3 m1) {
	return mul(m1, m0);
}
[ForceInline]
public float4x4 operator* (float4x4 m0, float4x4 m1) {
	return mul(m1, m0);
}

/// Enable GLSL-style multiplication for floating-point matrix-vector operands.
[ForceInline]
public vector<T, M> operator*<T: __BuiltinFloatingPointType, int N, int M> (matrix<T, N, M> m, vector<T, N> v) {
	let _N = N;
	let _M = M;
	return mul(v, m);
}

/// Enable GLSL-style multiplication for integer matrix-vector operands.
[ForceInline]
public vector<T, M> operator*<T: __BuiltinIntegerType, int N, int M> (matrix<T, N, M> m, vector<T, N> v) {
	let _N = N;
	let _M = M;
	return mul(v, m);
}

/// Enable GLSL-style component-wise addition for floating-point vector types.
[ForceInline]
public vector<T, N> operator+<T: __BuiltinFloatingPointType, int N> (vector<T, N> v0, vector<T, N> v1) {
	let _N = N;
	return v0.add(v1);
}

/// Enable GLSL-style component-wise addition for integer vector types.
[ForceInline]
public vector<T, N> operator+<T: __BuiltinIntegerType, int N> (vector<T, N> v0, vector<T, N> v1) {
	let _N = N;
	return v0.add(v1);
}

/// Enable GLSL-style component-wise subtraction for floating-point vector types.
[ForceInline]
public vector<T, N> operator-<T: __BuiltinFloatingPointType, int N> (vector<T, N> v0, vector<T, N> v1) {
	let _N = N;
	return v0.sub(v1);
}

/// Enable GLSL-style component-wise subtraction for integer vector types.
[ForceInline]
public vector<T, N> operator-<T: __BuiltinIntegerType, int N> (vector<T, N> v0, vector<T, N> v1) {
	let _N = N;
	return v0.sub(v1);
}

/// Enable GLSL-style component-wise addition for floating-point matrix types.
[ForceInline]
public matrix<T, R, C> operator+<T: __BuiltinFloatingPointType, int R, int C> (matrix<T, R, C> m0, matrix<T, R, C> m1) {
	let _R = R;
	let _C = C;
	return m0.add(m1);
}

/// Enable GLSL-style component-wise subtraction for floating-point matrix types.
[ForceInline]
public matrix<T, R, C> operator-<T: __BuiltinFloatingPointType, int R, int C> (matrix<T, R, C> m0, matrix<T, R, C> m1) {
	let _R = R;
	let _C = C;
	return m0.sub(m1);
}
