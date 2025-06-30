
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// GLM library
use glm;

// Local imports
use crate::*;



//////
//
// Traits
//

/// An extension trait adding various functionality for manipulating *GLM* [`Vector`] dimensionality.
pub trait DimensionsExt<T: glm::Number+From<isize>, const N: usize>
{
	fn addComponent (&self, c: T) -> glm::TVec<T, {N+1}>
		where [(); N+1]:; // make sure N+1 is still a valid usize (i.e. does not overflow)

	fn concat<const M: usize> (&self, added: &glm::TVec<T, M>) -> glm::TVec<T, {N+M}>
		where [(); N+M]:; // make sure N+M is still a valid usize (i.e. does not overflow)
}

impl<T: glm::Number+From<isize>, const N: usize> DimensionsExt<T, N> for glm::TVec<T, N>
{
	fn addComponent (&self, c: T) -> glm::TVec<T, {N+1}>
		where [(); N+1]: // make sure N+1 is still a valid usize (i.e. does not overflow)
	{
		self.concat(&glm::TVec1::from_element(c))
	}

	fn concat<const M: usize> (&self, added: &glm::TVec<T, M>) -> glm::TVec<T, {N+M}>
		where [(); N+M]: // make sure N+M is still a valid usize (i.e. does not overflow)
	{
		let mut newVec = std::mem::MaybeUninit::<glm::TVec::<T, {N+M}>>::uninit(); {
			let newVec = unsafe {
				// SAFETY: We know that newVec holds correctly sized and aligned memory for a `glm::TVec::<T, {N+M}>`
				&mut *(newVec.as_mut_ptr() as *mut T as *mut [T; N+M])
			};
			for i in 0..N {
				newVec[i] = self[i];
			}
			for i in 0..M {
				newVec[N+i] = added[i];
			}
		}
		unsafe {
			// SAFETY: we initialized all N+M components just above
			newVec.assume_init()
		}
	}
}
