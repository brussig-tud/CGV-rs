If automatic mipmap generation is desired, which [`gpu::mipmap::Generator`] to use. For a reasonably fast,
conservative default for this parameter, pass the value provided by [`defaultMipmapping()`]. If no mipmaps should be
generated, the `None` constant [`NO_MIPMAPS`] can be specified, which is already annotated with an appropriate
no-op/zero-cost `MipmapGenerator` type parameter.
