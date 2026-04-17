
//////
//
// Imports
//

// Standard library
use std::marker::PhantomData;
/* for commented-out `Phony`: */ //use std::{ops::Deref, ops::DerefMut, borrow::Borrow, borrow::BorrowMut};



//////
//
// Functions
//

/// Extend the lifetime of a reference as required by the caller.
///
/// The `Object: 'out` bound prevents extending beyond lifetimes that appear inside the type `Object` itself (e.g. in
/// `Foo<'a>`, it ensures `'out ≤ 'a`). However, **it does not prevent extending beyond the lifetime of the allocation
/// of `Foo` itself**. Notably, for owned types with no lifetime parameters (e.g. `String`, `i32`), the bound is
/// vacuously satisfied for any `'out`, including `'static`.
///
/// # Safety
///
/// The caller must ensure that the referenced allocation remains valid (not dropped or deallocated) for the entirety of
/// `'out`. The `where` bound is a necessary but insufficient condition for soundness – the caller must additionally
/// uphold that the *memory* being pointed to lives at least as long as the returned reference.
///
/// # Arguments
///
/// * `object` – A reference to some object.
///
/// # Returns
///
/// A reference to the same data as `object`, with a lifetime as indicated by the receiver/call site.
#[inline(always)]
pub unsafe fn extendLifetime<'out, Object> (object: &Object) -> &'out Object
	where Object: 'out + ?Sized
{
	unsafe {
		&*(object as *const Object)
	}
}

/// Extend the lifetime of a mutable reference as required by the caller.
///
/// The `Object: 'out` bound prevents extending beyond lifetimes that appear inside the type `Object` itself (e.g. in
/// `Foo<'a>`, it ensures `'out ≤ 'a`). However, **it does not prevent extending beyond the lifetime of the allocation
/// of `Foo` itself**. Notably, for owned types with no lifetime parameters (e.g. `String`, `i32`), the bound is
/// vacuously satisfied for any `'out`, including `'static`.
///
/// # Safety
///
/// The caller must ensure that the referenced allocation remains valid (not dropped or deallocated) for the entirety of
/// `'out`. The `where` bound is a necessary but insufficient condition for soundness – the caller must additionally
/// uphold that the *memory* being pointed to lives at least as long as the returned reference. Furthermore, the caller
/// must ensure that no other references (shared or mutable) to the same data exist for the duration of `'out`.
///
/// # Arguments
///
/// * `object` – A mutable reference to some object.
///
/// # Returns
///
/// A mutable reference to the same data as `object`, with a lifetime as indicated by the receiver/call site.
#[inline(always)]
pub unsafe fn extendLifetime_mut<'out, Object> (object: &mut Object) -> &'out mut Object
	where Object: 'out + ?Sized
{
	unsafe {
		&mut *(object as *mut Object)
	}
}

/// Creates an (invalid if derefenced) reference to an object of the specified type.
///
/// # Returns
///
/// A `'static` reference to some object of the specified type. Must not be dereferenced.
#[inline(always)]
pub const unsafe fn defaultRef<T> () -> &'static T {
	unsafe { &*(1usize as *const T) }
}

/// Perform a shallow memory copy for copy-assigning from one value of type `T` to another. Can be used for copying
/// objects that are not `Copy`.
///
/// # Arguments
///
/// * `target` – A reference to the target object that shall receive the copied memory contents.
/// * `source` – A reference to the source object that holds the to-be-copied memory contents.
#[inline(always)]
pub unsafe fn copyAssign<T: Sized> (target: &mut T, source: &T) {
	unsafe { std::ptr::copy_nonoverlapping(source as *const T, target as *mut T, 1); }
}

/// Construct a version of the given `str` slice "slid" to another memory location at `offset` bytes from its original
/// place.
///
/// # Safety
///
/// The caller is responsible for ensuring that the memory at the new location contains valid UTF-8.
///
/// # Arguments
///
/// * `source` – The source `str` to slide.
/// * `offset` – The delta (in bytes) to slide the `str` by.
///
/// # Returns
///
/// A new slice of the same length as `source` offset to the new location.
#[inline(always)]
pub unsafe fn offsetStr (source: &str, offset: isize) -> &str {
	unsafe { str::from_utf8_unchecked(std::slice::from_raw_parts(source.as_ptr().offset(offset), source.len())) }
}



//////
//
// Structs
//

////
// StridedIter

