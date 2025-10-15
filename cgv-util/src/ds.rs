
//////
//
// Imports
//

// Standard library
use std::{ops::{Range, Index}, collections::BTreeSet};

// Ordered-float crate
use ordered_float::OrderedFloat;



//////
//
// Classes
//

////
// MultiIndexContainer

/// A simple container that allows for both efficient random access and removal/insertion at O(1), at the cost of one
/// level of indirection and thus higher per-operation overhead than other containers that can do one or the other well.
struct MultiIndexContainer {}


////
// UniqueArray

/// The trait of things that can be stored in a [`UniqueArray`].
pub trait UniqueArrayElement<K: PartialOrd+Clone>: Clone {
	fn key (&self) -> &K;
}
impl UniqueArrayElement<Self> for bool {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for i8 {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for u8 {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for i16 {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for u16 {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for i32 {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for u32 {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for i64 {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for u64 {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for i128 {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for u128 {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for OrderedFloat<f32> {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for OrderedFloat<f64> {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for &String {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for &str {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for std::path::PathBuf {
	fn key (&self) -> &Self {
		self
	}
}
impl UniqueArrayElement<Self> for &std::path::Path {
	fn key (&self) -> &Self {
		self
	}
}

/// A collection storing elements sequentially (in the order they were inserted) inside a contiguous region of memory,
/// efficiently guaranteeing element uniqueness at the cost of requiring more memory (in the worst case double that of a
/// [`Vec`]). This is a lightweight, less-featured alternative to the collections provided by the popular
/// [index-](https://crates.io/crates/indexmap)/[ordermap](https://crates.io/crates/ordermap) crates. Its most notable
/// trade-off is that it **cannot do constant-time lookup** of elements, falling back to a
/// [linear search](UniqueArray::get) instead.
///
/// **NOTE**: `UniqueArray` does not implement [`IndexMut`] as mutating an element from the outside leaves the container
/// with no way of vetting the changes, and thus it could not uphold the uniqueness guarantee. If you need to change an
/// element in the array, use [`UniqueArray::changeElement`] instead.
#[derive(Clone)]
pub struct UniqueArray<K: Ord+Clone, E: UniqueArrayElement<K>> {
	keys: BTreeSet<K>,
	elems: Vec<E>
}
impl<K: Ord+Clone, E: UniqueArrayElement<K>> UniqueArray<K, E>
{
	/// Create a new, empty `UniqueArray`.
	pub fn new () -> Self { Self {
		keys: BTreeSet::new(), elems: Vec::new()
	}}

	/// Create a new, empty `UniqueArray` with pre-allocated capacity for `n` elements.
	pub fn withCapacity (n: usize) -> Self { Self {
		keys: BTreeSet::new(), elems: Vec::with_capacity(n)
	}}

	/// Return the number of elements currently stored in the `UniqueArray`.
	#[inline(always)]
	pub fn len (&self) -> usize {
		self.elems.len()
	}

	/// Push a new element onto the end of the `UniqueArray`, if it does not already contain an equivalent element that
	/// [keys](UniqueArrayElement::key) to the same value.
	///
	/// # Arguments
	///
	/// * `elem` – The new element to be inserted.
	///
	/// # Returns
	///
	/// `Ok(())`if the element was successfully inserted, `Err(())` if an equivalent was already present.
	///
	/// # Examples
	///
	/// ```rust
	/// let mut uniqueThings = UniqueArray::new();
	///
	/// assert!(collection.push(2).is_ok());
	/// assert!(collection.push(3).is_ok());
	/// assert!(collection.push(2).is_err()); // duplicate
	/// ```
	pub fn push (&mut self, elem: E) -> Result<(), ()>
	{
		if self.keys.insert(elem.key().to_owned()) {
			self.elems.push(elem);
			assert_eq!(self.keys.len(), self.elems.len());
			Ok(())
		}
		else {
			assert_eq!(self.keys.len(), self.elems.len());
			Err(())
		}
	}

	/// Attempts to concatenate the elements of another `UniqueArray` to `self`.
	///
	/// This function checks that all elements from the `other` array are unique with respect to `self`. If any element
	/// in `other` already exists, the operation is aborted leaving `self` **unchanged**, and the function returns an
	/// error.
	///
	/// # Parameters
	///
	/// * `other` – A reference to another `UniqueArray` whose elements are to be joined to `self`.
	///
	/// # Returns
	///
	/// `Ok` if the join was successfull, `Err` if the join failed due to duplicate elements.
	///
	/// # Example
	///
	/// ```rust
	/// let mut array1 = UniqueArray::new();
	/// array1.push(1).unwrap();
	/// array1.push(2).unwrap();
	///
	/// let mut array2 = UniqueArray::new();
	/// array2.push(3).unwrap();
	/// array2.push(4).unwrap();
	///
	/// assert!(array1.tryJoin(&array2).is_ok());
	/// assert_eq!(array1.len(), 4);
	///
	/// assert!(array1.tryJoin(&array2).is_err()); // Operation will now fail due to duplication
	/// ```
	pub fn tryJoin (&mut self, other: &UniqueArray<K, E>) -> Result<(), ()>
	{
		// Pass 1 - check for uniqueness
		for elem in other.elems.iter() {
			if self.contains(elem) {
				return Err(())
			}
		}

		// Pass 2 - push elements
		for elem in other.elems.iter() {
			self.push(elem.clone()).unwrap();
		};
		Ok(())
	}

