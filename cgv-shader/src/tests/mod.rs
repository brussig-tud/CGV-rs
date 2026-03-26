
//////
//
// Module definitions
//

/// Tests for the `compile` module.
#[cfg(feature="compilation")]
mod compile;

/// Tests for the `slang_runtime` module.
#[cfg(feature="slang_runtime")]
mod slang;



//////
//
// Tests for functionality in the root module
//

////
// Imports

// Local imports
#[expect(unused_imports)]
use crate::*;


////
// Tests

/* nothing here yet */
