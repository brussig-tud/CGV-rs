
//////
//
// Module definitions
//

/// Implements the `UniqueVec` collection.
pub mod unique_vec;
pub use unique_vec::{UniqueVec, UniqueVecElement, BTreeUniqueVec, HashUniqueVec}; // re-export

/// Implements the `BorrowVec` collection.
mod ref_vec;
pub use ref_vec::RefVec; // re-export
