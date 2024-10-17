# CGV-rs
A high performance, web-enabled rapid prototyping framework for computer graphics and visualization research.


### Building for the web

Invoke `cargo build --target wasm32-unknown-unknown` from the repository root. Then, create the JavaScript bindings by running (again, from the repository root):
```bash
wasm-bindgen --no-typescript --target web --out-dir ./pkg --out-name "cgv-sample" ./target/wasm32-unknown-unknown/debug/cgv-sample.wasm
```

This creates the (ignored by git) folder `pkg` inside the repository root. To run the example, serve `index.html` from the repository root afterwards, e.g. using
```bash
miniserve . --index index.html -p 8080
```
to be able to run in a browser form `localhost:8080`. You can also upload the file `/target/index.html` and the folders `/target/res` and `/pkg` to any static page webserver to deploy the example to the web.

If you don't have it already, you can install `miniserve` from source via
```bash
cargo install miniserve
```
