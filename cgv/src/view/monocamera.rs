
//////
//
// Imports
//

// Standard library
/* nothing here yet */

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
pub struct MonoCamera<'own> {
	name: String,
	framebuffer: &'own hal::Framebuffer,
	renderState: Box<RenderState>,
	globalPasses: Vec<GlobalPassDeclaration<'own>>,
	parameters: CameraParameters,
	dirty: bool
}

impl MonoCamera<'_>
{
	pub fn new (
		context: &Context, renderSetup: &RenderSetup, resolution: glm::UVec2, colorFormat: wgpu::TextureFormat,
		depthStencilFormat: hal::DepthStencilFormat, name: Option<&str>
	) -> Self
	{
		// Determine name
		let name: String = if let Some(name) = name { name } else { "UnnamedMonoCamera" }.into();

		// Create framebuffer
		let framebuffer = hal::FramebufferBuilder::withDims(&resolution)
			.withLabel(format!("{name}_framebuffer").as_str())
			.attachColor(colorFormat, Some(wgpu::TextureUsages::TEXTURE_BINDING))
			.attachDepthStencil(depthStencilFormat, Some(wgpu::TextureUsages::COPY_SRC))
			.build(context);

		// Initialize the main (and only) render state
		let renderState = Box::new(RenderState::new(
			context, framebuffer, Some(format!("{name}_renderState").as_str())
		));

		// Construct
		Self {
			name,
			framebuffer: util::statify(&renderState.framebuffer),
			globalPasses: vec![GlobalPassDeclaration {
				info: GlobalPassInfo {
					pass: GlobalPass::Simple,
					renderState: util::statify(&renderState),
					clearColor: *renderSetup.defaultClearColor(),
					depthClearValue: renderSetup.defaultDepthClearValue(),
				},
				completionCallback: None,
			}],
			renderState,
			parameters: CameraParameters::defaultWithAspect(resolution.x as f32 / resolution.y as f32),
			dirty: true
		}
	}
}

impl Camera for MonoCamera<'_>
{
	fn projection (&self, _: &GlobalPassDeclaration) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.borrowData().projection
	}
	fn projectionAt (&self, _: glm::UVec2) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.borrowData().projection
	}

	fn view (&self, _: &GlobalPassDeclaration) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.borrowData().view
	}
	fn viewAt (&self, _: glm::UVec2) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.borrowData().view
	}

	fn resize (&mut self, context: &Context, viewportDims: glm::UVec2) {
		self.renderState.framebuffer.resize(context, viewportDims);
		self.parameters.intrinsics.aspect = viewportDims.x as f32 / viewportDims.y as f32;
		self.dirty = true;
	}

	fn parameters (&self) -> &CameraParameters {
		&self.parameters
	}

	fn parameters_mut (&mut self) -> &mut CameraParameters {
		self.dirty = true;
		&mut self.parameters
	}

	fn update (&mut self) -> bool
	{
		if self.dirty
		{
			let mats = self.renderState.viewingUniforms.borrowData_mut();
			mats.projection = match self.parameters.intrinsics.fovY
			{
				FoV::Perspective(fovY) => transformClipspaceOGL2WGPU(&glm::perspective(
					self.parameters.intrinsics.aspect, fovY, self.parameters.intrinsics.zNear,
					self.parameters.intrinsics.zFar
				)),

				FoV::Orthographic(height)
				=> {
					let halfHeight = height * 0.5;
					let halfWidth = halfHeight * self.parameters.intrinsics.aspect;
					transformClipspaceOGL2WGPU(&glm::ortho(
						-halfWidth, halfWidth, -halfHeight, halfHeight, self.parameters.intrinsics.zNear,
						self.parameters.intrinsics.zFar
					))
				}
			};
			mats.view = glm::look_at(
				&self.parameters.extrinsics.eye,
				&(self.parameters.extrinsics.eye + self.parameters.extrinsics.dir*self.parameters.intrinsics.f),
				&self.parameters.extrinsics.up
			);
			self.dirty = false;
			true
		}
		else {
			false
		}
	}

	fn declareGlobalPasses (&self) -> &[GlobalPassDeclaration<'_>] {
		self.globalPasses.as_slice()
	}
	fn framebuffer (&self) -> &hal::Framebuffer {
		&self.framebuffer
	}

	fn name (&self) -> &str {
		&self.name
	}

	fn getDepthReadbackDispatcher (&self, pixelCoords: glm::UVec2) -> Option<DepthReadbackDispatcher<'_>>
	{
		self.framebuffer.depthStencil().map(|depthStencil| { DepthReadbackDispatcher::new(
			&pixelCoords, &Viewport {
				min: glm::vec2(0u32, 0u32), extend: depthStencil.dimsWH()
			},
			self.projectionAt(pixelCoords), self.viewAt(pixelCoords), &depthStencil
		)})
	}
}
