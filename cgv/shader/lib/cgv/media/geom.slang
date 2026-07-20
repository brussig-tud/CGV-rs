
//////
//
// Module setup
//

/// Datastructures and interfaces for representing geometry.
module geom;


////
// Imports

// Shaderlib prelude
import "cgv/prelude.slang";

// Linear algebra module
import "cgv/math/lin/common.slang";


////
// Module namespace

// Insert into cgv::media namespace
public namespace cgv::media {

// Import root namespace
using cgv;



//////
//
// Interfaces
//

/// An interface for vertices that can be consumed by the GPU rasterization pipeline.
public interface IVertex {
	/// The vertex type with at least one field with `SV_Position` semantics.
	associatedtype VertexType: IPosition<float>;

	/// Emit a GPU-consumable vertex.
	[ForceInline]
	public func emit () -> VertexType;
}

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
	property uv: vector<Scalar, N>;
}
public extension<T: __BuiltinFloatingPointType, int N> vector<T, N>: ITexCoords<T, N> {
	property uv: vector<T, N> {
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
public struct VertexN<PosScalar: __BuiltinFloatingPointType>: IPosition<PosScalar>, INormal<float>, IVertex
{
	/// Our GPU-consumable vertex type.
	public struct GpuVertex: IPosition<float>, INormal<float>
	{
		public float4 pos : SV_Position;
		public var normal: float4;

		[ForceInline]
		public __init (Vec4<PosScalar> pos, float4 normal) {
			this.pos = float4(pos); this.normal = normal;
		}
	}
	public typealias VertexType = GpuVertex;

	/// The vertex position.
	public Vec4<PosScalar> pos;

	/// The vertex normal.
	public float4 normal;

	///	Emit a GPU-consumable version.
	[ForceInline]
	public func emit () -> VertexType {
		return VertexType(pos, normal);
	}

	/// Construct with the given position and normal.
	[ForceInline]
	public __init (Vec4<PosScalar> pos, float4 normal) {
		this.pos = pos; this.normal = normal;
	}
}

/// A simple vertex with 2D texture coordinates (the *U* refers to the coordinate tuple *uv*).
public struct VertexU<PosScalar: __BuiltinFloatingPointType>: IPosition<PosScalar>, ITexCoords<float, 2>, IVertex
{
	/// Our GPU-consumable vertex type.
	public struct GpuVertex: IPosition<float>, ITexCoords<float, 2>
	{
		public float4 pos : SV_Position;
		public var uv: float2;

		[ForceInline]
		public __init (Vec4<PosScalar> pos, float2 uv) {
			this.pos = float4(pos); this.uv = uv;
		}
	}
	public typealias VertexType = GpuVertex;

	/// The vertex position.
	public Vec4<PosScalar> pos;

	/// The vertex texture coordinates.
	public float2 uv;

	///	Emit a GPU-consumable version.
	[ForceInline]
	public func emit () -> VertexType {
		return VertexType(pos, uv);
	}

	/// Construct with the given position and texture coordinates.
	[ForceInline]
	public __init (Vec4<PosScalar> pos, float2 uv) {
		this.pos = pos; this.uv = uv;
	}
}

/// A simple vertex with color.
public struct VertexC<PosScalar: __BuiltinFloatingPointType>: IPosition<PosScalar>, IColor, IVertex
{
	/// Our GPU-consumable vertex type.
	public struct GpuVertex: IPosition<float>, IColor
	{
		public float4 pos : SV_Position;
		public var color: float4;

		[ForceInline]
		public __init (Vec4<PosScalar> pos, float4 color) {
			this.pos = float4(pos); this.color = color;
		}
	}
	public typealias VertexType = GpuVertex;

	/// The vertex position.
	public Vec4<PosScalar> pos;

	/// The vertex color.
	public float4 color;

	///	Emit a GPU-consumable version.
	[ForceInline]
	public func emit () -> VertexType {
		return VertexType(pos, color);
	}

	/// Construct with the given position and normal.
	[ForceInline]
	public __init (Vec4<PosScalar> pos, float4 color) {
		this.pos = pos; this.color = color;
	}
}

/// A simple vertex with normal and color.
public struct VertexNC<PosScalar: __BuiltinFloatingPointType>: IPosition<PosScalar>, INormal<float>, IColor, IVertex
{
	/// Our GPU-consumable vertex type.
	public struct GpuVertex: IPosition<float>, INormal<float>, IColor
	{
		public float4 pos : SV_Position;
		public var normal: float4;
		public var color: float4;

		[ForceInline]
		public __init (Vec4<PosScalar> pos, float4 normal, float4 color) {
			this.pos = float4(pos); this.normal = normal; this.color = color;
		}
	}
	public typealias VertexType = GpuVertex;

	/// The vertex position.
	public Vec4<PosScalar> pos;

	/// The vertex normal.
	public float4 normal;

	/// The vertex color.
	public float4 color;

	///	Emit a GPU-consumable version.
	[ForceInline]
	public func emit () -> VertexType {
		return VertexType(pos, normal, color);
	}

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
