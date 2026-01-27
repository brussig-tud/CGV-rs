
//! This module provides the `UniqueVec`, a `Vec`-like collection that ensures uniqueness of its elements.
//!
//! # Overview
//!
//! The core of the module is the [`UniqueVec`] struct, but it also defines several auxiliary traits and structs are
//! are required for its functionality. In order to use custom data with `UniqueVec`s, clients must implement the
//! [`UniqueVecElement`] trait which defines what an element's key is and how it is computed.
//!
//! `UniqueVec` is designed to be a drop-in replacement for [`Vec`] in many cases, providing similar runtime
//! characteristics and a familiar API, while automatically managing uniqueness in a performant way.
//!
//! `UniqueVec` uses a set data structure internally to track uniqueness, over which it is generic so any kind of set
//! implementation for which the [`UniqueSet`] trait is implemented can be used. The module provides the convenience
//! type aliases [`BTreeUniqueVec`] and [`HashSetUniqueVec`] that use a [`BTreeSet`] and [`HashSet`], respectively.
//!
//! # Examples
//!
//! ```
//! # use cgv_util::ds::BTreeUniqueVec;
//! let mut v = BTreeUniqueVec::new();
//! assert!(v.push(1));
//! assert!(v.push(2));
//! assert!(!v.push(1)); // 1 is already in the collection
//!
//! assert_eq!(v.len(), 2);
//! assert_eq!(v[0], 1);
//! assert_eq!(v[1], 2);
//! ```
//!
//! # Stable References Requirement
//!
//! A major design contract of this crate is that the keys returned by [`UniqueVecElement::key`] must remain stable even
//! if the element is moved in memory. See the documentation of [`UniqueVecElement`] for more details.



//////
//
// Imports
//

// Standard library
use std::{
	collections::{BTreeSet, HashSet}, hash::{Hash, Hasher}, ops::{Index, Deref}, slice::SliceIndex
};

// ordered_float library
use ordered_float::OrderedFloat;

// Serde library
#[cfg(feature="serde")]
use serde;



//////
//
// Traits
//

/// Trait for elements that can be stored in a [`UniqueVec`].
///
/// Each element must be able to provide a key that is used to determine its uniqueness
/// within the collection. The key can be a reference type for efficiency, but minor restrictions apply (see section
/// "Caveats" below).
///
/// # Examples
///
/// Basic implementation for a custom struct:
///
/// ```
/// # use cgv_util::ds::UniqueVecElement;
///
/// struct User {
///     id: u64,
///     username: String,
/// }
///
/// impl UniqueVecElement for User {
///     type Key<'k> = u64;
///
///     fn key(&self) -> Self::Key<'_> {
///         self.id
///     }
/// }
/// ```
///
/// # Caveats
///
/// The [`key`](Self::key) method is allowed to return references to data. However, these references **must remain
/// stable** even when the element itself is moved in memory:
///
/// * **Safe**: References to data on the heap (e.g., a `&str` referencing the contents of a `String` field).
/// * **Logic Error**: References to member fields of the element itself (e.g., an `&[u8]` referencing a `[u8; N]` array
///   inlined as a struct field).
///
/// Enforcing this at runtime is not done for performance reasons. It is a design contract that implementors of this
/// trait must uphold. Failing to do so can and likely will result in dangling pointers that cause segmentation faults
/// at best or hard-to-track logic errors (e.g. [`UniqueVec`] allowing duplicate elements) at worst.
pub trait UniqueVecElement
{
	/// The type of the key used to identify uniqueness for this element type.
	///
	/// The key must implement [`Ord`] and [`Hash`].
	type Key<'k>: Ord + Hash where Self: 'k;

	/// Returns the key for this element.
	///
	/// This key is used by [`UniqueVec`] to determine the uniqueness of elements.
	fn key (&self) -> Self::Key<'_>;
}
impl UniqueVecElement for bool {
	type Key<'k> = Self;

	fn key (&self) -> Self::Key<'_> {
		*self
	}
}
impl UniqueVecElement for i8 {
	type Key<'k> = Self;

	fn key (&self) -> Self::Key<'_> {
		*self
	}
}
impl UniqueVecElement for u8 {
	type Key<'k> = Self;

	fn key (&self) -> Self::Key<'_> {
		*self
	}
}
impl UniqueVecElement for i16 {
	type Key<'k> = Self;