/// An efficient iterator that reads values of type `T` at a fixed byte stride from a contiguous buffer. This enables
/// easy iteration over individual attributes in interleaved ("array of structs") data layouts without copying.
///
/// # Safety
///
/// Users must ensure that:
/// * The initial `ptr` points to a valid, aligned `T` within a live allocation.
/// * Every address `ptr + i*stride` for `i` in `0..remaining` also points to a valid, aligned `T` within the same
///   allocation.
pub struct StridedIter<T: Copy> {
	ptr: *const u8,
	stride: usize,
	remaining: usize,
	_phantom: PhantomData<T>,
}
impl<T: Copy> StridedIter<T>
{
	/// Create a new strided iterator.
	///
	/// # Arguments
	///
	/// * `ptr` – Pointer to the first `T` in the strided sequence (i.e. the `T` field in the first record).
	/// * `stride` – The stride between subsequent records.
	/// * `len` – The number of records after and including the one pointed to by `ptr` that can be iterated over.
	///
	/// # Safety
	///
	/// See [struct-level](StridedIter) safety documentation.
	#[inline(always)]
	pub unsafe fn new (ptr: *const T, stride: usize, len: usize) -> Self { Self {
		ptr: ptr as *const u8, stride, remaining: len, _phantom: PhantomData
	}}
}
impl<T: Copy> Iterator for StridedIter<T> {
	type Item = T;

	fn next (&mut self) -> Option<T>
	{
		if self.remaining == 0 {
			return None;
		}
		// SAFETY: guaranteed by caller (see struct-level docs)
		let value = unsafe { *(self.ptr as *const T) };
		self.ptr = unsafe { self.ptr.add(self.stride) };
		self.remaining -= 1;
		Some(value)
	}

	fn size_hint (&self) -> (usize, Option<usize>) {
		(self.remaining, Some(self.remaining))
	}
}
impl<T: Copy> ExactSizeIterator for StridedIter<T> {}

/// Helper to construct a [`StridedIter`] over the same field in a series of structured data records (aka. interleaved
/// data).
///
/// # Safety
///
/// This macro internally uses raw pointer manipulation, so it can only be used inside `unsafe` blocks. The required
/// invariants are documented in the struct-level documentation of [StridedIter].
///
/// # Arguments
///
/// * `data` – reference to the raw data container (or slice) holding the interleaved data
/// * `field` – field of a data record (e.g. tuple index `0`, `1`, or a named struct field)
/// * `T` – the type of the field
#[macro_export]
macro_rules! stridedIter
{
	($data:expr, $field:tt, $T:ty) => {{
		let base = std::ptr::addr_of!((*$data.as_ptr()).$field);
		cgv_util::notsafe::StridedIter::<$T>::new(
			base, size_of_val(&*$data.as_ptr()), $data.len(),
		)
	}};
}
pub use crate::stridedIter;


////
// StridedRefIter

/// An efficient iterator that references values of type `T` at a fixed byte stride from a contiguous buffer. This
/// enables easy by-reference iteration over individual attributes in interleaved ("array of structs") data layouts
/// without copying.
///
/// # Safety
///
/// Users must ensure that:
/// * The initial `ptr` points to a valid, aligned `T` within a live allocation.
/// * Every address `ptr + i*stride` for `i` in `0..remaining` also points to a valid, aligned `T` within the same
///   allocation.
pub struct StridedRefIter<'outer, T: Sized+'outer> {
	ptr: *const u8,
	stride: usize,
	remaining: usize,
	_phantom: PhantomData<&'outer T>,
}
impl<T: Sized> StridedRefIter<'_, T>
{
	/// Create a new strided referencing iterator.
	///
	/// # Arguments
	///
	/// * `ptr` – Pointer to the first `T` in the strided sequence (i.e. the `T` field in the first record).
	/// * `stride` – The stride between subsequent records.
	/// * `len` – The number of records after and including the one pointed to by `ptr` that can be iterated over.
	///
	/// # Safety
	///
	/// See [struct-level](StridedRefIter) safety documentation.
	#[inline(always)]
	pub unsafe fn new (ptr: *const T, stride: usize, len: usize) -> Self { Self {
		ptr: ptr as *const u8, stride, remaining: len, _phantom: PhantomData
	}}
}
impl<'outer, T: Sized+'outer> Iterator for StridedRefIter<'outer, T> {
	type Item = &'outer T;

	fn next (&mut self) -> Option<&'outer T>
	{
		if self.remaining == 0 {
			return None;
		}
		let value = unsafe {
			// SAFETY: guaranteed by caller (see struct-level docs), plus the reference we take here will be returned to
			// the caller with a lifetime equal to the lifetime of the iterator, which is correct.
			&*(self.ptr as *const T)
		};
		self.ptr = unsafe { self.ptr.add(self.stride) };
		self.remaining -= 1;
		Some(value)
	}

	fn size_hint (&self) -> (usize, Option<usize>) {
		(self.remaining, Some(self.remaining))
	}
}
impl<T: Sized> ExactSizeIterator for StridedRefIter<'_, T> {}

