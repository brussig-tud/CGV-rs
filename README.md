# OnTubeVis-rs
A rewrite of [OnTubeVis](https://github.com/brussig-tud/OnTubeVis) in Rust and *wgpu*.


### Building for the web

Invoke `wasm-pack build --target web` from the repository root. Serve the `/index.html` file from the repository root afterwards, e.g. using
```bash
miniserve . --index index.html -p 8080
```
to be able to run in a browser form `localhost:8080`. You can also upload the files `/index.html`, `/favicon.ico` and the folder `/pkg` to any static page webserver to deploy *OnTubeVis-rs* to the web.

If you don't have it already, you can install `miniserve` from source via
```bash
cargo install miniserve
```
