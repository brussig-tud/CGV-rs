
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
/// elements. Very useful in places where you need to call a function that needs such a slice of references. If you
/// borrow this slice more than once, it naturally saves conversion costs without additional effort on your part as the
/// slice of references is created only once [during construction](BorrowVec::new).
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
/// fn workMoreOnStrings (strings: &[impl AsRef<str>]) {
///     for string in strings {
///         println!("worked some more on '{}'", string.as_ref());
///     }
/// }
///
/// let strings = BorrowVec::new(vec!["foo".to_string(), "bar".into(), "baz".into()]);
/// workOnStrings(&*strings);
/// workMoreOnStrings(&*strings); // <- saved one conversion
/// ```
///
/// ##### Footnotes
///
/// \*It is zero-cost only if you actually use the slice of references it derefs to, otherwise you incur a small
/// construction overhead that serves you no practical purpose.
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
			//         outlive it (the lifetimes match). Also, the memory is on the heap and thus remains stable even
			//         when the owning object `self.vec` is moved. Finally, `BorrowVec` hides all its fields inside the
			//         private scope and defines no mutating methods, meaning Rust's aliasing rules are effectively
			//         never violated.
			&*(vec.as_slice() as *const [T])
		};
		Self { vec, borrowVec: slice.into_iter().map(|elem| elem).collect() }
	}

	/// Borrow a slice of the Vector of **owned** elements.
	#[inline(always)]
	pub fn owned (&self) -> &[T] {
		self.vec.as_slice()
	}

	/// Obtain a slice of the Vector of **borrowed** elements.
	#[inline(always)]
	pub fn borrowed (&self) -> &[&'this T] {
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
		self.borrowed()
	}
}
