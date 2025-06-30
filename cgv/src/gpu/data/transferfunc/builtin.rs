
//////
//
// Imports
//

// Standard library
use std::hint::unreachable_unchecked;

// GLM library
use glm;

// Local imports
use crate::*;
use gpu::data::{GPUTransferFunction, HasWGSLEquivalent};



//////
//
// Classes
//

/// Implementation for [`data::transferfunc::ScalarLinearRemap`].
impl<TSrc, TOut> GPUTransferFunction for data::transferfunc::ScalarLinearRemap<TSrc, TOut>
	where TSrc: glm::Number            + HasWGSLEquivalent,
	      TOut: glm::Number+From<TSrc> + HasWGSLEquivalent
{
	fn wgslFnName (&self) -> String {
		"tf_scalarLinearRemap".into()
	}
	fn wgslFnDef (&self) -> String {
		let rangeDecl = "";
		format!("fn {}(v: f32) -> f32 {{ {rangeDecl} return length(v); }}", self.wgslFnName())
	}
}

/// Implementation for [`data::transferfunc::VectorToPositionH`].
impl<T: glm::Number+From<isize>+HasWGSLEquivalent, const N: usize> GPUTransferFunction
for data::transferfunc::VectorToPositionH<T, N>
	where [(); N+1]:
{
	fn wgslFnName (&self) -> String {
		format!("tf_vec{N}fToPosH")
	}
	fn wgslFnDef (&self) -> String
	{
		let fnName = self.wgslFnName();
		let inType = match N {
			2 => "vec2f", 3 => "vec3f", _ => unreachable!()
		};
		let outType = match N {
			2 => "vec3f", 3 => "vec4f", _ => unsafe { unreachable_unchecked() }
		};
		format!("fn {fnName}(v: {inType}) -> {outType} {{ return {outType}(v, 1); }}")
	}
}

/// Implementation for [`data::transferfunc::VectorToDirectionH`].
impl<T: glm::Number+From<isize>+HasWGSLEquivalent, const N: usize> GPUTransferFunction
for data::transferfunc::VectorToDirectionH<T, N>
	where [(); N+1]:
{
	fn wgslFnName (&self) -> String {
		format!("tf_vec{N}fToDirH")
	}
	fn wgslFnDef (&self) -> String
	{
		let fnName = self.wgslFnName();
		let inType = match N {
			2 => "vec2f", 3 => "vec3f", _ => unreachable!()
		};
		let outType = match N {
			2 => "vec3f", 3 => "vec4f", _ => unsafe { unreachable_unchecked() }
		};
		format!("fn {fnName}(v: {inType}) -> {outType} {{ return {outType}(v, 0); }}")
	}
}

/// Implementation for [`data::transferfunc::VectorNormL2`].
impl<T: glm::Number+HasWGSLEquivalent, const N: usize> GPUTransferFunction for data::transferfunc::VectorNormL2<T, N>
{
	fn wgslFnName (&self) -> String {
		"tf_vectorNormL2".into()
	}
	fn wgslFnDef (&self) -> String {
		let inType = match N {
			1 => "vec2f", 2 => "vec2f", 3 => "vec3f", 4 => "vec4f",
			_ => unreachable!()
		};
		format!("fn {}(v: {inType}) -> f32 {{ return length(v); }}", self.wgslFnName())
	}
}
