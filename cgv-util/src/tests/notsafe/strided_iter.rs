
//////
//
// Imports
//

// Local imports
use {crate as cgv_util, crate::notsafe::*};



//////
//
// Structs
//

/// A record with a single primitive field.
struct SinglePrimitiveFieldRecord(u32);

/// A record with a single complex field.
struct SingleComplexFieldRecord(String);

/// A structured record with three mixed fields.
struct ThreeFieldsStructuredRecord<'this> {
	number: u32,
	complex: String,
	reference: &'this str
}
impl<'this> ThreeFieldsStructuredRecord<'this> {
	fn new (number: u32, complex: String, reference: &'this str) -> Self { Self {
		number, complex, reference
	}}
}

/// Using a primitive single-field record.
struct SinglePrimitiveField {
	data: Vec<SinglePrimitiveFieldRecord>
}
impl SinglePrimitiveField
{
	fn empty () -> Self { Self { data: vec![] } }

	fn createTestData () -> Self { Self {
		data: vec![
			SinglePrimitiveFieldRecord(0), SinglePrimitiveFieldRecord(1), SinglePrimitiveFieldRecord(2),
			SinglePrimitiveFieldRecord(3)
		]
	}}

	fn num (&self) -> usize {
		self.data.first().unwrap().0 as usize;
		self.data.len()
	}

	fn iter (&self) -> StridedCopyIter<'_, u32> {
		unsafe {
			// SAFETY: We store a `Vec` of structs, and `Vec` can be trusted to return the correct length, so alignment
			// and validity of the fields the iterator accesses is guaranteed.
			stridedCopyIter!(self.data, 0, u32)
		}
	}
}

/// Using a complex single-field record.
struct SingleComplexField {
	data: Vec<SingleComplexFieldRecord>
}
impl SingleComplexField
{
	fn empty () -> Self { Self {
		data: vec![]
	}}

	fn createTestData () -> Self { Self {
		data: vec![
			SingleComplexFieldRecord("zero".into()), SingleComplexFieldRecord("one".into()),
			SingleComplexFieldRecord("two".into()), SingleComplexFieldRecord("three".into())
		]
	}}

	fn num (&self) -> usize {
		self.data.len()
	}

	fn iter (&self) -> StridedRefIter<'_, String> {
		unsafe {
			// SAFETY: We store a `Vec` of structs, and `Vec` can be trusted to return the correct length, so alignment
			// and validity of the fields the iterator accesses is guaranteed.
			stridedRefIter!(self.data, 0, String)
		}
	}
}

/// Using a tuple record with three mixed fields.
struct ThreeFields<'this> {
	numberStrings: Vec<String>,
	data: Vec<(u32, String, &'this str)>
}
impl ThreeFields<'_>
{
	fn empty () -> Self { Self {
		numberStrings: vec![], data: vec![]
	}}

	fn createTestData () -> Self
	{
		let numberStrings = vec!["ten".to_string(), "eleven".into(), "twelve".into(), "thirteen".into()];
		let numberStringRefs = unsafe {
			// SAFETY: the `numberStrings` vector will not be changed anymore after construction, so the addresses of
			// its elements will remain stable. Plus, we won't return references to them that can outlive `self`.
			// We are essentially creating a self-referential struct to really stress-test the iterators.
			(&*(numberStrings[0].as_str() as *const str), &*(numberStrings[1].as_str() as *const str),
			 &*(numberStrings[2].as_str() as *const str), &*(numberStrings[3].as_str() as *const str))
		};
		Self {
			data: vec![
				(0, "zero".into(), numberStringRefs.0), (1, "one".into(), numberStringRefs.1),
				(2, "two".into(), numberStringRefs.2), (3, "three".into(), numberStringRefs.3)
			],
			numberStrings
		}
	}

	fn numbers (&self) -> StridedCopyIter<'_, u32> {
		unsafe {
			// SAFETY: We store a `Vec` of structs, and `Vec` can be trusted to return the correct length, so alignment
			// and validity of the fields the iterator accesses is guaranteed.
			stridedCopyIter!(self.data, 0, u32)
		}
	}

	fn complexes (&self) -> StridedRefIter<'_, String> {
		unsafe {
			// SAFETY: We store a `Vec` of structs, and `Vec` can be trusted to return the correct length, so alignment
			// and validity of the fields the iterator accesses is guaranteed.
			stridedRefIter!(self.data, 1, String)
		}
	}

	fn references (&self) -> StridedCopyIter<'_, &str> {
		unsafe {
			// SAFETY: We store a `Vec` of structs, and `Vec` can be trusted to return the correct length, so alignment
			// and validity of the fields the iterator accesses is guaranteed.
			stridedCopyIter!(self.data, 2, &str)
		}
	}
}

