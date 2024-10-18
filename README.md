# CGV-rs

A high performance, web-enabled prototyping framework for computer graphics and visualization research.


### Building for the web

Invoke `cargo build --release --target wasm32-unknown-unknown` from the repository root. Then, create the JavaScript bindings by running (again, from the repository root):
```bash
wasm-bindgen --no-typescript --target web --out-dir ./pkg --out-name "cgv-sample" ./target/wasm32-unknown-unknown/release/cgv-sample.wasm
```

This creates the (ignored by git) folder `pkg` inside the repository root. To run the example, serve `index.html` from `./pkg`, e.g. using
```bash
miniserve ./pkg --index index.html -p 8080
```
to be able to run in a browser from `localhost:8080`. You can also upload the folder `pkg` to any static page webserver to deploy the example to the web.

If you don't have it already, you can install *miniserve* from source into your local user *Cargo* package repository via
```bash
cargo install miniserve
```
