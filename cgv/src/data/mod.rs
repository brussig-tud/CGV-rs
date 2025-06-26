
//////
//
// Module definitions
//

/* nothing here yet */



//////
//
// Imports
//

// Standard library
use std::{hint::unreachable_unchecked, marker::PhantomData};

// GLM library
use glm;

// Local imports
use crate::*;



//////
//
// Structs and enums
//

/* nothing here yet */



//////
//
// Traits
//

/// A trait of vectors.
pub trait Vector<T: glm::Number> {
	fn dims () -> usize;
	fn component (&self, idx: usize) -> T;
}
impl<T: glm::Number> Vector<T> for glm::TVec1<T>
{
	#[inline(always)]
	fn dims () -> usize {
		1
	}

	#[inline(always)]
	fn component (&self, idx: usize) -> T {
		*self.get(idx).unwrap()
	}
}
impl<T: glm::Number> Vector<T> for glm::TVec2<T>
{
	#[inline(always)]
	fn dims () -> usize {
		2
	}

	#[inline(always)]
	fn component (&self, idx: usize) -> T {
		*self.get(idx).unwrap()
	}
}
impl<T: glm::Number> Vector<T> for glm::TVec3<T>
{
	#[inline(always)]
	fn dims () -> usize {
		3
	}

	#[inline(always)]
	fn component (&self, idx: usize) -> T {
		*self.get(idx).unwrap()
	}
}
impl<T: glm::Number> Vector<T> for glm::TVec4<T>
{
	#[inline(always)]
	fn dims () -> usize {
		4
	}

	#[inline(always)]
	fn component (&self, idx: usize) -> T {
		*self.get(idx).unwrap()
	}
}

/// The trait of being a vector that makes sense to homogenize in the context of real-time graphics.
pub trait Homogenizable<T: glm::Number, const N: usize>
	where Self: Vector<T>
{
	fn posH (&self) -> glm::TVec<T, {N+1}>;
	fn dirH (&self) -> glm::TVec<T, {N+1}>;
}
impl<T: glm::Number+From<u32>> Homogenizable<T, 1> for glm::TVec1<T>
{
	#[inline(always)]
	fn posH (&self) -> glm::TVec<T, 2> {
		glm::TVec2::<T>::new(self[0], 1.into())
	}

	#[inline(always)]
	fn dirH (&self) -> glm::TVec<T, 2> {
		glm::vec1_to_vec2(self)
	}
}
impl<T: glm::Number+From<u32>> Homogenizable<T, 2> for glm::TVec2<T>
{
	#[inline(always)]
	fn posH (&self) -> glm::TVec<T, 3> {
		glm::TVec3::<T>::new(self[0], self[1],1.into())
	}

	#[inline(always)]
	fn dirH (&self) -> glm::TVec<T, 3> {
		glm::vec2_to_vec3(self)
	}
}
impl<T: glm::Number+From<u32>> Homogenizable<T, 3> for glm::TVec3<T>
{
	#[inline(always)]
	fn posH (&self) -> glm::TVec<T, 4> {
		glm::TVec4::<T>::new(self[0], self[1], self[2], 1.into())
	}

	#[inline(always)]
	fn dirH (&self) -> glm::TVec<T, 4> {
		glm::vec3_to_vec4(self)
	}
}

/// An interface for an arbitrary transfer function.
pub trait TransferFunction<SourceType: Sized, OutputType: Sized>
{
	type SourceType = SourceType;
	type OutputType = OutputType;

	fn eval (&self, input: &SourceType) -> OutputType;
	fn wgslFnName(&self) -> String;
	fn wgslFn(&self) -> String;
}



//////
//
// Classes
//

/// A transfer function that linearly re-maps a scalar into a configurable range.
pub struct ScalarLinearRemap {
	inRange: glm::Vec2,
	outRange: glm::Vec2
}
impl ScalarLinearRemap {
	pub fn new (inRange: &glm::Vec2, outRange: &glm::Vec2) -> Self { Self {
		inRange: glm::vec2(inRange.x, inRange.y-inRange.x), outRange: *outRange
	}}
}
impl TransferFunction<f32, f32> for ScalarLinearRemap
{
	fn eval (&self, input: &f32) -> f32 {
		let t = (*input-self.inRange.x)/self.inRange.y;
		glm::lerp_scalar(t, self.outRange.x, self.outRange.y)
	}
	fn wgslFnName (&self) -> String {
		"tf_scalarLinearRemap".into()
	}
	fn wgslFn (&self) -> String {
		let rangeDecl = "";
		format!("fn {}(v: f32) -> f32 {{ {rangeDecl} return length(v); }}", self.wgslFnName())
	}
}

