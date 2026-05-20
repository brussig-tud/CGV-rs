use std::{any::Any, ptr::NonNull};

use crate::tagged_ptr::*;


#[test]
fn fromSafe_ok ()
{
	let b0x = Box::new(0u32) as Box<dyn Any>;
	let raw = NonNull::from(&*b0x);
	let ptr = b0x.tag::<2>(3);
	assert_eq!(ptr.raw(), raw);
	assert_eq!(ptr.tag(), 3);
}
#[test] #[should_panic(expected = "align")]
fn fromSafe_badType ()
{
	(Box::new(0u32) as Box<dyn Any>).tag::<3>(3);
}

#[test]
fn new_ok ()
{
	let raw = NonNull::<u32>::dangling();
	let ptr = Tagged::<_, 2>::new(raw, 3);
	assert_eq!(ptr.raw(), raw);
	assert_eq!(ptr.tag(), 3);
}
#[test] #[should_panic(expected = "align")]
fn new_badType ()
{
	Tagged::<_, 3>::new(NonNull::<u32>::dangling(), 3);
}
#[test] #[should_panic(expected = "AlignmentError")]
fn new_badVal ()
{
	Tagged::<_, 2>::new(NonNull::<u32>::without_provenance(2.try_into().unwrap()), 3);
}

#[test]
fn tryNew_ok ()
{
	let raw = NonNull::<u32>::dangling();
	let ptr = Tagged::<_, 2>::tryNew(raw, 3).unwrap();
	assert_eq!(ptr.raw(), raw);
	assert_eq!(ptr.tag(), 3);
}
#[test]
fn tryNew_badType ()
{
	let raw = NonNull::<u32>::dangling().cast::<u16>();
	let ptr = Tagged::<_, 2>::tryNew(raw, 3).unwrap();
	assert_eq!(ptr.raw(), raw);
	assert_eq!(ptr.tag(), 3);
}
#[test]
fn tryNew_badValue ()
{
	assert!(matches!(
		Tagged::<_, 2>::tryNew(NonNull::<u32>::without_provenance(2.try_into().unwrap()), 3),
		Err(AlignmentError)
	));
}

#[test] #[should_panic]
fn newUnchecked_tagTooBig ()
{
	unsafe{Tagged::<_, 2>::newUnchecked(NonNull::<u32>::dangling(), 4)};
}

#[test]
fn setTag_ok ()
{
	let raw = NonNull::<u32>::dangling();
	let mut ptr = Tagged::<_, 2>::new(raw, 1);
	ptr.setTag(2);
	assert_eq!(ptr.raw(), raw);
	assert_eq!(ptr.tag(), 2);
}
#[test] #[should_panic]
fn setTag_tooBig ()
{
	let mut ptr = Tagged::<_, 2>::new(NonNull::<u32>::dangling(), 3);
	ptr.setTag(4);
}

#[test]
fn intoRaw ()
{
	assert_eq!(Box::new(67u32).tag::<2>(3).untag(), (Box::new(67), 3));
}

#[test]
fn deref ()
{
	let ptr = Box::new(67u32);
	assert_eq!(*ptr, 67);
}
#[test]
fn deref_mut ()
{
	let mut ptr = Box::new(67u32);
	*ptr = 11037;
	assert_eq!(*ptr, 11037);
}

