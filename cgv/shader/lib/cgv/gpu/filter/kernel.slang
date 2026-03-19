
//////
//
// Module definition
//

// A module providing various filter kernel abstractions and implementations.
module kernel;


//////
// Imports

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

/// A simple 2^*N* box filter kernel, suitable e.g. for calculating power-of-two mip maps.
public struct Box2<DataType, int Dims> : IKernel<DataType, Dims>
	where DataType: IFloat, lin::IElement
{
	/// The vector-valued integer index into data.
	public typealias Index = vector<int, Dims>;

	/// The number of elements in the box filter kernel.
	public static constexpr int KERNEL_SIZE = int(powr(2, Dims));

	/// The resulting (constant) weight of each element in the kernel.
	public static constexpr float WEIGHT = 1/float(KERNEL_SIZE);

	/// Filter the value at the query point `q` through the given data accessor.
	/// # Arguments
	/// * `q` – The query point inside the data source.
	/// * `data` – The data accessor with signature `func(q: vector<int, Dims>) -> DataType`.
	/// # Returns
	/// The filtered value.
	override func apply (q: vector<int, Dims>, data: IFunc<DataType, vector<int, Dims>>) -> DataType
	{
        // Work around bug in emitted Slang debug info by mirroring with a local constant
		let weight = DataType::Repr(WEIGHT);

		// Convolve value with 2^N box kernel
		var val = DataType(0);
		for (int i=0; i<KERNEL_SIZE; ++i) {
			var ki = Index(0);
			[ForceUnroll]
			for (int d=0; d<Dims; ++d) // Since each dimension is 2 wide, the per-dimension indices correspond to the
				ki[d] = (i>>d) & 1;    // bits of i.
			val = val.add(data(q+ki).scale(weight));
		}

		// Done!
		return val;
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

// Sub-namespace close
}



//////
//
// Module finalization
//

// Module namespace close
}
