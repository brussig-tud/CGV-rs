# Development ToDos

This document tracks architectural issues that either will eventually need attention or would just be beneficial to implement at some point.


## Performance

### Crate `cgv`

* **Unfiforms upload management**: investigate leveraging `wgpu::util::StagingBelt` for super efficient, streamlined uniform updates. Could be integrated either in `Application::prepareFrame` or perhaps via its own dedicated hook.


## Software Design

### Crate `cgv`

* **Proper interior mutability**: `Player` currently uses volatile writes on unsafely `mut`-casted references to implement interior mutability. White it's probably the most performant way, it has several issues – ugly code being the least problematic of them. Could be the underlying cause of several hard to reproduce current bugs in the player. The performance overhead of proper interior mutability is most likely a non-issue, so it should be changed.
* **Data with a GUI-interface**: Especially in large structures like the `Player`, the disconnect between *Egui* types used to interface with GUI controls and the *nalgebra* types used for rendering requires additional mirror fields, which also need to be exposed with `pub(crate)` in order to be able to outsource the large code bodies for GUI managament into separate files. Some of it can be alleviated with dedicated custom widgets, but for simple things like single vectors this is not justifyable. Modelling the process mirroring internal data with *Egui*-compatible representations with dedicated functionality could make this less of a mess.

### Crate `cgv_shader`

* **Sort out error types**: We're currently still using an ad-hoc design that just grew without knowing what the fully fleshed out crate architecture will look like. Rework, sanitize and document.
