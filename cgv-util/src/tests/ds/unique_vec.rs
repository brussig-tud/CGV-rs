
//////
//
// Imports
//

// Local imports
use crate::ds::unique_vec::*;



//////
//
// Structs
//

#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default,Debug,PartialEq,Eq)]
struct LargeKeyElement {
	id: String,
	data: i32,
}
impl UniqueVecElement for LargeKeyElement {
	type Key<'a> = String;

	fn key(&self) -> Self::Key<'_> {
		self.id.clone()
	}
}

#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default,Debug,PartialEq,Eq)]
struct RefKeyElement {
	id: String,
	data: i32,
}
impl UniqueVecElement for RefKeyElement {
	type Key<'a> = &'a str;

	fn key (&self) -> Self::Key<'_> {
		&self.id
	}
}

#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default,Debug,PartialEq,Eq)]
struct HashElement {
	name: String,
	data: i32,
}
impl UniqueVecElement for HashElement
{
	type Key<'a> = u64;

	fn key(&self) -> Self::Key<'_> {
		use std::collections::hash_map::DefaultHasher;
		use std::hash::{Hash, Hasher};
		let mut s = DefaultHasher::new();
		self.name.hash(&mut s);
		s.finish()
	}
}



//////
//
// Tests
//

#[test]
fn test_from_vec()
{
	let mut v: BTreeUniqueVec<_> = vec![
		LargeKeyElement { id: "a".into(), data: 1 },
		LargeKeyElement { id: "b".into(), data: 2 },
		LargeKeyElement { id: "a".into(), data: 3 }
	].into();

	assert_eq!(v.len(), 2);
	assert_eq!(v[0].id, "a");
	assert_eq!(v[1].id, "b");

	// Ensure keys were actually populated
	assert!(!v.push(LargeKeyElement { id: "b".into(), data: 4 } /* <- duplicate, should "fail" */));
}
#[test]
fn test_hash_from_vec()
{
	let mut v: HashUniqueVec<_> = vec![
		LargeKeyElement { id: "a".into(), data: 1 },
		LargeKeyElement { id: "b".into(), data: 2 },
		LargeKeyElement { id: "a".into(), data: 3 }
	].into();

	assert_eq!(v.len(), 2);
	assert_eq!(v[0].id, "a");
	assert_eq!(v[1].id, "b");

	// Ensure keys were actually populated
	assert!(!v.push(LargeKeyElement { id: "b".into(), data: 4 } /* <- duplicate, should "fail" */));
}

#[test]
fn test_from_vec_unchecked()
{
	let mut v = unsafe {
		BTreeUniqueVec::fromVec_unchecked(vec![
			LargeKeyElement { id: "a".into(), data: 1 },
			LargeKeyElement { id: "b".into(), data: 2 },
		])
	};

	assert_eq!(v.len(), 2);
	assert_eq!(v[0].id, "a");
	assert_eq!(v[1].id, "b");

	// Ensure keys were actually populated
	assert!(!v.push(LargeKeyElement { id: "a".into(), data: 3 } /* <- duplicate, should "fail" */));
}
#[test]
fn test_hash_from_vec_unchecked()
{
	let mut v = unsafe {
		HashUniqueVec::fromVec_unchecked(vec![
			LargeKeyElement { id: "a".into(), data: 1 },
			LargeKeyElement { id: "b".into(), data: 2 },
		])
	};

	assert_eq!(v.len(), 2);
	assert_eq!(v[0].id, "a");
	assert_eq!(v[1].id, "b");

	// Ensure keys were actually populated
	assert!(!v.push(LargeKeyElement { id: "a".into(), data: 3 } /* <- duplicate, should "fail" */));
}

