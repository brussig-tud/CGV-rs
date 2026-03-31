
//////
//
// Module setup
//

/// Functionality for interacting with textures.
module texture;


//////
// Imports

// Local imports
import "cgv/math/util.slang";


/////
// Module namespace

// Insert into cgv::media::tex namespace
public namespace cgv::media::tex {



//////
//
// Enums
//

/// Enumeration of currently supported texture formats.
///
/// **NOTE**: The enum values are being manually synced to the (unstable) `format` magic values used by *Slang*.
public enum Format: int
{
	/// A 4 channel 32-bit floating point texture.
	RGBA32F = 1,

	/// A 4 channel 16-bit floating point texture.
	RGBA16F = 2,

	/// A 2 channel 32-bit floating point texture.
	RG32F = 3,

	/// A 2 channel 16-bit floating point texture.
	RG16F = 4,

	/// A 3 channel 11/11/10-bit floating point texture.
	R11F_G11F_B10F = 5,

	/// A 1 channel 32-bit floating point texture.
	R32F = 6,

	/// A 1 channel 16-bit floating point texture.
	R16F = 7,

	/// A 4 channel 16-bit normalized unsigned integer texture.
	RGBA16 = 8,

	/// A 4 channel 10/10/10/2-bit normalized unsigned integer texture.
	RGB10_A2 = 9,

	/// A 4 channel 8-bit normalized unsigned integer texture.
	RGBA8 = 10,

	/// A 2 channel 16-bit normalized unsigned integer texture.
	RG16 = 11,

	/// A 2 channel 8-bit normalized unsigned integer texture.
	RG8 = 12,

	/// A 1 channel 16-bit normalized unsigned integer texture.
	R16 = 13,

	/// A 1 channel 8-bit normalized unsigned integer texture.
	R8 = 14,

	/// A 4 channel 16-bit normalized signed integer texture.
	RGBA16_SNORM = 15,

	/// A 4 channel 8-bit normalized signed integer texture.
	RGBA8_SNORM = 16,

	/// A 2 channel 16-bit normalized signed integer texture.
	RG16_SNORM = 17,

	/// A 2 channel 8-bit normalized signed integer texture.
	RG8_SNORM = 18,

	/// A 1 channel 16-bit normalized signed integer texture.
	R16_SNORM = 19,

	/// A 1 channel 8-bit normalized signed integer texture.
	R8_SNORM = 20,

	/// A 4 channel 32-bit signed integer texture.
	RGBA32I = 21,

	/// A 4 channel 16-bit signed integer texture.
	RGBA16I = 22,

	/// A 4 channel 8-bit signed integer texture.
	RGBA8I = 23,

	/// A 2 channel 32-bit signed integer texture.
	RG32I = 24,

	/// A 2 channel 16-bit signed integer texture.
	RG16I = 25,

	/// A 2 channel 8-bit signed integer texture.
	RG8I = 26,

	/// A 1 channel 32-bit signed integer texture.
	R32I = 27,

	/// A 1 channel 16-bit signed integer texture.
	R16I = 28,

	/// A 1 channel 8-bit signed integer texture.
	R8I = 29,

	/// A 4 channel 32-bit unsigned integer texture.
	RGBA32UI = 30,

	/// A 4 channel 16-bit unsigned integer texture.
	RGBA16UI = 31,

	/// A 4 channel 10/10/10/2-bit unsigned integer texture.
	RGB10_A2UI = 32,

	/// A 4 channel 8-bit unsigned integer texture.
	RGBA8UI = 33,

	/// A 2 channel 32-bit unsigned integer texture.
	RG32UI = 34,

	/// A 2 channel 16-bit unsigned integer texture.
	RG16UI = 35,

	/// A 2 channel 8-bit unsigned integer texture.
	RG8UI = 36,

	/// A 1 channel 32-bit unsigned integer texture.
	R32UI = 37,

	/// A 1 channel 16-bit unsigned integer texture.
	R16UI = 38,

	/// A 1 channel 8-bit unsigned integer texture.
	R8UI = 39,

	/// A 1 channel 64-bit unsigned integer texture.
	R64UI = 40,

	/// A 1 channel 64-bit signed integer texture.
	R64I = 41
}

/// Return the resolution of the given `_Texture` object.
/// # Parameters
/// * `tex` – The texture to query the resolution of.
/// # Returns
/// The resolution of the given texture.
[ForceInline]
public func getResolution<T: ITexelElement, int access, int isShadow, int isCombined, int format> (
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
public func getResolution<T: ITexelElement, int access, int isShadow, int isCombined, int format> (
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
public func getResolution<T: ITexelElement, int access, int isShadow, int isCombined, int format> (
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
public func areDimsPow2<T: ITexelElement, int access, int isShadow, int isCombined, int format> (
	tex: _Texture<
		T, __Shape1D, /* isArray: */ 0, /* isMS: */ 0, /* sampleCount: */0, access, isShadow, isCombined, format
	>
) -> bool1 {
	return cgv::math::isPow2(getResolution(tex));
}

/// Checks for each dimension of the given `_Texture` object whether the corresponding resolution is a power of 2.
/// # Parameters
/// * `tex` – The texture to check.
/// # Returns
/// A vector of booleans indicating whether the corresponding dimension is a power of 2.
[ForceInline]
public func areDimsPow2<T: ITexelElement, int access, int isShadow, int isCombined, int format> (
	tex: _Texture<
		T, __Shape2D, /* isArray: */ 0, /* isMS: */ 0, /* sampleCount: */0, access, isShadow, isCombined, format
	>
) -> bool2 {
	return cgv::math::isPow2(getResolution(tex));
}

/// Checks for each dimension of the given `_Texture` object whether the corresponding resolution is a power of 2.
/// # Parameters
/// * `tex` – The texture to check.
/// # Returns
/// A vector of booleans indicating whether the corresponding dimension is a power of 2.
[ForceInline]
public func areDimsPow2<T: ITexelElement, int access, int isShadow, int isCombined, int format> (
	tex: _Texture<
		T, __Shape3D, /* isArray: */ 0, /* isMS: */ 0, /* sampleCount: */0, access, isShadow, isCombined, format
	>
) -> bool3{
	return cgv::math::isPow2(getResolution(tex));
}



//////
//
// Module finalization
//

// Module namespace close
}
