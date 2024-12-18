# CGV-rs

A high performance, web-enabled prototyping framework for computer graphics and visualization research.


### Building for the web

Invoke `cargo build --release --target wasm32-unknown-unknown` from the repository root. Then, create the JavaScript bindings by running (again, from the repository root):
```bash
wasm-bindgen --no-typescript --target web --out-dir ./pkg --out-name "cgv-sample" ./target/wasm32-unknown-unknown/release/cgv-sample.wasm
```

This creates the (ignored by git) folder `pkg` inside the repository root. To run the example, serve `index.html` from `./pkg`, e.g. using
```bash
miniserve ./pkg --index index.html -p 8080 --tls-cert ./cgv-build/cert/miniserve.crt --tls-key cgv-build/cert/miniserve.key
```
to be able to run in a browser from `https://localhost:8080`. You can also upload the contents of the `pkg` folder to any static page webserver to deploy the example to the web.

If you don't have either of them already, you can install both the *wasm-bindgen* CLI tool and *miniserve* from source into your local user *Cargo* environment via
```bash
cargo install wasm-bindgen-cli miniserve
```
