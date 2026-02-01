# CGV-rs

A high performance, web-enabled prototyping framework for computer graphics and visualization research.


### Building the examples

The example crates reside in `/examples/<example-name>` from the repository root, and the crates are named according to the scheme `ex-<example-name>`. For instance, to build and run the basic example showcasing how to set up a minimal app with rendering, run

```bash
cargo run --package ex-basic --release
```

from the repository root.


### Building for the web

Each example can also be built for the web via *wasm-bindgen*. Currently, the convention is to package each *CGV-rs* web app into its own self-contained wwwroot. In the following, this is illustrated for deploying each example into their own folder inside the `.gitignore`d `/pkg` directory of the repository root.

For the basic example, invoke `cargo build --package ex-basic --release --target wasm32-unknown-unknown` from the repository root. Then, create the JavaScript bindings by running (again, from the repository root):
```bash
wasm-bindgen --no-typescript --target web --out-dir ./pkg/ex-basic --out-name "ex-basic" ./target/wasm32-unknown-unknown/release/ex-basic.wasm
```

This creates the (ignored by git) folder `pkg` inside the repository root. To run the example, serve `index.html` from `./pkg`, e.g. using
```bash
miniserve ./pkg/ex-basic --index index.html -p 8080 --tls-cert ./cgv-build/cert/miniserve.crt --tls-key cgv-build/cert/miniserve.key
```
to be able to run in a browser from `https://localhost:8080`. You can also upload the contents of a `pkg` folder to any static page webserver to deploy the example to the web.

If you don't have either of them already, you can install both the *wasm-bindgen* CLI tool and *miniserve* from source into your local user *Cargo* environment via
```bash
cargo install wasm-bindgen-cli miniserve
```