#[test]
fn test_push_and_get()
{
	let mut v = BTreeUniqueVec::default();
	assert!(v.push(LargeKeyElement { id: "a".into(), data: 1 }));
	assert!(v.push(LargeKeyElement { id: "b".into(), data: 2 }));
	assert!(!v.push(LargeKeyElement { id: "a".into(), data: 3 }));

	assert_eq!(v.len(), 2);
	assert_eq!(v[0].id, "a");
	assert_eq!(v[1].id, "b");

	assert_eq!(v.get(0..1).unwrap()[0].id, "a");
	assert_eq!(v.get(..).unwrap().len(), 2);
	assert_eq!(v[0..2][1].id, "b");

	assert!(v.checkConsistency());
}
#[test]
fn test_hash_push_and_get()
{
	let mut v = HashUniqueVec::new();
	assert!(v.push(LargeKeyElement { id: "a".into(), data: 1 }));
	assert!(v.push(LargeKeyElement { id: "b".into(), data: 2 }));
	assert!(!v.push(LargeKeyElement { id: "a".into(), data: 3 }));

	assert_eq!(v.len(), 2);
	assert_eq!(v[0].id, "a");
	assert_eq!(v[1].id, "b");

	assert_eq!(v.get(0..1).unwrap()[0].id, "a");
	assert_eq!(v.get(..).unwrap().len(), 2);
	assert_eq!(v[0..2][1].id, "b");

	assert!(v.checkConsistency());
}

#[test]
fn test_first_and_last()
{
	let mut v = BTreeUniqueVec::new();
	v.push(LargeKeyElement { id: "a".into(), data: 1 });
	let first = v.first().unwrap();
	let last = v.last().unwrap();
	assert_eq!(first.id, "a");
	assert_eq!(first.data, 1);
	assert_eq!(last.id, "a");
	assert_eq!(last.data, 1);

	v.push(LargeKeyElement { id: "b".into(), data: 2 });
	assert!(v.checkConsistency());

	let first = v.first().unwrap();
	let last = v.last().unwrap();
	assert_eq!(first.id, "a");
	assert_eq!(first.data, 1);
	assert_eq!(last.id, "b");
	assert_eq!(last.data, 2);
}
#[test]
fn test_hash_first_and_last()
{
	let mut v = HashUniqueVec::new();
	v.push(LargeKeyElement { id: "a".into(), data: 1 });
	let first = v.first().unwrap();
	let last = v.last().unwrap();
	assert_eq!(first.id, "a");
	assert_eq!(first.data, 1);
	assert_eq!(last.id, "a");
	assert_eq!(last.data, 1);

	v.push(LargeKeyElement { id: "b".into(), data: 2 });
	assert!(v.checkConsistency());

	let first = v.first().unwrap();
	let last = v.last().unwrap();
	assert_eq!(first.id, "a");
	assert_eq!(first.data, 1);
	assert_eq!(last.id, "b");
	assert_eq!(last.data, 2);
}

#[test]
fn test_pop()
{
	let mut v = BTreeUniqueVec::new();
	v.push(LargeKeyElement { id: "a".into(), data: 1 });
	v.push(LargeKeyElement { id: "b".into(), data: 2 });
	assert!(v.checkConsistency());

	assert_eq!(v.pop(), Some(LargeKeyElement { id: "b".into(), data: 2 }));
	assert_eq!(v.len(), 1);
	assert!(v.checkConsistency());

	assert!(v.push(LargeKeyElement { id: "b".into(), data: 3 }));
	assert!(v.checkConsistency());
	assert_eq!(v[1].data, 3)
}
#[test]
fn test_hash_pop()
{
	let mut v = HashUniqueVec::new();
	v.push(LargeKeyElement { id: "a".into(), data: 1 });
	v.push(LargeKeyElement { id: "b".into(), data: 2 });
	assert!(v.checkConsistency());

	assert_eq!(v.pop(), Some(LargeKeyElement { id: "b".into(), data: 2 }));
	assert_eq!(v.len(), 1);
	assert!(v.checkConsistency());

	assert!(v.push(LargeKeyElement { id: "b".into(), data: 3 }));
	assert!(v.checkConsistency());
	assert_eq!(v[1].data, 3)
}

#[test]
fn test_contains_key_and_fetch()
{
	let mut v = BTreeUniqueVec::new();
	v.push(LargeKeyElement { id: "a".into(), data: 1 });
	v.push(LargeKeyElement { id: "b".into(), data: 2 });

	assert!(v.containsKey(&"a".to_string()));
	assert!(v.containsKey(&"b".to_string()));
	assert!(!v.containsKey(&"c".to_string()));

	assert_eq!(v.fetch(&"a".to_string()), Some(&LargeKeyElement { id: "a".into(), data: 1 }));
	assert_eq!(v.fetch(&"b".to_string()), Some(&LargeKeyElement { id: "b".into(), data: 2 }));
	assert_eq!(v.fetch(&"c".to_string()), None);
}
#[test]
fn test_hash_contains_key_and_fetch()
{
	let mut v = HashUniqueVec::new();
	v.push(LargeKeyElement { id: "a".into(), data: 1 });
	v.push(LargeKeyElement { id: "b".into(), data: 2 });

	assert!(v.containsKey(&"a".to_string()));
	assert!(v.containsKey(&"b".to_string()));
	assert!(!v.containsKey(&"c".to_string()));

	assert_eq!(v.fetch(&"a".to_string()), Some(&LargeKeyElement { id: "a".into(), data: 1 }));
	assert_eq!(v.fetch(&"b".to_string()), Some(&LargeKeyElement { id: "b".into(), data: 2 }));
	assert_eq!(v.fetch(&"c".to_string()), None);
}

