
//////
//
// Module definition
//

// A module providing various filter kernel abstractions and implementations.
module kernel;


//////
// Imports

// Metaprogramming module
import "cgv/meta.slang";

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

/// A 3^*N* polyphase box filter kernel, suitable e.g. for calculating non-power-of-2 mip maps. The hardcoded per-dimension phase shifts assume a reduction from *n* to *n*' = *floor*(*n*/2).
///
/// **TODO: Generalize to arbitrary reduction from first principles by convolving appropriately scaled box filter with a box kernel of uniform extent 1.**
public struct PolyphaseBox3<DataType, int N> : IKernel<DataType, N>
	where DataType: IFloat, lin::IElement
{
	/// The underlying scalar type.
	public typealias Scalar = DataType::Repr;

	/// The vector-valued integer index into data.
	public typealias Index = vector<int, N>;

	/// The number 3^*N* of elements in the box filter kernel.
	public static const int KERNEL_SIZE = meta::pow<3, N>();

	/// The size of the data domain, which decides the phase shift per pixel.
	public vector<int, N> size;

	/// Filter the value at the query point `q` through the given data accessor.
	/// # Arguments
	/// * `q` – The query point inside the data source.
	/// * `data` – The data accessor with signature `func(q: vector<int, Dims>) -> DataType`.
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
		return val.scale(Scalar(1)/Scalar(KERNEL_SIZE));
	}

	/// Construct for the given target size.
	public __init (vector<int, N> size) {
		this.size = size;
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
	sizes: vector<int, Dims>, q: vector<int, Dims>, data: IFunc<DataType, vector<int, Dims>>
) -> DataType
	where DataType: IFloat, lin::IElement
{
	let kernel = PolyphaseBox3<DataType, Dims>(sizes);
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
