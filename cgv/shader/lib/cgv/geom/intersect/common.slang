
//////
//
// Module setup
//

/// Commonly used assorted functionality for intersection tests.
module common;


////
// Imports

// Shaderlib prelude
import "cgv/prelude.slang";

// Linear algebra module
import "cgv/math/lin/common.slang";


////
// Module namespace

// Insert into cgv::geom::intersect namespace
public namespace cgv::geom::intersect {

// Import root namespace
using cgv;



//////
//
// Interfaces
//

/// An interface for representing information about the intersection point of a ray with something else, most commonly a surface.
///
/// By convention, we denote the ray parameter *l* instead of the more common *t* to delineate it from the parameter of other 1-dimensional entities we may intersect with, like curves (which often also use *t* as their parameter).
public interface IHit<Element: math::lin::IElement>
{
	/// The hitpoint.
	property point : Element;

	/// The normal of the intersected entity at the hitpoint.
	property normal : Element;

	/// The ray parameter *l* at the hit.
	property l : Element::Repr;
}



//////
//
// Structs
//

/// A common default implementation of the Hit interface.
public struct Hit<Element: math::lin::IElement>: IHit<Element>
{
	/// The hitpoint.
	public var point: Element;

	/// The normal of the intersected entity at the hitpoint.
	public var normal: Element;

	/// The ray parameter at the hit.
	public var l: Element::Repr;
}



//////
//
// Module finalization
//

// Module namespace close
}