#[test]
fn test_remove()
{
	let mut v = BTreeUniqueVec::new();
	v.push(LargeKeyElement { id: "a".into(), data: 1 });
	v.push(LargeKeyElement { id: "b".into(), data: 2 });
	v.push(LargeKeyElement { id: "c".into(), data: 3 });

	assert_eq!(v.remove(1).id, "b");
	assert_eq!(v.len(), 2);
	assert!(v.checkConsistency());
	assert_eq!(v[1].id, "c");

	assert!(v.push(LargeKeyElement { id: "b".into(), data: 4 }));
	assert!(v.checkConsistency());
	assert_eq!(v[2].data, 4);
}
#[test]
fn test_hash_remove()
{
	let mut v = HashUniqueVec::new();
	v.push(LargeKeyElement { id: "a".into(), data: 1 });
	v.push(LargeKeyElement { id: "b".into(), data: 2 });
	v.push(LargeKeyElement { id: "c".into(), data: 3 });

	assert_eq!(v.remove(1).id, "b");
	assert_eq!(v.len(), 2);
	assert!(v.checkConsistency());
	assert_eq!(v[1].id, "c");

	assert!(v.push(LargeKeyElement { id: "b".into(), data: 4 }));
	assert!(v.checkConsistency());
	assert_eq!(v[2].data, 4);
}

#[test]
fn test_adhoc_key()
{
	let mut v = BTreeUniqueVec::new();
	assert!(v.push(HashElement { name: "a".into(), data: 1 }));
	assert!(v.push(HashElement { name: "b".into(), data: 2 }));
	assert!(!v.push(HashElement { name: "a".into(), data: 3 }));
	assert_eq!(v.len(), 2);
}
#[test]
fn test_hash_adhoc_key()
{
	let mut v = HashUniqueVec::new();
	assert!(v.push(HashElement { name: "a".into(), data: 1 }));
	assert!(v.push(HashElement { name: "b".into(), data: 2 }));
	assert!(!v.push(HashElement { name: "a".into(), data: 3 }));
	assert_eq!(v.len(), 2);
}

#[test]
fn stresstest_large_keys()
{
	let mut v = BTreeUniqueVec::new();
	v.push(LargeKeyElement { id: "hello".into(), data: 0 });
	v.push(LargeKeyElement { id: "world".into(), data: 1 });

	// We push many elements to force reallocation
	for i in 0..1000 {
		v.push(LargeKeyElement { id: format!("additional element {}", 1+i), data: 2+i });
	}

	// Check consistency after many pushes
	assert!(v.checkConsistency());
	assert_eq!(v[0].id, "hello");
	assert_eq!(v[1].data, 1);
	assert_eq!(v[2].id, "additional element 1");
	assert_eq!(v[3].data, 3);
	assert_eq!(v.len(), 1002);
	assert!(v.containsKey(&"world".to_string()));
	assert!(v.containsKey(&"additional element 256".to_string()));

	// Check uniqueness still works after many pushes
	assert!(!v.push(LargeKeyElement { id: "world".into(), data: 123 }));
	assert!(!v.push(LargeKeyElement { id: "additional element 128".into(), data: 321 }));
	assert_eq!(v.len(), 1002);
}
#[test]
fn stresstest_hash_large_keys()
{
	let mut v = HashUniqueVec::new();
	v.push(LargeKeyElement { id: "hello".into(), data: 0 });
	v.push(LargeKeyElement { id: "world".into(), data: 1 });

	// We push many elements to force reallocation
	for i in 0..1000 {
		v.push(LargeKeyElement { id: format!("additional element {}", 1+i), data: 2+i });
	}

	// Check consistency after many pushes
	assert!(v.checkConsistency());
	assert_eq!(v[0].id, "hello");
	assert_eq!(v[1].data, 1);
	assert_eq!(v[2].id, "additional element 1");
	assert_eq!(v[3].data, 3);
	assert_eq!(v.len(), 1002);
	assert!(v.containsKey(&"world".to_string()));
	assert!(v.containsKey(&"additional element 256".to_string()));

	// Check uniqueness still works after many pushes
	assert!(!v.push(LargeKeyElement { id: "world".into(), data: 123 }));
	assert!(!v.push(LargeKeyElement { id: "additional element 128".into(), data: 321 }));
	assert_eq!(v.len(), 1002);
}

