# OnTubeVis-rs
A rewrite of OnTubeVis in Rust and WGPU

### Building for the web

Invoke `wasm-pack build --target web` from the repository root. Serve the `index.html` file from the repository root afterwards, e.g. using
```bash
miniserve . --index index.html -p 8080
```

If you don't have it already, you can install `miniserve` from source via
```bash
cargo install miniserve
```
