
//////
//
// Imports
//

// Standard library
use std::{ops::Deref, ops::DerefMut, borrow::Borrow, borrow::BorrowMut};



//////
//
// Functions
//

/// Creates an (invalid if derefenced) reference to an object of the specified type.
///
/// # Returns
///
/// A `'static` reference to some object of the specified type. Must not be dereferenced.
#[inline(always)]
pub const unsafe fn defaultRef<T> () -> &'static T {
	&*(1usize as *const T)
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
	std::ptr::copy_nonoverlapping(source as *const T, target as *mut T, 1);
}



//////
//
// Classes
//

////
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
}


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
	/// # Returns
	///
	/// A reference to the object with `'static` lifetime.
	#[inline(always)]
	pub unsafe fn as_ref (&self) -> &'static T {
		&*self.ptr
	}

	/// Mutable access to the reference.
	///
	/// # Returns
	///
	/// A mutable reference to the object with `'static` lifetime.
	#[inline(always)]
	pub unsafe fn as_mut (&mut self) -> &'static mut T {
		&mut *self.ptr
	}
}
