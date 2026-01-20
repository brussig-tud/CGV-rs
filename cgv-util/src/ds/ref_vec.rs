
//////
//
// Imports
//

// Standard library
use std::ops::Deref;



//////
//
// Structs
//

/// A zero-cost* convenience wrapper around [`Vec`] that [derefs](Deref) to a slice of references to the stored
/// elements. Very useful in places where you need to call a function that needs such a slice of references (as is often
/// the case when interacting with C libraries). If you borrow this slice more than once, it naturally saves conversion
/// costs without additional effort on your part as the slice of references is created only once
/// [during construction](RefVec::new).
///
/// # Example
///
/// ```
/// use cgv_util::ds::RefVec;
///
/// #[derive(Debug)]
/// struct LargeStruct(u128);
///
/// fn workOnThings (things: &[&LargeStruct]) {
///     for thing in things {
///         println!("worked on '{thing:?}'");
///     }
/// }
/// fn workMoreOnThings (things: &[&LargeStruct]) {
///     for thing in things {
///         println!("worked some more on '{thing:?}'");
///     }
/// }
///
/// let things = RefVec::new(vec![LargeStruct(0), LargeStruct(1), LargeStruct(2), LargeStruct(3)]);
/// workOnThings(&things);
/// workMoreOnThings(&things); // <- saved one conversion
/// ```
///
/// ##### Footnotes
///
/// \*It is zero-cost only if you actually use the slice of references it derefs to, otherwise the construction overhead
/// is not amortized.
pub struct RefVec<'this, T: 'this> {
	vec: Vec<T>,
	refs: Vec<&'this T>,
}
impl<'this, T: 'this> RefVec<'this, T>
{
	/// Create the `RefVec` by moving in the given regular [`Vec`].
	///
	/// # Arguments
	///
	/// * `vec` – The regular vec to wrap with the new `RefVec`.
	///
	/// # Returns
	///
	/// The `RefVec` wrapping `vec`.
	#[inline(always)]
	pub fn new (vec: Vec<T>) -> Self {
		let slice = unsafe {
			// SAFETY: our struct will own this memory, so the references into said memory that it will keep cannot
			//         outlive it (the lifetimes match). Also, the memory is on the heap and thus remains stable even
			//         when the owning object `self.vec` is moved. Finally, `RefVec` hides all its fields inside the
			//         private scope and defines no mutating methods, meaning Rust's aliasing rules are effectively
			//         never violated.
			&*(vec.as_slice() as *const [T])
		};
		Self { vec, refs: slice.into_iter().map(|elem| elem).collect() }
	}

	/// Borrow a slice over the Vector of **owned** elements.
	#[inline(always)]
	pub fn elements (&self) -> &[T] {
		self.vec.as_slice()
	}

	/// Borrow a slice over the Vector of **references** to the elements.
	#[inline(always)]
	pub fn references (&self) -> &[&'this T] {
		self.refs.as_slice()
	}
}
impl<'this, T: 'this> From<Vec<T>> for RefVec<'this, T> {
	#[inline(always)]
	fn from (value: Vec<T>) -> Self {
		Self::new(value)
	}
}
impl<'this, T: 'this> Deref for RefVec<'this, T> {
	type Target = [&'this T];

	#[inline(always)]
	fn deref (&self) -> &Self::Target {
		self.references()
	}
}
