
//////
//
// Module definition
//

// A module providing various filtering functionality.
module filter;


//////
// Imports

// Linear algebra module
import "cgv/lin/common.slang";

// Local imports
import "cgv/gpu/filter/kernel.slang";


/////
// Module namespace

// Insert into cgv::gpu namespace
namespace cgv::gpu {



//////
//
// Functions
//

/// Filter the given tensor using the specified kernel.
/// # Arguments
/// * `res` – The resolution of the tensor to filter.
/// * `kernel` – The filter kernel to use.
/// * `writer` – The writer for filtered values with signature `func(pos: vector<int, Dims>, value: DataType)`.
/// * `reader` – The source tensor reader with signature `func(pos: vector<int, Dims>) -> DataType`.
func filter<DataType, int Dims, Kernel>(
	const res: vector<int, Dims>, kernel: Kernel,
	writer: IFunc<void, vector<int, Dims>, DataType>, reader: IFunc<DataType, vector<int, Dims>>
)
where DataType: IFloat, lin::IElement
where Kernel: filter::IKernel<DataType, Dims>
{
}




//////
//
// Module finalization
//

// Module namespace close
}