/// Using a structured record with three mixed fields.
struct ThreeFieldsStructured<'this> {
	numberStrings: Vec<String>,
	data: Vec<ThreeFieldsStructuredRecord<'this>>
}
impl ThreeFieldsStructured<'_>
{
	fn empty () -> Self { Self {
		numberStrings: vec![], data: vec![]
	}}

	fn createTestData () -> Self
	{
		let numberStrings = vec!["ten".to_string(), "eleven".into(), "twelve".into(), "thirteen".into()];
		let numberStringRefs = unsafe {
			// SAFETY: the `numberStrings` vector will not be changed anymore after construction, so the addresses of
			// its elements will remain stable. Plus, we won't return references to them that can outlive `self`.
			// We are essentially creating a self-referential struct to really stress-test the iterators.
			(&*(numberStrings[0].as_str() as *const str), &*(numberStrings[1].as_str() as *const str),
			 &*(numberStrings[2].as_str() as *const str), &*(numberStrings[3].as_str() as *const str))
		};
		Self {
			data: vec![
				ThreeFieldsStructuredRecord::new(0, "zero".into(), numberStringRefs.0),
				ThreeFieldsStructuredRecord::new(1, "one".into(), numberStringRefs.1),
				ThreeFieldsStructuredRecord::new(2, "two".into(), numberStringRefs.2),
				ThreeFieldsStructuredRecord::new(3, "three".into(), numberStringRefs.3)
			],
			numberStrings
		}
	}

	fn numbers (&self) -> StridedCopyIter<'_, u32> {
		unsafe {
			// SAFETY: We store a `Vec` of structs, and `Vec` can be trusted to return the correct length, so alignment
			// and validity of the fields the iterator accesses is guaranteed.
			stridedCopyIter!(self.data, number, u32)
		}
	}

	fn complexes (&self) -> StridedRefIter<'_, String> {
		unsafe {
			// SAFETY: We store a `Vec` of structs, and `Vec` can be trusted to return the correct length, so alignment
			// and validity of the fields the iterator accesses is guaranteed.
			stridedRefIter!(self.data, complex, String)
		}
	}

	fn references (&self) -> StridedCopyIter<'_, &str> {
		unsafe {
			// SAFETY: We store a `Vec` of structs, and `Vec` can be trusted to return the correct length, so alignment
			// and validity of the fields the iterator accesses is guaranteed.
			stridedCopyIter!(self.data, reference, &str)
		}
	}
}



//////
//
// Tests
//

#[test]
fn test_singlePrimitiveField ()
{
	// Create test data
	let data = SinglePrimitiveField::createTestData();
	let mut iter = data.iter();

	// Check each iteration
	assert_eq!(iter.len(), 4);
	assert_eq!(iter.next(), Some(0));
	assert_eq!(iter.len(), 3);
	assert_eq!(iter.next(), Some(1));
	assert_eq!(iter.len(), 2);
	assert_eq!(iter.next(), Some(2));
	assert_eq!(iter.len(), 1);
	assert_eq!(iter.next(), Some(3));
	assert_eq!(iter.len(), 0);
	assert_eq!(iter.next(), None);

	// Check ::collect()
	assert_eq!(data.iter().collect::<Vec<_>>(), vec![0, 1, 2, 3]);
}

#[test]
fn test_singlePrimitiveField_empty ()
{
	// Create test data
	let data = SinglePrimitiveField::empty();
	let mut iter = data.iter();

	// Check single iteration
	assert_eq!(iter.len(), 0);
	assert_eq!(iter.next(), None);

	// Check ::collect()
	assert_eq!(data.iter().collect::<Vec<_>>(), Vec::<u32>::new());
}

#[test]
fn test_singleComplexField ()
{
	// Create test data
	let data = SingleComplexField::createTestData();
	let mut iter = data.iter();

	// Check each iteration
	assert_eq!(iter.len(), 4);
	assert_eq!(iter.next().unwrap(), "zero");
	assert_eq!(iter.len(), 3);
	assert_eq!(iter.next().unwrap(), "one");
	assert_eq!(iter.len(), 2);
	assert_eq!(iter.next().unwrap(), "two");
	assert_eq!(iter.len(), 1);
	assert_eq!(iter.next().unwrap(), "three");
	assert_eq!(iter.len(), 0);
	assert_eq!(iter.next(), None);

	// Check ::collect()
	assert_eq!(data.iter().collect::<Vec<_>>(), vec!["zero", "one", "two", "three"]);
}

#[test]
fn test_singleComplexField_empty ()
{
	// Create test data
	let data = SingleComplexField::empty();
	let mut iter = data.iter();

	// Check single iteration
	assert_eq!(iter.len(), 0);
	assert_eq!(iter.next(), None);

	// Check ::collect()
	assert_eq!(data.iter().collect::<Vec<_>>(), Vec::<&String>::new());
}

