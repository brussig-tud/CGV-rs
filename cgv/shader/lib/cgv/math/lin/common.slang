
//////
//
// Module setup
//

/// Commonly used assorted functionality for handling geometry.
module common;


////
// Imports

// Local imports
/* nothing here yet */


////
// Module namespace

// Insert into cgv::math::lin namespace
public namespace cgv::math::lin {



//////
//
// Interfaces
//

/// Interface of generic elements of algebraic structures, e.g. numbers, vectors, matrices, etc.
public interface IElement: IFloat
{
	/// The type of the internal number representation.
	associatedtype Repr: __BuiltinFloatingPointType;

	/// The dimensionality of the element..
	public constexpr static const uint dims;

	/// Create an instance of the element with all components set to the given value.
	public static func splat (in const value: Repr) -> This;

	/// Perform linear interpolation of this element and another one.
	public func lerp (in const other: This, in const t: Repr) -> This;

	/// Perform fused-multiply-add operation on the element with the given other elements.
	public func fma (in const mult: This, in const add: This) -> This;

	/// Perform fused-multiply-add operation on the element by scaling it with the given scalar and adding the other element.
	public func fma (in const scalar: Repr, in const add: This) -> This;
}
public extension<T: __BuiltinFloatingPointType> T: IElement
{
	public typealias Repr = This;
	public constexpr static const uint dims = 1;

	[ForceInline]
	public static func splat (in const value: Repr) -> This {
		return This(value);
	}

	[ForceInline]
	public func lerp (in const other: This, in const t: Repr) -> This {
		return lerp(this, other, t);
	}

	[ForceInline]
	public func fma (in const mult: This, in const add: This) -> This {
		return fma(this, mult, add);
	}
}
public struct Foo {
	public int f;
	public double o;
}
public extension<T: __BuiltinFloatingPointType, int N> vector<T, N>: IElement
{
	public typealias Repr = T;
	public constexpr static const uint dims = N;

	[ForceInline]
	public static func splat (in const value: Repr) -> This {
		return This(value);
	}

	[ForceInline]
	public func lerp (in const other: This, in const t: Repr) -> This {
		return lerp(this, other, t);
	}

	[ForceInline]
	public func fma (in const mult: This, in const add: This) -> This {
		return fma(this, mult, add);
	}
}
public extension<T: __BuiltinFloatingPointType, int C, int R> matrix<T, C, R>: IElement
// Note switched usage of C and R wrt. Slang `matrix` generic definition since we treat Slang matrices as column-major
// even through they're technically row-major.
{
	public typealias Repr = T;
	public constexpr static const uint dims = C*R;

	[ForceInline]
	public static func splat (in const value: Repr) -> This {
		return This(value);
	}

	[ForceInline]
	public func lerp (in const other: This, in const t: Repr) -> This {
		return lerp(this, other, t);
	}

	[ForceInline]
	public func fma (in const mult: This, in const add: This) -> This {
		return fma(this, mult, add);
	}
}

/// Interface of series of generic elements of algebraic structures, e.g. lists or vectors of numbers, lists or matrices of column vectors, etc.
public interface IElementSeq<Elem: IElement, int N>
{
	/// Access an element in the sequence.
	__subscript (uint i) -> Elem;

	/// Overwrite an element in the sequence.
	[mutating]
	public func set (int i, in const newValue: Elem);
}
public extension<T: __BuiltinFloatingPointType, int Dplus1> vector<T, Dplus1>: IElementSeq<T, Dplus1> {
	[ForceInline][mutating]
	public func set (int i, in const newValue: T) {
		this[i] = newValue;
	}
}
public extension<T: __BuiltinFloatingPointType, int N, int Num> matrix<T, Num, N>: IElementSeq<vector<T, N>, Num>
{
	__subscript (int i) -> vector<T, N> {
		[ForceInline]           get { return this[i]; }
		[ForceInline][mutating] set { this[i] = newValue; }
	}

	[ForceInline][mutating]
	public func set (int i, in const newValue: vector<T, N>) {
		this[i] = newValue;
	}
}



//////
//
// Module finalization
//

// Module namespace close
}
