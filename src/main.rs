
//////
//
// Imports
//

// Standard library
/* nothing here yet */

// WASM Bindgen
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Anyhow
use anyhow::Result;

// Local modules
use ontubevis_rs::*;



//////
//
// Functions
//

// Application entry point
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(main))]
pub fn main() -> Result<()>{
	// Just hand off control flow
	run()
}
