
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
pub trait UniqueArrayElement<K: PartialOrd+Clone> {
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

/// A container storing elements sequentially (in the order that they are pushed to it) inside a contiguous region of
/// memory, efficiently guaranteeing element uniqueness at the cost of requiring more memory (in the worst case double
/// that of a [`Vec`]).
///
/// **NOTE**: `UniqueArray` does not implement [`IndexMut`] as mutating an element in place leaves the container with no
/// way of vetting the changes, and thus it could not uphold the uniqueness guarantee. If you need to change a value in
/// the array, use [`UniqueArray::changeElement`] instead.
#[derive(Clone)]
struct UniqueArray<K: Ord+Clone, E: UniqueArrayElement<K>> {
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

	/// Return a range that spans the indices of all elements in the `UniqueArray`.
	#[inline(always)]
	pub fn len (&self) -> usize {
		self.elems.len()
	}

	/// Push a new element onto the end of the `UniqueArray`, if it does not already contain an equivalent element.
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
