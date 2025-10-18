
//////
//
// Imports
//

// Standard library
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU32, AtomicU64};



//////
//
// Globals
//

/// A realm of unique unsigned 32-bit integers.
pub static GLOBAL_REALM_U32: LazyLock<RealmU32> = LazyLock::new(|| RealmU32::default());

/// A realm of unique unsigned 64-bit integers.
pub static GLOBAL_REALM_U64: LazyLock<RealmU64> = LazyLock::new(|| RealmU64::default());



//////
//
// Traits
//

/// Represents a realm where all entities within must be unique.
pub trait Realm<EntityType> : Sized+Send+Sync {
	fn newEntity (&self) -> EntityType;
}



//////
//
// Classes
//

/// A realm of unique unsigned 32-bit integers.
#[derive(Default)]
pub struct RealmU32 {
	counter: AtomicU32
}
impl RealmU32 {
	pub const fn zero () -> RealmU32 {
		RealmU32 { counter: AtomicU32::new(0) }
	}

	pub const fn one () -> RealmU32 {
		RealmU32 { counter: AtomicU32::new(1) }
	}
}
unsafe impl Send for RealmU32 {}
unsafe impl Sync for RealmU32 {}
impl Realm<u32> for RealmU32 {
	/// Create and return a new 32-bit integer that is unique to this realm.
	fn newEntity (&self) -> u32 {
		self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
	}
}

/// A realm of unique unsigned 64-bit integers.
#[derive(Default)]
pub struct RealmU64 {
	counter: AtomicU64
}
impl RealmU64 {
	pub const fn zero () -> RealmU64 {
		RealmU64 { counter: AtomicU64::new(0) }
	}

	pub const fn one () -> RealmU64 {
		RealmU64 { counter: AtomicU64::new(1) }
	}
}
unsafe impl Send for RealmU64 {}
unsafe impl Sync for RealmU64 {}
impl Realm<u64> for RealmU64 {
	/// Create and return a new 64-bit integer that is unique to this realm.
	fn newEntity (&self) -> u64 {
		self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
	}
}



//////
//
// Functions
//

/// Sample a `u32` from the corresponding global realm (i.e. will be unique within the current process). This is a
/// convenience shorthand for calling `unique::GLOBAL_REALM_U32.newEntity()` that also reads semantically sound:
///
/// ```let id = unique::uint32()```
#[inline(always)]
pub fn uint32 () -> u32 {
	GLOBAL_REALM_U32.newEntity()
}

/// Sample a `u64` from the corresponding global realm (i.e. will be unique within the current process). This is a
/// convenience shorthand for calling `unique::GLOBAL_REALM_U64.newEntity()` that also reads semantically sound:
///
/// ```let id = unique::uint64()```
#[inline(always)]
pub fn uint64 () -> u64 {
	GLOBAL_REALM_U64.newEntity()
}
