
//////
//
// Module setup
//

/// Commonly used assorted functionality for handling geometry.
module common;


//////
// Imports

// Local imports
/* nothing here yet */


/////
// Module namespace

// Insert into cgv::lin namespace
public namespace cgv::lin {



//////
//
// Interfaces
//

/// Interface of generic elements of algebraic structures, e.g. numbers, vectors, matrices, etc.
public interface IElement {
	/// The type of the internal number representation.
	associatedtype Repr: __BuiltinFloatingPointType;
}
public extension<T: __BuiltinFloatingPointType> T: IElement {
	public typealias Repr = This;
}
public extension<T: __BuiltinFloatingPointType, int N> vector<T, N>: IElement {
	public typealias Repr = T;
}
public extension<T: __BuiltinFloatingPointType, int R, int C> matrix<T, R, C>: IElement {
	public typealias Repr = T;
}



//////
//
// Module finalization
//

// Module namespace close
}
