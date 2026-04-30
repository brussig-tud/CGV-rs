
//////
//
// Module setup
//

/// Datastructures and interfaces for representing geometry.
module geom;


//////
// Imports

// Shaderlib prelude
import "cgv/prelude.slang";

// Linear algebra module
import "cgv/math/lin/common.slang";


/////
// Module namespace

// Insert into cgv::media namespace
public namespace cgv::media {

// Import root namespace
using cgv;



//////
//
// Interfaces
//

/// An interface for data records containing a position (e.g. in a vertex).
public interface IPosition<Scalar: __BuiltinFloatingPointType> {
	/// The position.
	property pos: Vec4<Scalar>;
}
public extension<T: __BuiltinFloatingPointType> vector<T, 4>: IPosition<T> {
	property pos: Vec4<T> {
		[ForceInline] get { return this; }
		[ForceInline] set { this = newValue; }
	}
}

/// An interface for data records containing a normal (e.g. in a vertex).
public interface INormal<Scalar: __BuiltinFloatingPointType> {
	/// The normal.
	property normal: Vec4<Scalar>;
}
public extension<T: __BuiltinFloatingPointType> vector<T, 4>: INormal<T> {
	property normal: Vec4<T> {
		[ForceInline] get { return this; }
		[ForceInline] set { this = newValue; }
	}
}

/// An interface for data records containing a tangent (e.g. in a vertex).
public interface ITangent<Scalar: __BuiltinFloatingPointType> {
	/// The tangent.
	property tangent: Vec4<Scalar>;
}
public extension<T: __BuiltinFloatingPointType> vector<T, 4>: ITangent<T> {
	property tangent: Vec4<T> {
		[ForceInline] get { return this; }
		[ForceInline] set { this = newValue; }
	}
}

/// An interface for data records containing a single set of texture coordinates (e.g. in a vertex).
///
/// **NOTE**: the *CGV-rs* core shader library intentionally does not provide standardized interfaces for sets of multiple texture coordinates. Rather, algorithms/datastructures or other facilities that need them should define their own interfaces for this, with clearly defined semantics for each set.
public interface ITexCoords<Scalar: __BuiltinFloatingPointType, int N> {
	/// The set of texture coordinates.
	property texcoord: vector<Scalar, N>;
}
public extension<T: __BuiltinFloatingPointType, int N> vector<T, N>: ITexCoords<T, N> {
	property texcoord: vector<T, N> {
		[ForceInline] get { return this; }
		[ForceInline] set { this = newValue; }
	}
}

/// An interface for data records containing a color (e.g. in a vertex).
public interface IColor {
	/// The color.
	property color: float4;
}
public extension float4: IColor {
	property color: float4 {
		[ForceInline] get { return this; }
		[ForceInline] set { this = newValue; }
	}
}



//////
//
// Structs
//

/// A simple vertex with normal.
public struct VertexN<PosScalar: __BuiltinFloatingPointType>: IPosition<PosScalar>, INormal<float>
{
	/// The vertex position.
	public Vec4<PosScalar> pos;

	/// The vertex normal.
	public float4 normal;

	/// Construct with the given position and normal.
	[ForceInline]
	public __init (Vec4<PosScalar> pos, float4 normal) {
		this.pos = pos; this.normal = normal;
	}
}

/// A simple vertex with 2D texture coordinates.
public struct VertexTC<PosScalar: __BuiltinFloatingPointType>: IPosition<PosScalar>, ITexCoords<float, 2>
{
	/// The vertex position.
	public Vec4<PosScalar> pos;

	/// The vertex color.
	public float2 texcoord;

	/// Construct with the given position and normal.
	[ForceInline]
	public __init (Vec4<PosScalar> pos, float2 texcoord) {
		this.pos = pos; this.texcoord = texcoord;
	}
}

/// A simple vertex with color.
public struct VertexC<PosScalar: __BuiltinFloatingPointType>: IPosition<PosScalar>, IColor
{
	/// The vertex position.
	public Vec4<PosScalar> pos;

	/// The vertex color.
	public float4 color;

	/// Construct with the given position and normal.
	[ForceInline]
	public __init (Vec4<PosScalar> pos, float4 color) {
		this.pos = pos; this.color = color;
	}
}

/// A simple vertex with normal and color.
public struct VertexNC<PosScalar: __BuiltinFloatingPointType>: IPosition<PosScalar>, INormal<float>, IColor
{
	/// The vertex position.
	public Vec4<PosScalar> pos;

	/// The vertex normal.
	public float4 normal;

	/// The vertex color.
	public float4 color;

	/// Construct with the given position and normal.
	[ForceInline]
	public __init (Vec4<PosScalar> pos, float4 normal, float4 color) {
		this.pos = pos; this.normal = normal; this.color = color;
	}
}



//////
//
// Module finalization
//

// Module namespace close
}
