
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// Serialization
use serde;



//////
//
// Traits
//

/// The trait of modules that make up a [`compile::Environment`].
pub trait Module: Sized+Clone {}



//////
//
// Structs
//

///
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub struct BytesModule(Vec<u8>);
impl BytesModule
{
	///
	#[inline(always)]
	pub fn fromVec (bytes: Vec<u8>) -> Self {
		Self(bytes)
	}

	///
	#[inline(always)]
	pub fn fromSlice (bytes: &[u8]) -> Self {
		Self(bytes.to_owned())
	}

	///
	#[inline(always)]
	pub fn irBytes (&self) -> &[u8] {
		self.0.as_slice()
	}
}
impl Module for BytesModule {}