	fn key (&self) -> Self::Key<'_> {
		*self
	}
}
impl UniqueVecElement for u16 {
	type Key<'k> = Self;

	fn key (&self) -> Self::Key<'_> {
		*self
	}
}
impl UniqueVecElement for i32 {
	type Key<'k> = Self;

	fn key (&self) -> Self::Key<'_> {
		*self
	}
}
impl UniqueVecElement for u32 {
	type Key<'k> = Self;

	fn key (&self) -> Self::Key<'_> {
		*self
	}
}
impl UniqueVecElement for i64 {
	type Key<'k> = Self;

	fn key (&self) -> Self::Key<'_> {
		*self
	}
}
impl UniqueVecElement for u64 {
	type Key<'k> = Self;

	fn key (&self) -> Self::Key<'_> {
		*self
	}
}
impl UniqueVecElement for i128 {
	type Key<'k> = Self;

	fn key (&self) -> Self::Key<'_> {
		*self
	}
}
impl UniqueVecElement for u128 {
	type Key<'k> = Self;

	fn key (&self) -> Self::Key<'_> {
		*self
	}
}
impl UniqueVecElement for f32 {
	type Key<'k> = OrderedFloat<Self>;

	fn key (&self) -> Self::Key<'_> {
		(*self).into()
	}
}
impl UniqueVecElement for f64 {
	type Key<'k> = OrderedFloat<Self>;

	fn key (&self) -> Self::Key<'_> {
		(*self).into()
	}
}
impl UniqueVecElement for String {
	type Key<'k> = &'k str;

	fn key (&self) -> Self::Key<'_> {
		self
	}
}
impl UniqueVecElement for &str {
	type Key<'k> = Self where Self: 'k;

	fn key (&self) -> Self::Key<'_> {
		*self
	}
}
impl UniqueVecElement for std::path::PathBuf {
	type Key<'k> = &'k std::path::Path;

	fn key (&self) -> Self::Key<'_> {
		self
	}
}
impl UniqueVecElement for &std::path::Path {
	type Key<'k> = Self where Self: 'k;

	fn key (&self) -> Self::Key<'_> {
		*self
	}
}
impl UniqueVecElement for uuid::Uuid {
	type Key<'k> = Self;

	fn key (&self) -> Self::Key<'_> {
		*self
	}
}


/// Abstraction over a set implementation used by [`UniqueVec`] for uniqueness tracking.
///
/// This trait allows [`UniqueVec`] to use different set implementations (like [`BTreeSet`]
/// or [`HashSet`]) to keep track of element keys.
pub trait UniqueSet<K>: Default {
	/// Insert a key into the set. Should return `true` if the key was not already present, `false` otherwise.
	fn insert (&mut self, key: K) -> bool;

	/// Should return `true` if the set contains the given key.
	fn contains (&self, key: &K) -> bool;

	/// Remove a key from the set. Should return `true` if the key was present, `false` otherwise.
	fn remove (&mut self, key: &K) -> bool;

	/// Return the number of elements in the set.
	fn len (&self) -> usize;
}
/// Implementation of [`UniqueSet`] using a [`BTreeSet`].
impl<K: Ord> UniqueSet<K> for BTreeSet<K> {
	#[inline(always)]
	fn insert (&mut self, key: K) -> bool { self.insert(key) }
	#[inline(always)]
	fn contains (&self, key: &K) -> bool { self.contains(key) }
	#[inline(always)]
	fn remove (&mut self, key: &K) -> bool { self.remove(key) }
	#[inline(always)]
	fn len (&self) -> usize { self.len() }
}
/// Implementation of [`UniqueSet`] using a [`HashSet`].
impl<K: Eq + Hash> UniqueSet<K> for HashSet<K> {
	#[inline(always)]
	fn insert (&mut self, key: K) -> bool { self.insert(key) }
	#[inline(always)]
	fn contains (&self, key: &K) -> bool { self.contains(key) }
	#[inline(always)]
	fn remove (&mut self, key: &K) -> bool { self.remove(key) }
	#[inline(always)]
	fn len (&self) -> usize { self.len() }
}



//////
//
// Structs
//

/// A wrapper around an element's key used for storage in a [`UniqueSet`].
///
/// `KeyWrapper` allows [`UniqueVec`] to store keys with a `'static` lifetime internally,
/// while still using the original key's [`Ord`], [`PartialEq`], and [`Hash`] implementations.
///
/// This is achieved by storing the key on the heap and using function pointers for operations.
pub struct KeyWrapper<T: UniqueVecElement> {
	// We use a pointer to the key data to avoid lifetime issues with T::Key<'static>
	// while still being able to compare keys.
	// SAFETY: This must be a valid T::Key instance.
	keyPtr: *const (),

