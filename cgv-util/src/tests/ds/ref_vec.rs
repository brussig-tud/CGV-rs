
//////
//
// Imports
//

// Local imports
use crate::ds::RefVec;



//////
//
// Tests
//

#[test]
fn test_new() {
	let refVec = RefVec::new(vec![1, 2, 3, 4, 5]);
	assert_eq!(refVec.len(), 5);
}

#[test]
fn test_empty() {
	let refVec: RefVec<bool> = RefVec::new(vec![]);
	assert_eq!(refVec.len(), 0);
	assert!(refVec.elements().is_empty());
	assert!(refVec.references().is_empty());
	assert!(refVec.is_empty());
}

#[test]
fn test_elements() {
	let vec = vec!["a".to_string(), "b".into()];
	let refVec = RefVec::new(vec);
	let elements = refVec.elements();
	assert_eq!(elements.len(), 2);
	assert_eq!(elements[0], "a");
	assert_eq!(elements[1], "b");
}

#[test]
fn test_references() {
	let vec = vec![11, 22, 33];
	let refVec = RefVec::new(vec);
	let references = refVec.references();
	assert_eq!(references.len(), 3);
	assert_eq!(*references[0], 11);
	assert_eq!(*references[1], 22);
	assert_eq!(*references[2], 33);
}

#[test]
fn test_deref() {
	let vec = vec![1.0, 2.0];
	let refVec = RefVec::new(vec);
	let refs: &[&f64] = &refVec; // <- this should compile as RefVec should deref to &[&T]
	assert_eq!(refs.len(), 2);
	assert_eq!(*refs[0], 1.0);
	assert_eq!(*refs[1], 2.0);
}

#[test]
fn test_from_vec() {
	let vec = vec![1, 2, 3];
	let refVec = RefVec::from(vec);
	assert_eq!(refVec.len(), 3);
}

#[test]
fn test_large_vec()
{
	let vec: Vec<_> = (0..1000).collect();
	let refVec = RefVec::new(vec);
	assert_eq!(refVec.len(), 1000);
	for i in 0..1000 {
		assert_eq!(refVec.elements()[i], i);
		assert_eq!(*refVec.references()[i], i);
	}
}
