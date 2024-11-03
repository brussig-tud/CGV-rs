
//////
//
// Imports
//

// Standard library
/* Nothing here yet */

// Local imports
use crate::*;
use view::*;



//////
//
// Classes
//

////
// MonoCamera

/// A camera producing a single, monoscopic image of the scene.
pub struct MonoCamera {
	name: String,
	renderTarget: Option<Box<hal::RenderTarget>>,
	renderState: Box<RenderState>,
	globalPasses: Vec<GlobalPassDeclaration<'static>>
}

impl MonoCamera
{
	pub fn new (
		context: &Context, allocateOwnedRenderTarget: Option<glm::UVec2>, renderSetup: &RenderSetup, name: Option<&str>
	) -> Box<Self>
	{
		// Determine name
		let name: String = if let Some(name) = name { name } else { "UnnamedMonoCamera" }.into();

		// Initialize render target if requested
		let renderTarget = if let Some(dims) = &allocateOwnedRenderTarget {
			Some(Box::new(hal::RenderTarget::new(
				context, dims, renderSetup.colorFormat, renderSetup.depthStencilFormat().into(), name.as_str()
			)))
		}
		else {
			None
		};

		// Initialize the main (and only) render state
		let renderState = Box::new(RenderState::new(
			context,
			if let Some(rt) = util::statify(&renderTarget) {
				ColorAttachment::Texture(&rt.color)
			} else {
				ColorAttachment::Surface
			},
			Some(hal::DepthStencilFormat::D32),
			Some((name.clone() + "_renderState").as_str())
		));

		// Create Self with fields initialized up to now
		let mut result = Box::new(Self {
			name,
			renderTarget,
			renderState,
			globalPasses: Vec::new(),
		});

		// setup global passes
		result.globalPasses.push(GlobalPassDeclaration {
			pass: GlobalPass::Simple,
			renderState: util::mutify(&result.renderState),
			completionCallback: None,
		});

		// Done!
		result
	}
}

impl Camera for MonoCamera
{
	fn projection (&self, _: &GlobalPassDeclaration) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.data.projection
	}

	fn view (&self, _: &GlobalPassDeclaration) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.data.view
	}

	fn resize (&mut self, context: &Context, viewportDims: &glm::UVec2, interactor: &dyn CameraInteractor)
	{
		if self.renderTarget.is_some() {
			self.renderTarget = Some(Box::new(hal::RenderTarget::new(
				context, viewportDims, wgpu::TextureFormat::Bgra8Unorm, hal::DepthStencilFormat::D32, self.name.as_str()
			)))
		}
		self.renderState.updateSize(context, viewportDims);
		self.renderState.viewingUniforms.data.projection = interactor.projection(viewportDims);
	}

	fn update (&mut self, interactor: &dyn CameraInteractor) {
		self.renderState.viewingUniforms.data.view = *interactor.view();
	}

	fn declareGlobalPasses (&self) ->&[GlobalPassDeclaration] {
		self.globalPasses.as_slice()
	}

	fn name (&self) -> &str { &self.name }

	/*fn getDepthValue (&self, context: &Context, surfaceCoords: &glm::UVec2) -> Option<f32>
	{
		if let Some(da) = util::statify(&self.renderState.depthStencilAttachment) {
			let mut enc = context.device.create_command_encoder(
				&wgpu::CommandEncoderDescriptor {label: Some("ReadbackTestCommandEncoder")}
			);
			enc.copy_texture_to_buffer(
				*da.texture.readbackView_tex.as_ref().unwrap(),
				*da.texture.readbackView_buf.as_ref().unwrap(), da.texture.descriptor.size
			);
			context.queue.submit(Some(enc.finish()));
			let dims = da.texture.dims2WH();
			let rowStride = da.texture.size.actual as u32 / (dims.y * size_of::<f32>() as u32);
			let loc = (surfaceCoords.y*rowStride + surfaceCoords.x) as usize;
			tracing::debug!("Querying {:?} - stride={rowStride}", surfaceCoords);
			{
				tracing::debug!("Mapping...");
				let buf = da.texture.readbackBuffer.as_ref().unwrap().as_ref();
				let depth = std::sync::atomic::AtomicU32::new((-1f32).to_bits());
				let depthRef = util::mutify(&depth);
				buf.slice(0..da.texture.size.actual).map_async(
					wgpu::MapMode::Read, move |result| {
						if result.is_ok() {
							tracing::debug!("Mapped!!!");
							let bufView = buf.slice(..).get_mapped_range();
							let bytes = bufView.iter().as_slice();
							let floats = unsafe { std::slice::from_raw_parts(
								bytes.as_ptr() as *const f32, bytes.len()/size_of::<f32>()
							)};
							let depth = floats[loc];
							tracing::warn!("Depth={depth}");
							depthRef.store(depth.to_bits(), std::sync::atomic::Ordering::Relaxed);
							drop(bufView);
							buf.unmap();
							tracing::debug!("Unmapped!!!");
						}
						else {
							tracing::debug!("Mapping Failure!!!");
						}
					}
				);
				tracing::debug!("Polling...");
				context.device.poll(wgpu::Maintain::Wait);
				context.queue.submit([]);
				Some(f32::from_bits(depth.load(std::sync::atomic::Ordering::Acquire)))
			}
		}
		else { None }
	}*/
}