/// A transfer function that converts an *n*-dimensional vector into a *n+1*-dimensional homogenous position vector by
/// adding a `1` as the last component.
pub struct VectorToPositionH<const N: usize> {}
impl<const N: usize, Vector: Homogenizable<f32, N>> TransferFunction<Vector, glm::TVec<f32, {N+1}>>
for VectorToPositionH<N>
{
	/// Evaluate the transfer function for the given input.
	fn eval (&self, input: &Self::SourceType) -> Self::OutputType {
		input.posH()
	}
	fn wgslFnName (&self) -> String {
		format!("tf_vec{}fToPosH", Vector::dims())
	}
	fn wgslFn (&self) -> String
	{
		let fnName = format!("tf_vec{}fToPosH", Vector::dims());
		let inType = match Vector::dims() {
			1 => "vec1f", 2 => "vec2f", 3 => "vec3f", _ => unreachable!()
		};
		let outType = match Vector::dims() {
			1 => "vec2f", 2 => "vec3f", 3 => "vec4f", _ => unsafe { unreachable_unchecked() }
		};
		format!("fn {fnName}(v: {inType}) -> {outType} {{ return {outType}(v, 1); }}")
	}
}

/// A transfer function that converts an *n*-dimensional vector into a *n+1*-dimensional homogenous direction vector by
/// adding a `0` as the last component.
pub struct VectorToDirectionH<const N: usize> {}
impl<const N: usize, Vector: Homogenizable<f32, N>> TransferFunction<Vector, glm::TVec<f32, {N+1}>>
for VectorToDirectionH<N>
{
	/// Evaluate the transfer function for the given input.
	fn eval (&self, input: &Self::SourceType) -> Self::OutputType {
		input.dirH()
	}
	fn wgslFnName (&self) -> String {
		format!("tf_vec{}fToDirH", Vector::dims())
	}
	fn wgslFn (&self) -> String
	{
		let fnName = format!("tf_vec{}fToDirH", Vector::dims());
		let inType = match Vector::dims() {
			1 => "vec1f", 2 => "vec2f", 3 => "vec3f", _ => unreachable!()
		};
		let outType = match Vector::dims() {
			1 => "vec2f", 2 => "vec3f", 3 => "vec4f", _ => unsafe { unreachable_unchecked() }
		};
		format!("fn {fnName}(v: {inType}) -> {outType} {{ return {outType}(v, 1); }}")
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
	fn wgslFnName (&self) -> String {
		"tf_vectorNormL2".into()
	}
	fn wgslFn (&self) -> String {
		let inType = match N {
			1 => "vec2f", 2 => "vec2f", 3 => "vec3f", 4 => "vec4f",
			_ => unreachable!()
		};
		format!("fn {}(v: {inType}) -> f32 {{ return length(v); }}", self.wgslFnName())
	}
}

/// A transfer function that performs arbitrary code to map between values of two different types.
pub struct ArbitraryTransformation<SourceType, OutputType, Transformation>
	where SourceType: Sized+Clone, OutputType: Sized+Clone,
	      Transformation: Fn(&SourceType)->OutputType
{
	transformation: Transformation,
	wgslFnID: String,
	wgslSrcType: String,
	wgslOutType: String,
	wgslBody: String,
	_phantom_SourceType: PhantomData<SourceType>,
	_phantom_OutputType: PhantomData<OutputType>
}
impl<SourceType, OutputType, Transformation> ArbitraryTransformation<SourceType, OutputType, Transformation>
	where SourceType: Sized+Clone, OutputType: Sized+Clone,
	      Transformation: Fn(&SourceType)->OutputType
{
	pub fn new (
		transformation: Transformation, wgslFnID: impl AsRef<str>, wgslSrcType: impl AsRef<str>, wgslOutType: impl AsRef<str>,
		wgslBody: impl AsRef<str>
	) -> Self { Self {
		transformation, wgslFnID: wgslFnID.as_ref().to_owned(), wgslSrcType: wgslSrcType.as_ref().to_owned(),
		wgslOutType: wgslOutType.as_ref().to_owned(), wgslBody: wgslBody.as_ref().to_owned(),
		_phantom_SourceType: PhantomData, _phantom_OutputType: PhantomData,
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
	fn wgslFnName (&self) -> String {
		format!("tf_arbitrary_{}", self.wgslFnID)
	}
	fn wgslFn (&self) -> String {
		format!("fn {}(v: {}) -> {} {{ {} }}", self.wgslFnName(), self.wgslSrcType, self.wgslOutType, self.wgslBody)
	}
}
