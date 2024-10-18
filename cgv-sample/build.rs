
// Custom build steps - in this example, it is used only for WASM support, by invoking the proper wasm-bindgen command,
// auto-generating an appropriate index.html, and packaging everything (including resources like the favicon and app
// manifest) for hosting.
fn main() -> cgv::Result<()> {
	// CGV-rs provides automation for this (including a check if we're even building for wasm32 to begin with)
	cgv::build::webDeployIfWasm("../pkg", &["Cargo.toml"])
}
