
//////
//
// Imports
//

// Standard library
/* nothing here yet */



//////
//
// Macros
//

/// Assert that the passed expression causes a panic.
#[macro_export]
macro_rules! assertPanics {
	($expression:expr) => {
		assert!(std::panic::catch_unwind(|| $expression).is_err());
	};
}
pub use assertPanics;