	/// Unsafely concatenates the elements of another `UniqueArray` to `self`, without checking for uniqueness.
	///
	/// # Parameters
	///
	/// * `other` – A reference to another `UniqueArray` whose elements are to be joined to `self`.
	///
	/// # Panics
	///
	/// If the join produces duplicates. This will cause the internal datastructures to desync which is very cheap to
	/// detect, so it is done even in *Release* builds rather than leaving the `UniqueArray` in an unusable state.
	///
	/// # Example
	///
	/// ```rust
	/// let mut array1 = UniqueArray::new();
	/// array1.push(1).unwrap();
	/// array1.push(2).unwrap();
	///
	/// let mut array2 = UniqueArray::new();
	/// array2.push(3).unwrap();
	/// array2.push(4).unwrap();
	///
	/// unsafe { array1.join_unchecked(&array2) }
	/// assert_eq!(array1.len(), 4);
	///
	/// unsafe { array1.join_unchecked(&array2) } // This will panic!
	/// ```
	pub unsafe fn join_unchecked (&mut self, other: &UniqueArray<K, E>) {
		self.elems.extend_from_slice(other.elems.as_slice());
		self.keys.extend(other.keys.iter().cloned());
		if self.elems.len() != self.keys.len() {
			panic!("UniqueArray::join_unchecked: internal state corruption! Did you try to join duplicate elements?");
		}
	}

	/// Checks if the given element is present in the collection.
	///
	/// # Arguments
	///
	/// * `elem` – A reference to the element to look for in the collection. Makes use of the
	/// [key](UniqueArrayElement::key) provided by the [`UniqueArrayElement`] (which `E` implements) to enable cheap
	/// comparison.
	///
	/// # Returns
	///
	/// * `true` if the given element is found in the collection.
	/// * `false` if the element is not found.
	pub fn contains (&self, elem: &E) -> bool {
		self.keys.contains(elem.key())
	}

	/// Checks if the `UniqueArray` contains an element that is being [keyed](UniqueArrayElement::key) to the given
	/// value.
	///
	/// # Arguments
	///
	/// * `key` – The key to look for in the collection.
	///
	/// # Returns
	///
	/// `true` if an element with the given key exists in the collection, `false` otherwise.
	///
	/// # Example
	///
	/// TODO, TODO, TODO!
	/// ```rust
	/// let mut map = MyMap::new();
	/// map.insert(1, "value1");
	///
	/// assert!(map.containsKey(&1)); // Returns true, as key 1 exists.
	/// assert!(!map.containsKey(&2)); // Returns false, as key 2 does not exist.
	/// ```
	pub fn containsKey (&self, key: &K) -> bool {
		self.keys.contains(key)
	}

	/// Retrieves a reference to an element in the `UniqueArray` that [keys](UniqueArrayElement::key) to the given
	/// value.
	///
	/// Determining whether the element is present in the collection is fast, so this method will return very quickly if
	/// no such element is contained. But **actually finding the element** if it exists **involves a linear search**.
	///
	/// # Arguments
	///
	/// * `key` – The value that the desired element should key to.
	///
	/// # Returns
	///
	/// `Some` element from the collection that keys to the specified value, `None` if no such element exists in the
	/// collection.
	///
	/// # Example
	///
	/// TODO, TODO, TODO!
	/// ```rust
	/// let mut collection = MyCollection::new();
	/// collection.insert("key1", "value1");
	///
	/// if let Some(value) = collection.get(&"key1") {
	///     println!("Found: {}", value);
	/// } else {
	///     println!("Key not found");
	/// }
	/// ```
	pub fn get (&self, key: &K) -> Option<&E> {
		if self.keys.contains(key) {
			self.elems.iter().find(|&e| e.key() == key)
		}
		else {
			None
		}
	}

	/// Replace the element at `index` with the value returned by the closure `modifier`.
	pub fn changeElement (&mut self, index: usize, modifier: impl Fn(&E)->E) -> Result<(), ()>
	{
		let oldKey = self.elems[index].key();
		let newValue = modifier(&self.elems[index]);
		if self.keys.insert(newValue.key().to_owned()) {
			self.keys.remove(oldKey);
			self.elems[index] = newValue;
			assert_eq!(self.keys.len(), self.elems.len());
			Ok(())
		}
		else {
			assert_eq!(self.keys.len(), self.elems.len());
			Err(())
		}
	}

	/// Return a range that spans the indices of all elements in the `UniqueArray`.
	#[inline(always)]
	pub fn indices (&self) -> Range<usize> {
		0..self.elems.len()
	}

	/// Return a reference to the elements in the `UniqueArray`.
	#[inline(always)]
	pub fn elements (&self) -> &Vec<E> {
		&self.elems
	}

	/// Obtain a read-only iterator to the stored elements. This is merely a stop-gap method until [`Iterator`] is
	/// properly implemented.
	#[inline(always)]
	pub fn iter (&'_ self) -> core::slice::Iter<'_, E> {
		self.elems.iter()
	}
}
impl<K: Ord+Clone, E: UniqueArrayElement<K>> Index<usize> for UniqueArray<K, E>
{
	/// The element type of the [`UniqueArray`].
	type Output = E;

	/// Reference the element at the given index.
	fn index (&self, index: usize) -> &Self::Output {
		&self.elems[index]
	}
}
