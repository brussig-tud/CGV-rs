use std::{fmt, mem::ManuallyDrop, num::NonZero, ops::{Deref, DerefMut}, ptr::NonNull};


/// Wraps any kind of non-null pointer and stores additional data in the address's trailing zero bits.
///
/// To allow the desired number of tag bits, the pointer must be aligned to at least 2^`NBITS` bytes.
#[repr(transparent)]
pub struct Tagged<P: Pointer, const NBITS: u8> (
	/// The wrapped pointer, with the address's lower `NBITS` bits replaced by an arbitrary tag. Since the tag may be
	/// zero, the address must always be ≥ 2^`NBITS`.
	NonNull<P::Pointee>
);
impl<P: Pointer, const NBITS: u8> Tagged<P, NBITS>
{
	/// The number of available tag bits.
	pub const NBITS: u8 = NBITS;

	/// Bitmask to extract the tag from a pointer. The bitwise inverse recovers the original address.
	const TAG_MASK: usize = (1 << NBITS) - 1;

	/// Wrap and tag a safely dereferenceable pointer. **Panics** if the pointee type's alignment is too low for `NBITS`
	/// tag bits, even if the given instance happens to be sufficiently aligned. Prefer this constructor wherever
	/// possible. For restrictions on tag values see [`setTag`](Self::setTag).
	pub fn fromSafe (ptr: P, tag: usize) -> Self where P: SafePointer
	{
		Self::assertTypeAlign(align_of_val(&*ptr));
		// SAFETY: SafePointer requires that pointers be dereferenceable, which in turn requires proper alignment.
		unsafe{Self::newUnchecked(ptr, tag)}
	}
	/// Wrap and tag a pointer to a type whose alignment is known at compiletime. **Panics** if the pointer's or pointee
	/// type's alignment is too low for `NBITS` tag bits. Prefer this constructor if [`fromSafe`](Self::fromSafe) cannot
	/// be used. For restrictions on tag values see [`setTag`](Self::setTag).
	pub fn new (ptr: P, tag: usize) -> Self where P::Pointee: Aligned
	{
		Self::assertTypeAlign(P::Pointee::ALIGN);
		// Raw pointers needn't be properly aligned, so we must check that this one is.
		Self::tryNew(ptr, tag).unwrap()
	}
	/// **Panics** if the given alignment is less than the minimum pointee alignment for `NBITS` tag bits.
	fn assertTypeAlign (align: usize)
	{
		assert!(
			align >= 1 << NBITS,
			"Attempted to create a tagged pointer to type {}, which is aligned to {align} bytes. \
			 However, {NBITS} tag bits require alignment of at least {}.",
			std::any::type_name::<P::Pointee>(),
			1 << NBITS,
		);
	}
	/// Wrap and tag a pointer. Returns [`Err`] if the pointer's alignment is too low for [`Self::NBITS`] tag bits.
	/// For insufficiently aligned pointee types, this function may succeed sometimes but fail elsewhen, depending on
	/// where the pointee happens to be located. To avoid unexpected failures, prefer [`Self::fromSafe`] and
	/// [`Self::new`] wherever possible. Use this function only if the pointee type's alignment cannot be checked even
	/// at runtime or you know that the pointee has stricter alignment than its type requires. For restrictions on tag
	/// values see [`Self::setTag`].
	pub fn tryNew (ptr: P, tag: usize) -> Result<Self, AlignmentError>
	{
		if ptr.raw().addr().get() & Self::TAG_MASK != 0 {return Err(AlignmentError)}
		Ok(unsafe{Self::newUnchecked(ptr, tag)})
	}
	/// Wrap and tag a pointer without checking that it is sufficiently aligned for `NBITS` tag bits. For restrictions
	/// on tag values see [`Self::setTag`].  
	/// **Safety**: The pointer's address must have at least `NBITS` trailing zeros.
	pub unsafe fn newUnchecked (ptr: P, tag: usize) -> Self
	{
		let mut slf = Self(ptr.intoRaw());
		slf.setTag(tag);
		slf
	}

