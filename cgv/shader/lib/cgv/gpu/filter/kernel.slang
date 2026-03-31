
//////
//
// Module definition
//

// A module providing various filter kernel abstractions and implementations.
module kernel;


//////
// Imports

// Shaderlib prelude
import "cgv/prelude.slang";

// Metaprogramming module
import "cgv/meta.slang";

// Math module
import "cgv/math/util.slang";

// Linear algebra module
import "cgv/lin/common.slang";


/////
// Module namespace

// Insert into cgv::gpu::filter namespace
namespace cgv::gpu::filter {



//////
//
// Interfaces
//

/// The generic interface of all filter kernels. The data the kernel is applied to is provided via a generic accessor
/// to facilitate arbitrary data sources.
public interface IKernel<DataType, int Dims>
	where DataType: IFloat, lin::IElement
{
	/// Filter the value at the query point `q` through the given data accessor.
	/// # Arguments
	/// * `q` – The query point inside the data source.
	/// * `data` – The data accessor with signature `func(q: vector<int, Dims>) -> DataType`.
	/// # Returns
	/// The filtered value.
	func apply (q: vector<int, Dims>, data: IFunc<DataType, vector<int, Dims>>) -> DataType;
};



//////
//
// Structs
//

////
// Sub-namespaces

/// Namespace providing some standard kernels.
public namespace kernel {

/// A simple 2^*N* box filter kernel, suitable e.g. for calculating power-of-2 mip maps.
public struct Box2<DataType, int N> : IKernel<DataType, N>
	where DataType: IFloat, lin::IElement
{
	/// The underlying number/scalar type.
	public typealias Scalar = DataType::Repr;

	/// The vector-valued integer index into data.
	public typealias Index = vector<int, N>;

	/// The number 2^*N* of elements in the box filter kernel.
	public static const int KERNEL_SIZE = meta::pow2<N>();

	/// The resulting (constant) weight of each element in the kernel.
	public static const Scalar WEIGHT = Scalar(1)/Scalar(KERNEL_SIZE);

	/// Filter the value at the query point `q` through the given data accessor.
	/// # Arguments
	/// * `q` – The query point inside the data source.
	/// * `data` – The data accessor with signature `func(q: vector<int, N>) -> DataType`.
	/// # Returns
	/// The filtered value.
	override func apply (q: vector<int, N>, data: IFunc<DataType, vector<int, N>>) -> DataType
	{
		// Convolve value with 2^N box kernel
		var val = DataType(0);
		for (int i=0; i<KERNEL_SIZE; ++i) {
			var ki = Index(0);
			[ForceUnroll]
			for (int d=0; d<N; ++d) // Since each dimension is 2 wide, the per-dimension indices correspond to the
				ki[d] = (i>>d) & 1;    // bits of i.
			val = val.add(data(q+ki));
		}
		return val.scale(WEIGHT);
	}
}

/// **TODO: Factor out into appropriate utility module**
[ForceInline]
constexpr func i2coord<int N, int EXTENT> (in const i: int) -> vector<int, N>
{
	var i = i;
	var result: vector<int, N>;
	[ForceUnroll]
	for (int d=0; d<N; ++d) {
		result[d] = i % EXTENT;
		i /= EXTENT;
 	}
 	return result;
}

/// **TODO: Factor out into appropriate utility module**
[ForceInline]
func odometer<int EXTENT, int N> (inout index: vector<int, N>)
{
	[ForceUnroll]
	for (int d=0; d<N; ++d) {
		index[d]++;
		if (index[d] < EXTENT)
			return;
		index[d] = 0;
	}
}

/// **TODO: Factor out into appropriate utility module**
func tensorProductAt<T: __BuiltinFloatingPointType, int Extent, int N> (
	in const vectors: vector<T, Extent>[N], in const index: vector<int, N>
) -> T
{
	// Precondition: index is within bounds, i.e. index[d] < Extent for all d
	var elem = T(1);
	[ForceUnroll]
	for (int d=0; d<N; ++d)
		elem = elem * vectors[d][index[d]];
	return elem;
}

/// **TODO: Factor out into appropriate utility module**
func tensorProduct2<T: __BuiltinFloatingPointType, int Extent> (in const vectors: vector<T, Extent>[2])
	-> matrix<T, Extent, Extent>
{
	var tensor: matrix<T, Extent, Extent>;
	[ForceUnroll]
	for (int j=0; j<Extent; ++j)
		[ForceUnroll]
		for (int i=0; i<Extent; ++i)
			tensor[j][i] = tensorProductAt(vectors, int2(i, j));
	return tensor;
}

/// A 3^*N* polyphase box filter kernel, suitable e.g. for calculating non-power-of-2 mip maps. The hardcoded per-dimension phase shifts assume a reduction from *n* to *n*' = *floor*(*n*/2).
///
/// **TODO: Generalize to arbitrary reduction from first principles by convolving appropriately scaled box filter with a box kernel of uniform extent 1.**
public struct PolyphaseBox3<DataType, int N> : IKernel<DataType, N>
	where DataType: IFloat, lin::IElement
{
	/// Enum indicating whether a resolution along a dimension is divisible by 2.
	public enum DimensionMode {
		Div2=0, NonDiv2=1, One=2
	};

	/// The underlying scalar type.
	public typealias Scalar = DataType::Repr;

	/// A 3-vector of the underlying scalar type.
	public typealias Vec3 = cgv::Vec3<Scalar>;

	/// The vector-valued integer index into data.
	public typealias Index = vector<int, N>;

	/// The number 3^*N* of elements in the box filter kernel.
	public static const int KERNEL_SIZE = meta::pow<3, N>();

	/// The set of single-dimension power-of-2 box filter weights for use in our 3-wide kernel.
	public static const Vec3 BOX_WEIGHTS = Vec3(Scalar(.5), Scalar(.5), Scalar(0));

	/// The set of single-dimension weights that correspond to an identity/no-op filter.
	public static const Vec3 IDENTITY_WEIGHTS = Vec3(Scalar(1), Scalar(0), Scalar(0));

	/// The size of the data domain, which decides the phase shift per pixel.
	public vector<int, N> size;

	/// Which dimensions of the source data have a power-of-2 resolution.
	/// **TODO: Use the `DimensionMode` enum once vectors of enums are fixed.**
	public vector<int, N> mode;

	/// Filter the value at the query point `q` through the given data accessor.
	/// # Arguments
	/// * `q` – The query point inside the data source.
	/// * `data` – The data accessor with signature `func(q: vector<int, Dims>) -> DataType`.
	/// # Returns
	/// The filtered value.
	override func apply (q: vector<int, N>, data: IFunc<DataType, vector<int, N>>) -> DataType
	{
		// Compute dimensional weights that form the actual filter kernel via their tensor product (as polyphase filters
		// depend on the location `q` being filtered, the filter kernel can not be pre-computed).
		//
		// We get one such 3-wide weight vector w_d per dimension d. For d=2, the actual 3² kernel at the current
		// location would result from the tensor product w_x ⊗ w_y:
		//
		//   w_y1 * w_x1  │  w_y1 * w_x2  │  w_y1 * w_x3
		//  ──────────────┼───────────────┼───────────────
		//   w_y2 * w_x1  │  w_y2 * w_x2  │  w_y2 * w_x3
		//  ──────────────┼───────────────┼───────────────
		//   w_y3 * w_x1  │  w_y3 * w_x2  │  w_y3 * w_x3
		//
		var weights: vector<Scalar, 3>[N]; // <- we ideally want to use a matrix here, but Slang is weird with its row
		                                   //    and column semantics; arrays are clearer and unambiguous.
		for (int d=0; d<N; ++d)
			if (mode[d] == DimensionMode::Div2)
				// For divisible-by-2 dimensions, we fall back to the 2-wide box filter
				weights[d] = BOX_WEIGHTS;
			else if (mode[d] == DimensionMode::NonDiv2)
			{
				// For non-divisible-by-2 dimensions, we use a 3-wide trapezoidal filter (the result of reconstructing
				// the box filter on the misaligned output grid) phase-shifted according to the respective coordinate in
				// the output domain.
				let p = q[d]>>1;
				let tgtSize = size[d]>>1;
				weights[d][0] = Scalar(tgtSize-p);
				weights[d][1] = Scalar(tgtSize  );
				weights[d][2] = Scalar(   1   +p);
				weights[d] *= Scalar(1)/Scalar(size[d]);
			}
			else // DimensionMode::One
				weights[d] = IDENTITY_WEIGHTS;

		// Convolve value with 3^N kernel
		var val = DataType(0);
		var ki = Index(0); // <- "k"ernel "i"ndex – index to an element inside the kernel tensor
		for (int i=0; i<KERNEL_SIZE; ++i) {
			// The filter kernel with the final weights is the tensor product of the per-dimension weight vectors (see
			// explanation above). We're using each element ad-hoc rather than pre-computing the full kernel tensor.
			let weight = tensorProductAt(/* weightVecs: */ weights, /* tensorElem: */ki);
			val = val.add(data(q+ki).scale(weight));
			odometer<3>(ki); // <- compute next kernel index
		}
		return val;
	}

	/// Construct for the given input data size.
	public __init (vector<int, N> dataSize)
	{
		size = dataSize;
		[ForceUnroll]
		for (int d=0; d<N; ++d)
			mode[d] = math::isDiv2(dataSize[d]) ? DimensionMode::Div2 : (
				dataSize[d] != 1 ? DimensionMode::NonDiv2 : DimensionMode::One
			);
	}
}

/// Function-style access to the `Box2` filter kernel.
[ForceInline]
public func box2<DataType, int Dims> (q: vector<int, Dims>, data: IFunc<DataType, vector<int, Dims>>) -> DataType
	where DataType: IFloat, lin::IElement
{
	static constexpr Box2<DataType, Dims> kernel;
	return kernel.apply(q, data);
}

/// Function-style access to the `PolyphaseBox3` filter kernel.
[ForceInline]
public func polyphaseBox3<DataType, int Dims> (
	origSize: vector<int, Dims>, q: vector<int, Dims>, data: IFunc<DataType, vector<int, Dims>>
) -> DataType
	where DataType: IFloat, lin::IElement
{
	let kernel = PolyphaseBox3<DataType, Dims>(origSize);
	return kernel.apply(q, data);
}

// Sub-namespace close
}



//////
//
// Module finalization
//

// Module namespace close
}
