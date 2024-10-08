
/// Decorates the given reference with a `'static` lifetime.
///
/// # Arguments
///
/// * `reference` - The reference to statify.
#[inline(always)]
pub fn statify<T: ?Sized> (reference: &T) -> &'static T {
	unsafe { &(*(reference as *const T)) }
}

/// Returns a mutable reference to the given object behind the given immutable reference.
///
/// # Arguments
///
/// * `reference` - The reference to mutify.
#[inline(always)]
pub fn mutify<T: ?Sized> (reference: &T) -> &'static mut T
{
	#[allow(invalid_reference_casting)]
	unsafe { &mut *((reference as *const T) as *mut T) }
}
