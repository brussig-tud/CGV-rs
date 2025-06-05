
/// Custom build steps – currently, we only propagate the *Cargo*
/// [`TARGET`](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts)
/// variable to the crate when compiling.
fn main()
{
	// Propagate `TARGET` to the compiler so it's accessible at compile time
	println!(
		"cargo::rustc-env=CGV_TARGET_TRIPLE_CARGO={}",
		std::env::var("TARGET").expect("`TARGET` should be defined by Cargo")
	);
}