	/// Copy the wrapped pointer but replace the tag bits.
	fn retag (&self, tag: usize) -> NonNull<P::Pointee>
	{
		self.0.map_addr(|addr| unsafe{
			// SAFETY: The pointer's invariant guarantees that `addr & !TAG_MASK ≠ 0`. This invariant is preserved.
			NonZero::new_unchecked(addr.get() & !Self::TAG_MASK | tag & Self::TAG_MASK)
		})
	}

	/// Extract the tag stored in this pointer. The result is guaranteed to be less than 2^`NBITS`.
	pub fn tag (&self) -> usize
	{
		self.0.addr().get() & Self::TAG_MASK
	}
	/// Replace the stored tag without changing the pointer's address or provenance. The tag is trunctated to the lower
	/// `NBITS` bits; if this changes its value, the function may **panic**.
	pub fn setTag (&mut self, tag: usize)
	{
		debug_assert!(tag & Self::TAG_MASK == tag);
		self.0 = self.retag(tag);
	}

	/// Return the wrapped pointer without tag. The tagged pointer's ownership of the pointee is unchanged, so it
	/// mustn't be freed or otherwise invalidated. You must also obey any restrictions imposed by the wrapped type `P`;
	/// in particular you may only mutate the pointee if you could do so through `P`.
	pub fn raw (&self) -> NonNull<P::Pointee>
	{
		self.retag(0)
	}
	/// Consume the tagged pointer, transferring any ownership it has of the pointee to the result. Also returns the
	/// [`tag`](Self::tag).
	pub fn untag (self) -> (P, usize)
	{
		let tag = self.tag();
		// SAFETY: The pointer was obtained by consuming a P and has not freed, at least not by Self.
		(unsafe{P::fromRaw(ManuallyDrop::new(self).raw())}, tag)
	}
}
impl<P: Pointer, const NBITS: u8> std::ops::Drop for Tagged<P, NBITS>
{
	/// Drop the wrapped pointer using `P::drop`.
	fn drop (&mut self)
	{
		// SAFETY: See Self::untag.
		unsafe{P::fromRaw(Self::raw(self))};
	}
}
impl<P: SafePointer, const NBITS: u8> Deref for Tagged<P, NBITS>
{
	type Target = <P as Deref>::Target;

	fn deref (&self) -> &Self::Target
	{
		// SAFETY: Guaranteed by SafePointer.
		unsafe{self.raw().as_ref()}
	}
}
impl<P: SafePointer + DerefMut, const NBITS: u8> DerefMut for Tagged<P, NBITS>
{
	fn deref_mut (&mut self) -> &mut Self::Target
	{
		// SAFETY: Guaranteed by SafePointer.
		unsafe{Self::raw(self).as_mut()}
	}
}
unsafe impl<P: Pointer + Send, const NBITS: u8> Send for Tagged<P, NBITS> {}
unsafe impl<P: Pointer + Sync, const NBITS: u8> Sync for Tagged<P, NBITS> {}

/// Indicates that an address has insufficient alignment for the requested number of tag bits.
#[derive(Clone, Copy, Debug)]
pub struct AlignmentError;
impl fmt::Display for AlignmentError
{
	fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		f.write_str("Attempted to create a tagged pointer with an address whose alignment is too low for the requested \
			number of tag bits")
	}
}
impl std::error::Error for AlignmentError {}


/// Gives a type's alignment, provided that it is known at compiletime.
pub unsafe trait Aligned
{
	/// The implementor's alignment in bytes.  
	/// **Safety**: Must match the alignment determined by the compiler.
	const ALIGN: usize;
}
unsafe impl<T> Aligned for T
{
	const ALIGN: usize = align_of::<T>();
}
unsafe impl<T> Aligned for [T]
{
	const ALIGN: usize = align_of::<T>();
}


/// Abstracts over any kind of non-null pointer, including in particular smart pointers and references.
///
/// Implementors can be thought of as wrappers around a raw [`NonNull`] with additional ownership semantics.
pub unsafe trait Pointer: Sized
{
	/// The type the implementor points to.
	type Pointee: ?Sized;