#[test]
fn stresstest_ref_keys()
{
	let mut v = BTreeUniqueVec::new();
	v.push(RefKeyElement { id: "hello".into(), data: 0 });
	v.push(RefKeyElement { id: "world".into(), data: 1 });

	// We push many elements to force reallocation
	for i in 0..1000 {
		v.push(RefKeyElement { id: format!("additional element {}", 1+i), data: 2+i });
	}

	// Check consistency after many pushes
	assert!(v.checkConsistency());
	assert_eq!(v[0].id, "hello");
	assert_eq!(v[1].data, 1);
	assert_eq!(v[2].id, "additional element 1");
	assert_eq!(v[3].data, 3);
	assert_eq!(v.len(), 1002);
	assert!(v.containsKey(&"world"));
	assert!(v.containsKey(&"additional element 256"));

	// Check uniqueness still works after many pushes
	assert!(!v.push(RefKeyElement { id: "world".into(), data: 123 }));
	assert!(!v.push(RefKeyElement { id: "additional element 128".into(), data: 321 }));
	assert_eq!(v.len(), 1002);
}
#[test]
fn stresstest_hash_ref_keys()
{
	let mut v = HashUniqueVec::new();
	v.push(RefKeyElement { id: "hello".into(), data: 0 });
	v.push(RefKeyElement { id: "world".into(), data: 1 });

	// We push many elements to force reallocation
	for i in 0..1000 {
		v.push(RefKeyElement { id: format!("additional element {}", 1+i), data: 2+i });
	}

	// Check consistency after many pushes
	assert!(v.checkConsistency());
	assert_eq!(v[0].id, "hello");
	assert_eq!(v[1].data, 1);
	assert_eq!(v[2].id, "additional element 1");
	assert_eq!(v[3].data, 3);
	assert_eq!(v.len(), 1002);
	assert!(v.containsKey(&"world"));
	assert!(v.containsKey(&"additional element 256"));

	// Check uniqueness still works after many pushes
	assert!(!v.push(RefKeyElement { id: "world".into(), data: 123 }));
	assert!(!v.push(RefKeyElement { id: "additional element 128".into(), data: 321 }));
	assert_eq!(v.len(), 1002);
}

#[test]
fn test_move_collection()
{
	let mut v = BTreeUniqueVec::new();
	v.push("a".to_string());

	// Move v to a new location
	let mut v2 = v;

	// As the keys can only reference the heap, the move should not cause dangling pointers
	assert!(!v2.push("a".into()));
	assert!(v2.push("b".into()));
	assert_eq!(v2.len(), 2);
}
#[test]
fn test_hash_move_collection()
{
	let mut v = HashUniqueVec::new();
	v.push("a".to_string());

	// Move v to a new location
	let mut v2 = v;

	// As the keys can only reference the heap, the move should not cause dangling pointers
	assert!(!v2.push("a".into()));
	assert!(v2.push("b".into()));
	assert_eq!(v2.len(), 2);
}

#[test]
fn test_extend()
{

	let mut v1 = BTreeUniqueVec::new();
	v1.push(1);

	v1.extend(vec![1, 2]);

	assert_eq!(v1.len(), 2);
	assert_eq!(v1[0], 1);
	assert_eq!(v1[1], 2);
}
#[test]
fn test_hash_extend()
{
	let mut v1 = HashUniqueVec::new();
	v1.push(1);

	v1.extend(vec![1, 2]);

	assert_eq!(v1.len(), 2);
	assert_eq!(v1[0], 1);
	assert_eq!(v1[1], 2);
}

