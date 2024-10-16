# CGV-rs
A high performance, web-enabled rapid prototyping framework for computer graphics and visualization research.


### Building for the web

Invoke `wasm-pack build --target web` from the repository root. To run the example, serve `/target/index.html` file from the repository root afterwards, e.g. using
```bash
miniserve . --index /target/index.html -p 8080
```
to be able to run in a browser form `localhost:8080`. You can also upload the file `/target/index.html` and the folders `/target/res` and `/pkg` to any static page webserver to deploy the example to the web.

If you don't have it already, you can install `miniserve` from source via
```bash
cargo install miniserve
```
