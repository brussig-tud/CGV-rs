# Development ToDos

This document tracks architectural issues that either will eventually need attention or would just be beneficial to implement at some point.


## Performance

### Crate `cgv`

* **Unfiforms upload management**: investigate leveraging `wgpu::util::StagingBelt` for super efficient, streamlined uniform updates. Could be integrated either in `Application::prepareFrame` or perhaps via its own dedicated hook.


## Software Design

### Crate `cgv`

* **Multiple render passes and/or cameras**: Implement support for multiple active cameras and implement a stereo camera, both of which will result in multiple global render passes.
  * `GlobalPassInfo` overhaul: this struct is currently used both for declaring and for keeping the state of global render passes. It contains an `index` field that the player uses to access the `RenderState` corresponding to the global pass. When a camera declares a global pass however, it cannot know the index this pass will end up having in the global list of passes; the cleanest design would be to have a separate the declaration struct from the `GlobalPassInfo`.

* **View management and matrix stack**: The `viewing` uniforms in the render states currently rely on the cameras setting them. The cameras should however just report their view and projection matrices, with the player taking care of multiplying everything together correctly.
  * Give player mutable access to the uniforms (currently they're owned by the render states, which are behind shared references preventing mutation).
  * Implement a matrix stack initialized with the `view` and `projection` matrices of the camera that is currently rendering.
* ~~**Pre-defined renderers**: It makes sense for a graphics framework targeting visualization research to have a well thought-out concept of a data-driven renderer and supply a number of useful implementations out of the box. We need to design the renderer system such that it is generic and extensible and provide standard implementations, e.g. for spheres, boxes, superquadrics, lines, tubes etc.~~ Currently WiP (`develop_renderers` branch):
  * ~~Implement a proof-of-concept raycasted spheres renderer.~~ Done.
  * ~~Improve `renderer::HostData` ergonomics. There should just be a single `derive` macro for `InterleavedElem`, that also implements the other traits required for getting a blanket implementation of `renderer::HostData` and appropriate marker traits, depending on which attributes are decorated with `#[cgv_renderAttr(...)]` in the element struct.~~ Done.
  * Add a "IGeometryInput" interface to the `cgv` core shader library and add functionality to `renderer::data::gpu::BufferLayout` to auto-generate implementations that renderers can then just use (if they opt for requiring the `slang_runtime` feature), virtually eliminating the need for CPU-side boilerplate to adjust or reject render data with unsupported layouts and vastly improving renderer development ergonomics.

* **Data with a GUI-interface**: Especially in large structures like the `Player`, the disconnect between *Egui* types used to interface with GUI controls and the *nalgebra* types used for rendering requires additional mirror fields, which also need to be exposed with `pub(crate)` in order to be able to outsource the large code bodies for GUI managament into separate files. Some of it can be alleviated with dedicated custom widgets, but for simple things like single vectors this is not justifyable. Modelling the process mirroring internal data with *Egui*-compatible representations with dedicated functionality could make this less of a mess.

* **Input handling**: Currently, only desktop-centric input modalities are explicitly supported. Notably, the player currently translates pinch gestures into mouse wheel events. This is not optimal, we should instead provide all input in an agnostic way, and provide mapping to higher-level functions (e.g. compute zoom from mousewheel or gesture events, whatever is there at any given instant) that applications can use if they don't care about input mode specifics.

* **Graphics development ergonomics**: Currently, writing any sort of rendering code still requires plenty of repetitive, *WGPU*-specific boilerplate. It is neither possible nor intended to completely hide low-level rendering API details from clients, but certain very common tasks could use some helper facilities:
  * **Vertex Layout declaration**: Adding a `layoutDesc` function to a vertex struct (see e.g. basic example) that returns a `wgpu::VertexBufferLayout` for consumptprion by *WGPU* buffer APIs seems like it could be done by a procedural macro given certain constraints on the data type of fields we support.
  * **Creating a simple pipeline bindgroup for sampling textures**: The very common task of binding one or more textures to a pipeline for sampling in a shader could be mostly automated based solely on information in the `hal::Texture` objects.

### Crate `cgv_shader`

* **Sort out error types**: We're currently still using an ad-hoc design that just grew without knowing what the fully fleshed out crate architecture will look like. Rework, sanitize and document.
