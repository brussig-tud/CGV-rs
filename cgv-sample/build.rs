
/// Custom build steps – in this example, used only for WASM support, by auto-generating an appropriate index.html and
/// packaging that and related resources like the favicon and app manifest for hosting. At the moment, *wasm-bindgen*
/// still needs to be invoked manually afterwards to insert the actual WASM module into the package.
///
/// **ToDo**: look into `cargo install` to see if it can be used instead of the build script to include *wasm-bindgen*
fn main() -> cgv::Result<()> {
	// CGV-rs provides automation for this (including a check if we're even building for wasm32 to begin with)
	cgv::build::webDeployIfWasm("../pkg", &["Cargo.toml"])
}
