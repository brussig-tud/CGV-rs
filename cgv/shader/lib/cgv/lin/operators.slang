
//////
//
// Module definition
//

// A module (re-)defining mathematical operators for linear algebra.
module operators;



//////
//
// Functions
//

// Overload multiplication for matrix-matrix operands.
public float4x4 operator* (float4x4 m0, float4x4 m1) {
	return mul(m1, m0);
}

// Overload multiplication for matrix-vector operands.
public float4 operator* (float4x4 m, float4 v) {
	return mul(v, m);
}