	/// Copy-convert to a raw pointer without changing ownership of the pointee.
	fn raw (&self) -> NonNull<Self::Pointee>;
	/// Consume and convert to a raw pointer, transferring any ownership of the pointee. Returns the same value as
	/// [`raw`](Self::raw).
	fn intoRaw (self) -> NonNull<Self::Pointee>
	{
		ManuallyDrop::new(self).raw()
	}
	/// Recreate from a raw pointer, transferring any ownership of the pointee.  
	/// **Safety**: The pointer must have been obtained from [`Self::intoRaw`] and must not have been passed to this
	/// function already.
	unsafe fn fromRaw (ptr: NonNull<Self::Pointee>) -> Self;
}

/// Guarantees that pointers of a given type are always safe to dereference.
///
/// **Safety**: Dereferencing pointers returned by [`Pointer::raw`] and [`Pointer::intoRaw`] must return a reference to
/// the same value as calling [`Deref::deref`] and, if provided, [`DerefMut::deref_mut`] on `self`.
pub unsafe trait SafePointer: Pointer + Deref<Target = <Self as Pointer>::Pointee>
{
	/// Convenience function to convert this pointer into a [`Tagged`] one. See [`Tagged::fromSafe`] for details.
	fn tag<const NBITS: u8> (self, tag: usize) -> Tagged<Self, NBITS>
	{
		Tagged::fromSafe(self, tag)
	}
}

unsafe impl<T: ?Sized> Pointer for NonNull<T>
{
	type Pointee = T;

	fn raw (&self) -> Self {*self}
	fn intoRaw (self) -> Self {self}
	unsafe fn fromRaw (src: Self) -> Self {src}
}

unsafe impl<T: ?Sized> Pointer for &T
{
	type Pointee = T;

	fn raw (&self) -> NonNull<T> {(*self).into()}
	fn intoRaw (self) -> NonNull<T> {(*self).into()}
	unsafe fn fromRaw (raw: NonNull<T>) -> Self {unsafe{raw.as_ref()}}
}
unsafe impl<T: ?Sized> SafePointer for &T {}

unsafe impl<T: ?Sized> Pointer for &mut T
{
	type Pointee = T;

	fn raw (&self) -> NonNull<T> {(&**self).into()}
	fn intoRaw (self) -> NonNull<T> {(*self).into()}
	unsafe fn fromRaw (mut raw: NonNull<T>) -> Self {unsafe{raw.as_mut()}}
}
unsafe impl<T: ?Sized> SafePointer for &mut T {}

unsafe impl<T: ?Sized> Pointer for Box<T>
{
	type Pointee = T;

	fn raw (&self) -> NonNull<T>
	{
		NonNull::from(&**self)
	}
	fn intoRaw (self) -> NonNull<T>
	{
		// TODO: Replace with Box::into_non_null once it is stable.
		unsafe{NonNull::new_unchecked(Box::into_raw(self))}
	}
	unsafe fn fromRaw (ptr: NonNull<T>) -> Self
	{
		// TODO: Replace with Box::from_non_null once it is stable.
		unsafe{Self::from_raw(ptr.as_ptr())}
	}
}
unsafe impl<T: ?Sized> SafePointer for Box<T> {}

unsafe impl<T: ?Sized> Pointer for std::rc::Rc<T>
{
	type Pointee = T;

	fn raw (&self) -> NonNull<T>
	{
		unsafe{NonNull::new_unchecked(Self::as_ptr(self).cast_mut())}
	}
	fn intoRaw (self) -> NonNull<T>
	{
		unsafe{NonNull::new_unchecked(Self::into_raw(self).cast_mut())}
	}
	unsafe fn fromRaw (ptr: NonNull<T>) -> Self
	{
		unsafe{Self::from_raw(ptr.as_ptr())}
	}
}
unsafe impl<T: ?Sized> SafePointer for std::rc::Rc<T> {}

unsafe impl<T: ?Sized> Pointer for std::sync::Arc<T>
{
	type Pointee = T;

	fn raw (&self) -> NonNull<T>
	{
		unsafe{NonNull::new_unchecked(Self::as_ptr(self).cast_mut())}
	}
	fn intoRaw (self) -> NonNull<T>
	{
		unsafe{NonNull::new_unchecked(Self::into_raw(self).cast_mut())}
	}
	unsafe fn fromRaw (ptr: NonNull<T>) -> Self
	{
		unsafe{Self::from_raw(ptr.as_ptr())}
	}
}
unsafe impl<T: ?Sized> SafePointer for std::sync::Arc<T> {}