///
#[macro_export]
macro_rules! stridedRefIter
{
	($data:expr, $field:tt, $T:ty) => {{
		let base = std::ptr::addr_of!((*$data.as_ptr()).$field);
		cgv_util::notsafe::StridedRefIter::<$T>::new(
			base, size_of_val(&*$data.as_ptr()), $data.len(),
		)
	}};
}
pub use crate::stridedRefIter;


/*////
// Phony

/// A container for holding a *phony* object of the given type.
#[repr(align(8))]
pub struct Phony<T: Sized> where [(); size_of::<T>()]: {
	mem: [u8; size_of::<T>()]
}
impl<T: Sized> Phony<T> where [(); size_of::<T>()]: {
	/// Allocate memory to hold the phony object.
	pub unsafe fn new() -> Self { Phony {
		mem: [0; size_of::<T>()]
	}}
}
impl<T: Sized> AsRef<T> for Phony<T> where [(); size_of::<T>()]: {
	#[inline(always)]
	fn as_ref (&self) -> &T {
		unsafe { &*(&self.mem as *const u8 as *const T) }
	}
}
impl<T: Sized> AsMut<T> for Phony<T> where [(); size_of::<T>()]: {
	#[inline(always)]
	fn as_mut (&mut self) -> &mut T {
		unsafe { &mut *(&mut self.mem as *mut u8 as *mut T) }
	}
}
impl<T: Sized> Borrow<T> for Phony<T> where [(); size_of::<T>()]: {
	#[inline(always)]
	fn borrow (&self) -> &T {
		self.as_ref()
	}
}
impl<T: Sized> BorrowMut<T> for Phony<T> where [(); size_of::<T>()]: {
	#[inline(always)]
	fn borrow_mut (&mut self) -> &mut T {
		self.as_mut()
	}
}
impl<T: Sized> Deref for Phony<T> where [(); size_of::<T>()]: {
	type Target = T;
	fn deref (&self) -> &Self::Target {
		self.as_ref()
	}
}
impl<T: Sized> DerefMut for Phony<T> where [(); size_of::<T>()]: {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.as_mut()
	}
}*/


////
// UncheckedRef

/// A zero-overhead wrapper storing a completely unchecked reference to an arbitrary object.
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct UncheckedRef<T> {
	ptr: *mut T
}
impl<T> UncheckedRef<T>
{
	/// Create the unchecked reference from the given safe reference.
	///
	/// # Arguments
	///
	/// * `reference` – A safe reference to some object from which to create the unsafe wrapper.
	///
	/// # Returns
	///
	/// An instance unsafely wrapping the provided reference.
	#[inline(always)]
	pub fn new (reference: &T) -> Self {
		Self { ptr: reference as *const T as *mut T }
	}

	/// Creates a `null` reference that should not be dereferenced until assigned something.
	///
	/// # Returns
	///
	/// An instance of an unchecked `null` reference. Obviously, this should not be dereferenced.
	#[inline(always)]
	pub fn null () -> Self {
		Self { ptr: std::ptr::null_mut() }
	}

	/// Re-points the unchecked reference to something else.
	///
	/// # Arguments
	///
	/// * `reference` – A safe reference to some object that the unchecked reference should henceforth point to.
	#[inline(always)]
	pub fn reset (&mut self, reference: &T) {
		self.ptr = reference as *const T as *mut T;
	}

	/// Immutable access to the reference.
	///
	/// # Safety
	///
	/// The caller is responsible for ensuring the `UncheckedRef` points to a valid object. Furthermore, the caller is
	/// responsible for ensuring the lifetime of the receiver does not exceed the lifetime of the referenced object.
	///
	/// # Returns
	///
	/// A reference to the object with the same lifetime as the receiver.
	#[inline(always)]
	pub unsafe fn as_ref<'outer> (&self) -> &'outer T {
		unsafe { &*self.ptr }
	}

	/// Mutable access to the reference.
	///
	/// # Safety
	///
	/// The caller is responsible for ensuring the `UncheckedRef` points to a valid object. Furthermore, the caller is
	/// responsible for ensuring the lifetime of the receiver does not exceed the lifetime of the referenced object.
	///
	/// # Returns
	///
	/// A mutable reference to the object with the same lifetime as the receiver.
	#[inline(always)]
	pub unsafe fn as_mut<'outer> (&mut self) -> &'outer mut T {
		unsafe { &mut *self.ptr }
	}
}
