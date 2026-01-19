
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

/// A zero-cost wrapper around [`Vec`] that dereferences to a slice of references to its element type. Very useful for
/// convenience functions that create an array of things that require caller ownership, but which the owner will most
/// typically just borrow to other functions (often as a slice of references).
///
/// # Example
///
/// ```
/// use cgv_util::ds::BorrowVec;
///
/// fn workOnStrings (strings: &[impl AsRef<str>]) {
///     for string in strings {
///         println!("worked on '{}'", string.as_ref());
///     }
/// }
///
/// let strings = BorrowVec::new(vec!["foo".to_string(), "bar".into(), "baz".into()]);
/// workOnStrings(strings.borrowed());
/// ```
pub struct BorrowVec<'this, T: 'this> {
	vec: Vec<T>,
	borrowVec: Vec<&'this T>,
}
impl<'this, T: 'this> BorrowVec<'this, T>
{
	/// Create the `BorrowVec` by moving in the given regular [`Vec`].
	///
	/// # Arguments
	///
	/// * `vec` – The regular vec to wrap with the new `BorrowVec`.
	///
	/// # Returns
	///
	/// The `BorrowVec` wrapping `vec`.
	#[inline(always)]
	pub fn new (vec: Vec<T>) -> Self {
		let slice = unsafe {
			// SAFETY: our struct will own this memory, so the references into said memory that it will keep cannot
			//         outlive it (the lifetimes match). Also, `BorrowVec` hides all its fields inside the private
			//         scope and defines no mutating methods, meaning Rust's aliasing rules are effectively never
			//         violated.
			&*(vec.as_slice() as *const [T])
		};
		Self { vec, borrowVec: slice.into_iter().map(|elem| elem).collect() }
	}

	/// Borrow a slice of the Vector of **owned** elements.
	pub fn owned (&self) -> &[T] {
		self.vec.as_slice()
	}

	/// Obtain a slice of the Vector of **borrowed** elements.
	pub fn borrowed (&self) -> &[&T] {
		self.borrowVec.as_slice()
	}
}
impl<'this, T: 'this> From<Vec<T>> for BorrowVec<'this, T> {
	#[inline(always)]
	fn from (value: Vec<T>) -> Self {
		Self::new(value)
	}
}
impl<'this, T: 'this> Deref for BorrowVec<'this, T> {
	type Target = [&'this T];

	#[inline(always)]
	fn deref (&self) -> &Self::Target {
		self.borrowVec.as_slice()
	}
}
