
//////
//
// Module definitions
//

/// Tests for the `ds` module.
mod ds;

/// Tests for the `math` module.
mod math;



//////
//
// Tests for functionality in the root module
//

////
// Imports

// Local imports
use crate::*;


////
// Tests

#[test]
fn test_concatIfSome()
{
	let someStr = Some("hello");
	let noneStr: Option<&str> = None;
	let someString = Some("world".to_string());

	assert_eq!(concatIfSome(&someStr, " world"), Some("hello world".to_string()));
	assert_eq!(concatIfSome(&noneStr, " world"), None);
	assert_eq!(concatIfSome(&someString, "!"), Some("world!".to_string()));
}

#[test]
fn test_substrByteRange()
{
	// Our test string
	let whole = "The quick brown fox jumps over the lazy dog";
	
	// Middle subslice
	let sub = &whole[4..9]; // "quick"
	assert_eq!(substrByteRange(whole, sub), 4..9);

	// Full slice
	assert_eq!(substrByteRange(whole, whole), 0..whole.len());

	// Empty subslice at start
	assert_eq!(substrByteRange(whole, &whole[0..0]), 0..0);

	// Empty subslice at end
	assert_eq!(substrByteRange(whole, &whole[whole.len()..whole.len()]), whole.len()..whole.len());
}

#[test]
fn test_substrByteRange_utf8()
{
	// Test with German umlauts (2 bytes each)
	let whole = "Grüße aus der Küche";
	// "Grüße"
	// G: 1 byte (0)
	// r: 1 byte (1)
	// ü: 2 bytes (2,3)
	// ß: 2 bytes (4,5)
	// e: 1 byte (6)
	
	let sub1 = &whole[2..6]; // "üß"
	assert_eq!(substrByteRange(whole, sub1), 2..6);
	let sub2 = &whole[11..14]; // "der"
	assert_eq!(substrByteRange(whole, sub2), 11..14);

	// Test with Emoji (4 bytes)
	let emojiWhole = "Hello 🦀 Rust";
	// H e l l o (space) -> 6 bytes (0..6)
	// 🦀 -> 4 bytes (6..10)
	// (space) R u s t -> 5 bytes (10..15)

	let crab = &emojiWhole[6..10];
	assert_eq!(substrByteRange(emojiWhole, crab), 6..10);
	let rust = &emojiWhole[11..15];
	assert_eq!(substrByteRange(emojiWhole, rust), 11..15);
}

#[test]
#[should_panic]
fn test_substrByteRange_invalid_start() {
	let whole = "hello";
	let other = "world";
	let _ = substrByteRange(whole, other);
}

#[test]
#[should_panic]
fn test_substrByteRange_invalid_end() {
	let whole = "hello";
	let sub = "hello world";
	let _ = substrByteRange(whole, sub);
}
