
//////
//
// Imports
//

// Standard library
use std::collections::BTreeSet;

// Egui Code Editor widget
use egui_code_editor::Syntax;



//////
//
// Traits
//

/// An extension trait adding a constructor for the *Slang* shading language.
pub trait SlangSyntax {
	fn slang () -> Self;
}
impl SlangSyntax for Syntax
{
	fn slang () -> Self
	{
		Syntax {
			language: "Slang",
			case_sensitive: true,
			comment: "//",
			comment_multiline: ["/*", "*/"],
			hyperlinks: BTreeSet::from(["http"]),
			keywords: BTreeSet::from([
				"module", "func", "struct", "interface", "break", "continue", "if", "else", "const", "static", "let",
				"var", "for", "do", "while", "switch", "import", "mod", "type", "typealias", "return", "where",
				"override", "in", "out", "inout", "this", "public", "property", "get", "set"
			]),
			types: BTreeSet::from([
				// Primitives
				"bool", "int", "uint", "float", "half",

				// Complex primitive
				"float2",
				"float3",
				"float4",
				"float2x2",
				"float3x2",
				"float4x2",
				"float2x3",
				"float3x3",
				"float4x3",
				"float2x4",
				"float3x4",
				"float4x4",
				"half2",
				"half3",
				"half4",
				"half2x2",
				"half3x2",
				"half4x2",
				"half2x3",
				"half3x3",
				"half4x3",
				"half2x4",
				"half3x4",
				"half4x4",

				// Complex primitive generic types
				"vector",
				"matrix",

				// Primitive traits
				"__BuiltinType",
				"__BuiltinArithmeticType",
				"__BuiltinSignedArithmeticType",
				"__BuiltinIntegerType",
				"__BuiltinLogicalType",
				"__BuiltinInt32Type",
				"__BuiltinRealType",
				"__BuiltinFloatingPointType",
				"IInteger",
				"IFloat",
				"IArithmetic",
				"IArray",
				"IRWArray",
				"IFunc",

				// Builtin functions
				"abs",
				"floor",
				"ceil",
				"mul",
				"sin",
				"cos",
				"tan",
				"asin",
				"acos",
				"atan",
				"atan2",
				"sinh",
				"cosh",
				"tanh",
				"min",
				"max",
				"clamp",
				"saturate",
				"lerp",
				"length",
				"normalize",
				"step",
				"smoothstep",
				"ddx",
				"ddy",
				"fwidth"
			]),
			special: BTreeSet::from([
				"true", "false", "Unroll", "Inline", "ForceUnroll" ,"ForceInline"
			]),
		}
	}
}
