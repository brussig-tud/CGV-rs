
//////
//
// Module setup
//

/// Assorted media utilities.
module util;


//////
// Imports

// Local imports
import "cgv/math/util.slang";


/////
// Module namespace

// Insert into cgv::media namespace
public namespace cgv::media {



//////
//
// Functions
//

/// Return the resolution of the given `_Texture` object.
/// # Parameters
/// * `tex` – The texture to query the resolution of.
/// # Returns
/// The resolution of the given texture.
[ForceInline]
public func getTexResolution<T: ITexelElement, int access, int isShadow, int isCombined, int format> (
	tex: _Texture<
		T, __Shape1D, /* isArray: */ 0, /* isMS: */ 0, /* sampleCount: */0, access, isShadow, isCombined, format
	>
) -> int1 {
	var width: int;
	tex.GetDimensions(width);
	return int1(width);
}

/// Return the resolution of the given `_Texture` object.
/// # Parameters
/// * `tex` – The texture to query the resolution of.
/// # Returns
/// The resolution of the given texture.
[ForceInline]
public func getTexResolution<T: ITexelElement, int access, int isShadow, int isCombined, int format> (
	tex: _Texture<
		T, __Shape2D, /* isArray: */ 0, /* isMS: */ 0, /* sampleCount: */0, access, isShadow, isCombined, format
	>
) -> int2 {
	int width, height;
	tex.GetDimensions(width, height);
	return int2(width, height);
}

/// Return the resolution of the given `_Texture` object.
/// # Parameters
/// * `tex` – The texture to query the resolution of.
/// # Returns
/// The resolution of the given texture.
[ForceInline]
public func getTexResolution<T: ITexelElement, int access, int isShadow, int isCombined, int format> (
	tex: _Texture<
		T, __Shape3D, /* isArray: */ 0, /* isMS: */ 0, /* sampleCount: */0, access, isShadow, isCombined, format
	>
) -> int3 {
	int width, height, depth;
	tex.GetDimensions(width, height, depth);
	return int3(width, height, depth);
}

/// Checks for each dimension of the given `_Texture` object whether the corresponding resolution is a power of 2.
/// # Parameters
/// * `tex` – The texture to check.
/// # Returns
/// A vector of booleans indicating whether the corresponding dimension is a power of 2.
[ForceInline]
public func areTexDimsPow2<T: ITexelElement, int access, int isShadow, int isCombined, int format> (
	tex: _Texture<
		T, __Shape1D, /* isArray: */ 0, /* isMS: */ 0, /* sampleCount: */0, access, isShadow, isCombined, format
	>
) -> bool1 {
	return cgv::math::isPow2(getTexResolution(tex));
}

/// Checks for each dimension of the given `_Texture` object whether the corresponding resolution is a power of 2.
/// # Parameters
/// * `tex` – The texture to check.
/// # Returns
/// A vector of booleans indicating whether the corresponding dimension is a power of 2.
[ForceInline]
public func areTexDimsPow2<T: ITexelElement, int access, int isShadow, int isCombined, int format> (
	tex: _Texture<
		T, __Shape2D, /* isArray: */ 0, /* isMS: */ 0, /* sampleCount: */0, access, isShadow, isCombined, format
	>
) -> bool2 {
	return cgv::math::isPow2(getTexResolution(tex));
}

/// Checks for each dimension of the given `_Texture` object whether the corresponding resolution is a power of 2.
/// # Parameters
/// * `tex` – The texture to check.
/// # Returns
/// A vector of booleans indicating whether the corresponding dimension is a power of 2.
[ForceInline]
public func areTexDimsPow2<T: ITexelElement, int access, int isShadow, int isCombined, int format> (
	tex: _Texture<
		T, __Shape3D, /* isArray: */ 0, /* isMS: */ 0, /* sampleCount: */0, access, isShadow, isCombined, format
	>
) -> bool3{
	return cgv::math::isPow2(getTexResolution(tex));
}



//////
//
// Module finalization
//

// Module namespace close
}