#[test]
fn test_threeFields ()
{
	// Create test data
	let data = ThreeFields::createTestData();

	// Check each iteration over the numbers
	let mut numbers = data.numbers();
	assert_eq!(numbers.len(), 4);
	assert_eq!(numbers.next(), Some(0));
	assert_eq!(numbers.len(), 3);
	assert_eq!(numbers.next(), Some(1));
	assert_eq!(numbers.len(), 2);
	assert_eq!(numbers.next(), Some(2));
	assert_eq!(numbers.len(), 1);
	assert_eq!(numbers.next(), Some(3));
	assert_eq!(numbers.len(), 0);
	assert_eq!(numbers.next(), None);

	// Check each iteration over the complex fields
	let mut complexes = data.complexes();
	assert_eq!(complexes.len(), 4);
	assert_eq!(complexes.next().unwrap(), "zero");
	assert_eq!(complexes.len(), 3);
	assert_eq!(complexes.next().unwrap(), "one");
	assert_eq!(complexes.len(), 2);
	assert_eq!(complexes.next().unwrap(), "two");
	assert_eq!(complexes.len(), 1);
	assert_eq!(complexes.next().unwrap(), "three");
	assert_eq!(complexes.len(), 0);
	assert_eq!(complexes.next(), None);

	// Check each iteration over the references
	let mut references = data.references();
	assert_eq!(references.len(), 4);
	assert_eq!(references.next().unwrap(), "ten");
	assert_eq!(references.len(), 3);
	assert_eq!(references.next().unwrap(), "eleven");
	assert_eq!(references.len(), 2);
	assert_eq!(references.next().unwrap(), "twelve");
	assert_eq!(references.len(), 1);
	assert_eq!(references.next().unwrap(), "thirteen");
	assert_eq!(references.len(), 0);
	assert_eq!(references.next(), None);
}

#[test]
fn test_threeFields_empty ()
{
	// Create test data
	let data = ThreeFields::empty();

	// Check single iteration over the numbers
	let mut numbers = data.numbers();
	assert_eq!(numbers.len(), 0);
	assert_eq!(numbers.next(), None);

	// Check single iteration over the complex fields
	let mut complexes = data.complexes();
	assert_eq!(complexes.len(), 0);
	assert_eq!(complexes.next(), None);

	// Check single iteration over the references
	let mut references = data.references();
	assert_eq!(references.len(), 0);
	assert_eq!(references.next(), None);
}

#[test]
fn test_threeFieldsStructured ()
{
	// Create test data
	let data = ThreeFieldsStructured::createTestData();

	// Check each iteration over the numbers
	let mut numbers = data.numbers();
	assert_eq!(numbers.len(), 4);
	assert_eq!(numbers.next(), Some(0));
	assert_eq!(numbers.len(), 3);
	assert_eq!(numbers.next(), Some(1));
	assert_eq!(numbers.len(), 2);
	assert_eq!(numbers.next(), Some(2));
	assert_eq!(numbers.len(), 1);
	assert_eq!(numbers.next(), Some(3));
	assert_eq!(numbers.len(), 0);
	assert_eq!(numbers.next(), None);

	// Check each iteration over the complex fields
	let mut complexes = data.complexes();
	assert_eq!(complexes.len(), 4);
	assert_eq!(complexes.next().unwrap(), "zero");
	assert_eq!(complexes.len(), 3);
	assert_eq!(complexes.next().unwrap(), "one");
	assert_eq!(complexes.len(), 2);
	assert_eq!(complexes.next().unwrap(), "two");
	assert_eq!(complexes.len(), 1);
	assert_eq!(complexes.next().unwrap(), "three");
	assert_eq!(complexes.len(), 0);
	assert_eq!(complexes.next(), None);

	// Check each iteration over the references
	let mut references = data.references();
	assert_eq!(references.len(), 4);
	assert_eq!(references.next().unwrap(), "ten");
	assert_eq!(references.len(), 3);
	assert_eq!(references.next().unwrap(), "eleven");
	assert_eq!(references.len(), 2);
	assert_eq!(references.next().unwrap(), "twelve");
	assert_eq!(references.len(), 1);
	assert_eq!(references.next().unwrap(), "thirteen");
	assert_eq!(references.len(), 0);
	assert_eq!(references.next(), None);
}

#[test]
fn test_threeFieldsStructured_empty ()
{
	// Create test data
	let data = ThreeFieldsStructured::empty();

	// Check single iteration over the numbers
	let mut numbers = data.numbers();
	assert_eq!(numbers.len(), 0);
	assert_eq!(numbers.next(), None);

	// Check single iteration over the complex fields
	let mut complexes = data.complexes();
	assert_eq!(complexes.len(), 0);
	assert_eq!(complexes.next(), None);

	// Check single iteration over the references
	let mut references = data.references();
	assert_eq!(references.len(), 0);
	assert_eq!(references.next(), None);
}