#[test]
fn test_join()
{
	let mut v1 = BTreeUniqueVec::new();
	v1.push(1);
	v1.push(2);

	let mut v2 = BTreeUniqueVec::new();
	v2.push(2); // <- duplicate
	v2.push(3);

	let v = UniqueVec::join(&v1, &v2);

	assert_eq!(v.len(), 3);
	assert_eq!(v[0], 1);
	assert_eq!(v[1], 2);
	assert_eq!(v[2], 3);

	// Verify v1 and v2 are not consumed
	assert_eq!(v1.len(), 2);
	assert_eq!(v2.len(), 2);
}
#[test]
fn test_hash_join()
{
	let mut v1 = HashUniqueVec::new();
	v1.push(1);
	v1.push(2);

	let mut v2 = HashUniqueVec::new();
	v2.push(2); // <- duplicate
	v2.push(3);

	let v = UniqueVec::join(&v1, &v2);

	assert_eq!(v.len(), 3);
	assert_eq!(v[0], 1);
	assert_eq!(v[1], 2);
	assert_eq!(v[2], 3);

	// Verify v1 and v2 are not consumed
	assert_eq!(v1.len(), 2);
	assert_eq!(v2.len(), 2);
}

#[test]
fn test_join_move()
{
	let mut v1 = BTreeUniqueVec::new();
	v1.push(1);
	v1.push(2);

	let mut v2 = BTreeUniqueVec::new();
	v2.push(2); // <- duplicate
	v2.push(3);

	let v = UniqueVec::join_move(v1, v2);

	assert_eq!(v.len(), 3);
	assert_eq!(v[0], 1);
	assert_eq!(v[1], 2);
	assert_eq!(v[2], 3);
}
#[test]
fn test_hash_join_move()
{
	let mut v1 = HashUniqueVec::new();
	v1.push(1);
	v1.push(2);

	let mut v2 = HashUniqueVec::new();
	v2.push(2); // <- duplicate
	v2.push(3);

	let v = UniqueVec::join_move(v1, v2);

	assert_eq!(v.len(), 3);
	assert_eq!(v[0], 1);
	assert_eq!(v[1], 2);
	assert_eq!(v[2], 3);
}

#[cfg(feature="serde")]
#[test]
fn test_serde()
{
	let mut orig = BTreeUniqueVec::new();
	orig.push(LargeKeyElement { id: "a".into(), data: 1 });
	orig.push(LargeKeyElement { id: "b".into(), data: 2 });

	let json = serde_json::to_string(&orig).unwrap();
	let deser: BTreeUniqueVec<LargeKeyElement> = serde_json::from_str(&json).unwrap();

	assert_eq!(orig.len(), deser.len());
	assert_eq!(orig[0], deser[0]);
	assert_eq!(orig[1], deser[1]);
}
#[cfg(feature="serde")]
#[test]
fn test_hash_serde()
{
	let mut orig = HashUniqueVec::new();
	orig.push(LargeKeyElement { id: "a".into(), data: 1 });
	orig.push(LargeKeyElement { id: "b".into(), data: 2 });

	let json = serde_json::to_string(&orig).unwrap();
	let deser: HashUniqueVec<LargeKeyElement> = serde_json::from_str(&json).unwrap();

	assert_eq!(orig.len(), deser.len());
	assert_eq!(orig[0], deser[0]);
	assert_eq!(orig[1], deser[1]);
}

#[cfg(feature="serde")]
#[test]
fn test_serde_duplicate()
{
	let json = r#"[{"id": "a", "data": 1}, {"id": "a", "data": 2}]"#;
	let result: Result<BTreeUniqueVec<LargeKeyElement>, serde_json::Error> = serde_json::from_str(json);

	match result {
		Err(e) => assert!(e.to_string().contains("duplicate element")),
		Ok(_) => panic!("Should have failed due to duplicate element"),
	}
}
#[cfg(feature="serde")]
#[test]
fn test_hash_serde_duplicate()
{
	let json = r#"[{"id": "a", "data": 1}, {"id": "a", "data": 2}]"#;
	let result: Result<HashUniqueVec<LargeKeyElement>, serde_json::Error> = serde_json::from_str(json);

	match result {
		Err(e) => assert!(e.to_string().contains("duplicate element")),
		Ok(_) => panic!("Should have failed due to duplicate element"),
	}
}