	// Function pointers to handle Ord without knowing the concrete type T::Key<'static>
	cmpFn: fn (*const (), *const ()) -> std::cmp::Ordering,
	// Function pointers to handle Hash without knowing the concrete type T::Key<'static>
	hashFn: fn (*const (), &mut dyn Hasher),
	// Function pointers to handle Drop without knowing the concrete type T::Key<'static>
	dropFn: fn (*const ()),

	// Key phantom data
	keyPhantom: std::marker::PhantomData<T>,
}
impl<T: UniqueVecElement> KeyWrapper<T>
{
	#[inline]
	fn new (key: T::Key<'_>) -> Self
	{
		// We move the key to the heap and forget about its lifetime.
		let boxed = Box::new(key);
		unsafe {
			Self {
				keyPtr: Box::into_raw(boxed) as *const (),
				cmpFn: |a, b| {
					let a_key = &*(a as *const T::Key<'_>);
					let b_key = &*(b as *const T::Key<'_>);
					a_key.cmp(b_key)
				},
				hashFn: |ptr, mut state| {
					let key = &*(ptr as *const T::Key<'_>);
					key.hash(&mut state);
				},
				dropFn: |ptr| {
					let _ = Box::from_raw(ptr as *mut T::Key<'_>);
				},
				keyPhantom: std::marker::PhantomData,
			}
		}
	}

	#[inline(always)]
	fn shallow (key: &T::Key<'_>) -> std::mem::ManuallyDrop<Self> {
		std::mem::ManuallyDrop::new(Self::new(unsafe {
			// SAFETY: The transmute operation itself is sane as we don't change the type. However, shallow-copying the
			// key instance could result in a double free if the key owns resources. The `ManuallyDrop` wrapper averts
			// this.
			std::mem::transmute_copy(key)
		}))
	}
}
impl<T: UniqueVecElement> Drop for KeyWrapper<T> {
	#[inline(always)]
	fn drop (&mut self) {
		(self.dropFn)(self.keyPtr);
	}
}
impl<T: UniqueVecElement> PartialEq for KeyWrapper<T> {
	#[inline(always)]
	fn eq (&self, other: &Self) -> bool {
		self.cmp(other) == std::cmp::Ordering::Equal
	}
}
impl<T: UniqueVecElement> Eq for KeyWrapper<T> {}
impl<T: UniqueVecElement> PartialOrd for KeyWrapper<T> {
	#[inline(always)]
	fn partial_cmp (&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
impl<T: UniqueVecElement> Ord for KeyWrapper<T> {
	#[inline(always)]
	fn cmp (&self, other: &Self) -> std::cmp::Ordering {
		(self.cmpFn)(self.keyPtr, other.keyPtr)
	}
}
impl<T: UniqueVecElement> Hash for KeyWrapper<T> {
	#[inline(always)]
	fn hash<H: Hasher> (&self, state: &mut H) {
		(self.hashFn)(self.keyPtr, state);
	}
}


/// A vector that maintains uniqueness of its elements based on a key.
///
/// `UniqueVec` is a collection that behaves similarly to a standard [`Vec`], but it ensures that no two elements in the
/// collection have the same key, as defined by the [`UniqueVecElement`] trait.
///
/// It uses a internal [`Vec`] for storage and a [`UniqueSet`] (like [`BTreeSet`] or [`HashSet`]) to track uniqueness.
///
/// # Type Parameters
///
/// * `T` – The type of elements stored in the vector.
/// * `S` – The type of the set data structure used for uniqueness tracking.
pub struct UniqueVec<T: UniqueVecElement, S: UniqueSet<KeyWrapper<T>>>
{
	// Stores the actual elements of the collection in a `Vec`. This is why `UniqueVec`'s runtime characteristics and
	// public API are so close to the standard `Vec`.
	storage: Vec<T>,

	// Keys of the elements in `storage`.
	// SAFETY: The keys are stored with a 'static lifetime. This is safe as long as:
	// 1. The key refers to data owned by the element on the heap (stable address).
	// 2. The key is removed from this set before the element is removed from `storage`.
	keys: S,
}
impl<T: UniqueVecElement, S: UniqueSet<KeyWrapper<T>>> UniqueVec<T, S>
{
	/// Creates a new, empty `UniqueVec`.
	///
	/// # Examples
	///
	/// ```
	/// # use cgv_util::ds::BTreeUniqueVec;
	/// let v: BTreeUniqueVec<i32> = BTreeUniqueVec::new();
	/// assert_eq!(v.len(), 0);
	/// ```
	#[inline(always)]
	pub fn new () -> Self { Self {
		storage: Vec::new(), keys: S::default(),
	}}

	/// Creates a new, empty `UniqueVec` with the specified minimum capacity. The `UniqueVec` will be able to hold at
	/// least `capacity` elements without reallocating its contiguous storage.
	///
	/// # Arguments
	///
	/// * `capacity` – The desired minimum number of elements that should fit into the allocated contiguous storage.
	///
	/// # Examples
	///
	/// ```
	/// # use cgv_util::ds::*;
	/// let v: BTreeUniqueVec<u32> = UniqueVec::withCapacity(10);
	/// assert!(v.capacity() >= 10);
	/// ```
	#[inline(always)]
	pub fn withCapacity (capacity: usize) -> Self { Self {
		storage: Vec::with_capacity(capacity), keys: S::default(),
	}}

	/// Creates a `UniqueVec` around the given [`Vec`] without checking for uniqueness.
	///
	/// # Safety
	///
	/// The caller must ensure that all elements in the provided vector have unique keys. If there are duplicate keys,
	/// the internal consistency of the `UniqueVec` will be compromised.
	///
	/// # Arguments
	///
	/// * `vec` – The vector that should become the contiguous storage of the `UniqueVec`.
	///
	/// # Examples
	///
	/// ```
	/// # use cgv_util::ds::BTreeUniqueVec;
	/// let v = unsafe { BTreeUniqueVec::fromVec_unchecked(vec![1, 2, 3]) };
	/// assert_eq!(v.len(), 3);
	/// ```
	pub unsafe fn fromVec_unchecked(vec: Vec<T>) -> Self {
		let mut keys = S::default();
		for element in &vec {
			keys.insert(KeyWrapper::new(element.key()));
		}
		debug_assert_eq!(vec.len(), keys.len());
		Self { storage: vec, keys }
	}

	/// Appends an element to the back of the collection if no element with the same [key](UniqueVecElement::key) is
	/// currently present.
	///
	/// # Arguments
	///
	/// * `element` – The new element to be appended.
	///
	/// # Returns
	///
	/// `true` if the element was added (its key was unique), or `false` if it was not added because it would be
	/// considered a duplicate as per its key.
	///
	/// # Examples
	///
	/// ```
	/// # use cgv_util::ds::BTreeUniqueVec;
	/// let mut v = BTreeUniqueVec::new();
	/// assert!(v.push(1));
	/// assert!(!v.push(1));
	/// ```
	pub fn push (&mut self, element: T) -> bool {
		let key = element.key();
		let key_wrapper = KeyWrapper::new(key);

		if self.keys.insert(key_wrapper) {
			self.storage.push(element);
			true
		} else {
			false
		}
	}

	/// Removes the element at the back of the `UniqueVec` and returns it to the caller.
	///
	/// # Returns
	///
	/// `Some(T)` if the collection still had at least one element, `None` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// # use cgv_util::ds::BTreeUniqueVec;
	/// let mut v = BTreeUniqueVec::new();
	/// v.push(1);
	/// assert_eq!(v.pop(), Some(1));
	/// assert_eq!(v.pop(), None);
	/// ```
	pub fn pop (&mut self) -> Option<T> {
		let element = self.storage.pop()?;
		let key = element.key();
		let key_wrapper = KeyWrapper::new(key);
		self.keys.remove(&key_wrapper);
		Some(element)
	}

	/// Removes and returns the element at position `index` within the contiguous storage, shifting all elements after
	/// it to the left.
	///
	/// # Arguments
	///
	/// * `index` – The index inside the contiguous storage of the element to remove.
	///
	/// # Returns
	///
	/// The element that was moved out of the `UniqueVec`.
	///
	/// # Panics
	///
	/// Panics if `index` is out of bounds.
	///
	/// # Examples
	///
	/// ```
	/// # use cgv_util::ds::BTreeUniqueVec;
	/// let mut v = BTreeUniqueVec::new();
	/// v.push(1);
	/// v.push(2);
	/// assert_eq!(v.remove(0), 1);
	/// assert_eq!(v[0], 2);
	/// ```
	pub fn remove (&mut self, index: usize) -> T {
		let element = self.storage.remove(index);
		let key = element.key();
		let key_wrapper = KeyWrapper::new(key);
		self.keys.remove(&key_wrapper);
		element
	}

	/// References an element or subslice depending on the type of index.
	///
	/// # Arguments
	///
	/// * `index` – An index into the contiguous element storage of the `UniqueVec`.
	///
	/// # Returns
	///
	/// `Some` reference to the requested element or subslice, or `None` if `index` is out of bounds.
	///
	/// # Examples
	///
	/// ```
	/// # use cgv_util::ds::BTreeUniqueVec;
	/// let mut v = BTreeUniqueVec::new();
	/// v.push(1);
	/// assert_eq!(v.get(0), Some(&1));
	/// assert_eq!(v.get(1), None);
	/// ```
	#[inline(always)]
	pub fn get<I: SliceIndex<[T]>> (&self, index: I) -> Option<&I::Output> {
		self.storage.get(index)
	}

	/// References the element at the front of the `UniqueVec`.
	///
	/// # Returns
	///
	/// `Some(&T)` referencing the first element if the collection still had at least one, `None` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// # use cgv_util::ds::BTreeUniqueVec;
	/// let mut v = BTreeUniqueVec::new();
	/// v.push(1);
	/// v.push(2);
	/// assert_eq!(v.first(), Some(&1));
	/// ```
	#[inline(always)]
	pub fn first (&self) -> Option<&T> {
		self.storage.first()
	}

	/// References the element at the back of the `UniqueVec`.
	///
	/// # Returns
	///
	/// `Some(&T)` referencing the last element if the collection still had at least one, `None` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// # use cgv_util::ds::BTreeUniqueVec;
	/// let mut v = BTreeUniqueVec::new();
	/// v.push(1);
	/// v.push(2);
	/// assert_eq!(v.last(), Some(&2));
	/// ```
	#[inline(always)]
	pub fn last (&self) -> Option<&T> {
		self.storage.last()
	}

	/// Checks if the `UniqueVec` already contains an entry with the same key as `element`.
	///
	/// # Arguments
	///
	/// * `element` – Reference to an instance of the [element type](T) to check against.
	///
	/// # Returns
	///
	/// `true` if the collection contains an entry with the same key as `element`, `false` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// # use cgv_util::ds::BTreeUniqueVec;
	/// let v = BTreeUniqueVec::from(vec![0, 1, 3]);
	/// assert!(v.contains(&1));
	/// assert!(!v.contains(&2));
	/// ```
	#[inline(always)]
	pub fn contains (&self, element: &T) -> bool {
		self.keys.contains(&KeyWrapper::new(element.key()))
	}
	
	/// Check if the collection contains an element with the given [key](UniqueVecElement::key).
	///
	/// # Arguments
	///
	/// * `key` – Reference to a key to check against.
	///
	/// # Returns
	///
	/// `true` if the collection contains an entry that corresponds to `key`, `false` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// let v = cgv_util::ds::BTreeUniqueVec::from(vec![1,3]);
	/// assert!(v.containsKey(&1));
	/// assert!(!v.containsKey(&2));
	/// ```
	#[inline(always)]
	pub fn containsKey (&self, key: &T::Key<'_>) -> bool {
		self.keys.contains(&KeyWrapper::shallow(key))
	}

	/// Reference the element with the given [key](UniqueVecElement::Key).
	///
	/// **NOTE**: `UniqueVec` is **not** a map! Only *checking* for a key is fast (so this method returns quickly if the
	/// key is not in the collection). Actually *finding* the entry that corresponds to a key that passed the initial
	/// check requires iteration and is thus $O(n)$ in the worst case.
	///
	/// # Arguments
	///
	/// * `key` – Reference to a key that the element should correspond to.
	///
	/// # Returns
	///
	/// `Some(&T)` referencing the requested element if contained in the collection, `None` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// let v = cgv_util::ds::BTreeUniqueVec::from(vec![1, 3]);
	/// assert_eq!(v.fetch(&1), Some(&1));
	/// assert_eq!(v.fetch(&2), None);
	/// ```
	#[inline]
	pub fn fetch<'key> (&'key self, key: &'key T::Key<'key>) -> Option<&'key T>
	{
		if !self.containsKey(key) {
			None
		}
		else {
			self.storage.iter().find(|elem| elem.key().cmp(key) == std::cmp::Ordering::Equal)
		}
	}

	/// Returns the number of elements in the collection.
	///
	/// # Examples
	///
	/// ```
	/// let mut v = cgv_util::ds::BTreeUniqueVec::new();
	/// v.push(1);
	/// assert_eq!(v.len(), 1);
	/// ```
	#[inline(always)]
	pub fn len (&self) -> usize {
		self.storage.len()
	}

	/// Returns the maximum number of elements the collection can hold without reallocating the contiguous storage.
	///
	/// # Examples
	///
	/// ```
	/// let v = cgv_util::ds::BTreeUniqueVec::<u32>::withCapacity(11);
	/// assert!(v.capacity() >= 11);
	/// ```
	#[inline(always)]
	pub fn capacity (&self) -> usize {
		self.storage.capacity()
	}

	/// Returns `true` if the collection contains no elements, `false` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// use cgv_util::ds::BTreeUniqueVec;
	/// let v: BTreeUniqueVec<i32> = BTreeUniqueVec::new();
	/// assert!(v.isEmpty());
	/// ```
	#[inline(always)]
	pub fn isEmpty (&self) -> bool {
		self.storage.is_empty()
	}

	/// Returns an iterator over the elements of the collection.
	///
	/// # Examples
	///
	/// ```
	/// let mut v = cgv_util::ds::BTreeUniqueVec::new();
	/// v.push(1);
	/// v.push(2);
	/// let mut it = v.iter();
	/// assert_eq!(it.next(), Some(&1));
	/// assert_eq!(it.next(), Some(&2));
	/// assert_eq!(it.next(), None);
	/// ```
	#[inline(always)]
	pub fn iter (&self) -> std::slice::Iter<'_, T> {
		self.storage.iter()
	}

	/// Joins two `UniqueVec`s by cloning elements from `self` and `other` into a new `UniqueVec` instance, preserving
	/// the original insertion order within both operands.
	///
	/// # Arguments
	///
	/// * `other` – The over `UniqueVec` to join with. Only elements with keys not already present in `self` will be
	///             included.
	///
	/// # Returns
	///
	/// The new joined `UniqueVec` instance.
	///
	/// # Examples
	///
	/// ```
	/// use cgv_util::ds::*;
	/// let v1 = BTreeUniqueVec::from(vec![1, 2]);
	/// let v2 = BTreeUniqueVec::from(vec![2, 3]);
	/// let joined = UniqueVec::join(&v1, &v2);
	/// assert_eq!(joined.len(), 3);
	/// ```
	pub fn join<S1: UniqueSet<KeyWrapper<T>>> (&self, other: &UniqueVec<T, S1>) -> UniqueVec<T, S> where T: Clone {
		let mut result = self.clone();
		for other_elem in other.iter() {
			result.push(other_elem.clone());
		}
		result
	}

	/// Joins two `UniqueVec`s into a new single `UniqueVec` instance, preserving the original insertion order within
	/// both operands. This results in an identical `UniqueVec` to using [`join`], but the storage of `self` and the
	/// elements therein are fully reused, and the contents of `other` are moved to the new instance instead of
	/// producing clones.
	///
	/// # Arguments
	///
	/// * `other` – The over `UniqueVec` to join with. Only elements with keys not already present in `self` will be
	///             included. Elements are moved to the joined `UniqueVec`.
	///
	/// # Returns
	///
	/// The new joined `UniqueVec` instance.
	///
	/// # Examples
	///
	/// ```compile_fail
	/// use cgv_util::ds::*;
	/// let v1 = BTreeUniqueVec::from(vec![1, 2]);
	/// let v2 = BTreeUniqueVec::from(vec![2, 3]);
	///
	/// let joined = UniqueVec::join_move(v1, v2);
	///
	/// assert_eq!(joined.len(), 3);
	/// assert_eq!(v1.len(), 2); // <- COMPILE ERROR:
	/// assert_eq!(v2[1], 3);    // <- v1 and v2 were moved
	/// ```
	pub fn join_move<S1: UniqueSet<KeyWrapper<T>>> (mut self, other: UniqueVec<T, S1>) -> UniqueVec<T, S> {
		self.extend(other);
		self
	}

	/// Joins two `UniqueVec`s into a new single `UniqueVec` instance, preserving the original insertion order within
	/// both operands. This results in an identical `UniqueVec` to using [`join`], but the storage of `self` and the
	/// elements therein are fully reused, while only the contents of `other` are cloned.
	///
	/// # Arguments
	///
	/// * `other` – The over `UniqueVec` to join with. Only elements with keys not already present in `self` will be
	///             included.
	///
	/// # Returns
	///
	/// The new joined `UniqueVec` instance.
	///
	/// # Examples
	///
	/// ```compile_fail
	/// # use cgv_util::ds::*;
	/// let v1 = BTreeUniqueVec::from(vec![1, 2]);
	/// let v2 = BTreeUniqueVec::from(vec![2, 3]);
	///
	/// let joined = UniqueVec::join_moveLhs(v1, &v2);
	///
	/// assert_eq!(joined.len(), 3);
	/// assert_eq!(v2.len(), 2); // <- OK, v2 was not moved
	/// assert_eq!(v1[0], 1);    // <- COMPILE ERROR: v1 was moved!
	/// ```
	pub fn join_moveLhs<S1: UniqueSet<KeyWrapper<T>>> (mut self, other: &UniqueVec<T, S1>) -> UniqueVec<T, S>
	where T: Clone {
		self.extend(other.iter().cloned());
		self
	}

	/// Checks if the number of elements in storage matches the number of keys in the uniqueness set.
	///
	/// This is a lightweight consistency check.
	///
	/// # Returns
	///
	/// `true` if the check found no inconsistency, `false` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// # use cgv_util::ds::BTreeUniqueVec;
	/// let v = BTreeUniqueVec::from(vec![1, 2, 3]);
	/// assert!(v.checkLenConsistency());
	/// ```
	#[inline(always)]
	pub fn checkLenConsistency (&self) -> bool {
		self.storage.len() == self.keys.len()
	}

	/// Performs a thorough consistency check of the internal data structures.
	///
	/// This check ensures that every element in storage has a corresponding key in the
	/// uniqueness set, and (vice versa) that no keys other than those from the stored elements are present
	/// in the uniqueness set.
	///
	/// # Returns
	///
	/// `true` if the `UniqueVec` is perfectly consistent, `false` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// # use cgv_util::ds::BTreeUniqueVec;
	/// let v = BTreeUniqueVec::from(vec![1, 2, 3]);
	/// assert!(v.checkConsistency());
	/// ```
	#[inline]
	pub fn checkConsistency (&self) -> bool
	{
		// 1: Check if every element has its corresponding key stored
		for element in &self.storage {
			if !self.keys.contains(&KeyWrapper::new(element.key())) {
				return false;
			};
		}

		// 2: Check if no keys other than those from the stored elements are present in the uniqueness set
		self.checkLenConsistency()
	}
}
impl<T: UniqueVecElement, S: UniqueSet<KeyWrapper<T>>> From<Vec<T>> for UniqueVec<T, S>
{
	/// Creates a `UniqueVec` from a [`Vec`].
	///
	/// Duplicate elements in the vector will be ignored. Only the first occurrence of each key will be kept.
	fn from (vec: Vec<T>) -> Self {
		let mut result = Self::new();
		result.extend(vec);
		result
	}
}
impl<T: UniqueVecElement+Copy, S: UniqueSet<KeyWrapper<T>>> From<&[T]> for UniqueVec<T, S>
{
	/// Creates a `UniqueVec` from a slice of `T`.
	///
	/// Duplicate elements in the slice will be ignored. Only the first occurrence of each key will be kept.
	fn from (slice: &[T]) -> Self {
		let mut result = Self::new();
		result.extend(slice.iter().copied());
		result
	}
}
impl<T: UniqueVecElement, S: UniqueSet<KeyWrapper<T>>> FromIterator<T> for UniqueVec<T, S>
{
	/// Creates a `UniqueVec` from the given iterable.
	///
	/// Duplicate elements in the vector will be ignored. Only the first occurrence of each key will be kept.
	fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
		let mut result = Self::new();
		result.extend(iter);
		result
	}
}
impl<T: UniqueVecElement, S: UniqueSet<KeyWrapper<T>>, I: SliceIndex<[T]>> Index<I> for UniqueVec<T, S>
{
	type Output = I::Output;

	/// Returns a reference to an element or subslice depending on the type of index.
	///
	/// # Panics
	///
	/// Panics if the index is out of bounds.
	#[inline(always)]
	fn index (&self, index: I) -> &Self::Output {
		&self.storage[index]
	}
}
impl<T: UniqueVecElement, S: UniqueSet<KeyWrapper<T>>> Deref for UniqueVec<T, S>
{
	type Target = [T];

	/// Dereferences the collection to a slice of its elements.
	#[inline(always)]
	fn deref (&self) -> &Self::Target {
		&self.storage
	}
}
impl<T: UniqueVecElement, S: UniqueSet<KeyWrapper<T>>> IntoIterator for UniqueVec<T, S>
{
	type Item = T;
	type IntoIter = std::vec::IntoIter<T>;

	/// Returns an iterator that moves elements out of the collection.
	#[inline(always)]
	fn into_iter (self) -> Self::IntoIter {
		self.storage.into_iter()
	}
}
impl<T: UniqueVecElement, S: UniqueSet<KeyWrapper<T>>> Extend<T> for UniqueVec<T, S> {
	/// Extends the collection with elements from an iterator.
	///
	/// Only elements with keys not already present in the collection will be added.
	#[inline(always)]
	fn extend<I: IntoIterator<Item=T>> (&mut self, iter: I) {
		for item in iter {
			self.push(item);
		}
	}
}
impl<T: UniqueVecElement, S: UniqueSet<KeyWrapper<T>>> Default for UniqueVec<T, S> {
	#[inline(always)]
	fn default () -> Self {
		Self::new()
	}
}
impl<T: UniqueVecElement+Clone, S: UniqueSet<KeyWrapper<T>>> Clone for UniqueVec<T, S>
{
	#[inline(always)]
	fn clone (&self) -> Self
	{
		let storage = self.storage.clone();
		let mut keys = S::default();
		for element in &storage {
			keys.insert(KeyWrapper::new(element.key()));
		}
		Self { storage, keys }
	}
}
#[cfg(feature="serde")]
impl<T, S> serde::Serialize for UniqueVec<T, S>
	where T: UniqueVecElement+serde::Serialize, S: UniqueSet<KeyWrapper<T>>
{
	fn serialize<Ser: serde::Serializer>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error> {
		use serde::ser::SerializeSeq;
		let mut seq = serializer.serialize_seq(Some(self.storage.len()))?;
		for element in &self.storage {
			seq.serialize_element(&element)?;
		}
		seq.end()
	}
}
#[cfg(feature="serde")]
impl<'de, T, S> serde::Deserialize<'de> for UniqueVec<T, S>
	where T: UniqueVecElement+serde::Deserialize<'de>, S: UniqueSet<KeyWrapper<T>>
{
	/// Deserializes from the given `Deserializer`. Receiving duplicate elements will result in a deserialization error.
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>
	{
		// Deserialize the elements
		let storage: Vec<T> = deserializer.deserialize_seq(VecVisitor::new())?;

		// Rebuild key set
		let mut keys = S::default();
		for element in &storage {
			if !keys.insert(KeyWrapper::new(element.key())) {
				return Err(serde::de::Error::custom("duplicate element"));
			}
		}

		// Done!
		Ok(Self { storage, keys })
	}
}

#[cfg(feature="serde")]
struct VecVisitor<E> {
	marker: std::marker::PhantomData<fn() -> Vec<E>>
}
#[cfg(feature="serde")]
impl<E> VecVisitor<E> {
	fn new() -> Self { VecVisitor {
		marker: std::marker::PhantomData
	}}
}
#[cfg(feature="serde")]
impl<'de, E: serde::Deserialize<'de>> serde::de::Visitor<'de> for VecVisitor<E>
{
	type Value = Vec<E>;

	fn expecting (&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		formatter.write_str("a sequence of unique elements")
	}

	fn visit_seq<M: serde::de::SeqAccess<'de>> (self, mut access: M) -> Result<Self::Value, M::Error> {
		let mut elems = Vec::with_capacity(access.size_hint().unwrap_or(2));
		while let Some(elem) = access.next_element()? {
			elems.push(elem);
		}
		Ok(elems)
	}
}


/// A [`UniqueVec`] that uses a [`BTreeSet`] for uniqueness tracking.
pub type BTreeUniqueVec<T> = UniqueVec<T, BTreeSet<KeyWrapper<T>>>;

/// A [`UniqueVec`] that uses a [`HashSet`] for uniqueness tracking.
pub type HashUniqueVec<T> = UniqueVec<T, HashSet<KeyWrapper<T>>>;
