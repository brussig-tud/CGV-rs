
//////
//
// Imports
//

// Standard library
use std::marker::PhantomData;

// GLM library
use glm;

// Local imports
use crate::*;
use math::DimensionsExt;



//////
//
// Traits
//

/// An interface for an arbitrary transfer function.
pub trait TransferFunction<SourceType: Sized, OutputType: Sized>
{
	type SourceType = SourceType;
	type OutputType = OutputType;

	fn eval (&self, input: &SourceType) -> OutputType;
}



//////
//
// Classes
//

/// A transfer function that linearly re-maps a scalar into a configurable range.
pub struct ScalarLinearRemap<TSrc: glm::Number, TOut: glm::Number+From<TSrc>>
{
	inRange: glm::TVec2<TSrc>,
	outRange: glm::TVec2<TOut>
}
impl<TSrc: glm::Number, TOut: glm::Number+From<TSrc>> ScalarLinearRemap<TSrc, TOut> {
	pub fn new (inRange: &glm::TVec2<TSrc>, outRange: &glm::TVec2<TOut>) -> Self { Self {
		inRange: glm::vec2(inRange.x, inRange.y-inRange.x), outRange: *outRange
	}}
}
impl<TSrc: glm::Number, TOut: glm::Number+From<TSrc>> TransferFunction<TSrc, TOut> for ScalarLinearRemap<TSrc, TOut>
{
	fn eval (&self, input: &TSrc) -> TOut {
		let t = (*input-self.inRange.x)/self.inRange.y;
		glm::lerp_scalar(t.into(), self.outRange.x, self.outRange.y)
	}
}

/// A transfer function that converts an *n*-dimensional vector into a *n+1*-dimensional homogenous position vector by
/// adding a `1` as the last component.
pub struct VectorToPositionH<T: glm::Number+From<isize>, const N: usize> where [(); N+1]: {
	_phantom: PhantomData<T>
}
impl<T: glm::Number+From<isize>, const N: usize> TransferFunction<glm::TVec<T, N>, glm::TVec<T, {N+1}>>
for VectorToPositionH<T, N> where [(); N+1]:
{
	/// Evaluate the transfer function for the given input.
	fn eval (&self, input: &Self::SourceType) -> Self::OutputType {
		input.addComponent(1.into())
	}
}

/// A transfer function that converts an *n*-dimensional vector into a *n+1*-dimensional homogenous direction vector by
/// adding a `0` as the last component.
pub struct VectorToDirectionH<T: glm::Number+From<isize>, const N: usize> where [(); N+1]: {
	_phantom: PhantomData<T>
}
impl<T: glm::Number+From<isize>, const N: usize> TransferFunction<glm::TVec<T, N>, glm::TVec<T, {N+1}>>
for VectorToDirectionH<T, N> where [(); N+1]:
{
	/// Evaluate the transfer function for the given input.
	fn eval (&self, input: &Self::SourceType) -> Self::OutputType {
		input.addComponent(0.into())
	}
}

/// A transfer function that maps an *n*-dimensional vector to its L2-norm.
pub struct VectorNormL2<T: glm::Number, const N: usize> {
	_phantom_T: PhantomData<T>
}
impl<T, const N: usize> TransferFunction<glm::TVec<T, N>, T> for VectorNormL2<T, N>
where T: glm::Number + nalgebra::SimdComplexField<SimdRealField=T>
{
	/// Evaluate the transfer function for the given input.
	fn eval (&self, input: &Self::SourceType) -> T {
		input.norm()
	}
}

/// A transfer function that performs arbitrary code to map between values of two different types.
pub struct ArbitraryTransformation<SourceType, OutputType, Transformation>
where SourceType: Sized+Clone, OutputType: Sized+Clone,
      Transformation: Fn(&SourceType)->OutputType
{
	transformation: Transformation,
	_phantom: PhantomData<(SourceType, OutputType)>
}
impl<SourceType, OutputType, Transformation> ArbitraryTransformation<SourceType, OutputType, Transformation>
where SourceType: Sized+Clone, OutputType: Sized+Clone,
      Transformation: Fn(&SourceType)->OutputType
{
	pub fn new (transformation: Transformation) -> Self { Self {
		transformation, _phantom: PhantomData
	}}
}
impl<SourceType, OutputType, Transformation> TransferFunction<SourceType, OutputType>
for ArbitraryTransformation<SourceType, OutputType, Transformation>
where SourceType: Sized+Clone, OutputType: Sized+Clone,
      Transformation: Fn(&SourceType)->OutputType
{
	fn eval (&self, input: &SourceType) -> OutputType {
		(&self.transformation)(input)
	}
}
