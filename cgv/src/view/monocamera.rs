
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
pub struct MonoCamera {
	name: String,
	renderState: RenderState,
	defaultClearColor: wgpu::Color, // <- cached default clear color (we need it to be able to undo overrides)
	globalPasses: Vec<GlobalPassInfo>,
	parameters: CameraParameters,
	dirty: bool
}
impl MonoCamera
{
	fn declareRenderPasses (renderSetup: &RenderSetup) -> Vec<GlobalPassInfo>
	{
		vec![GlobalPassInfo {
			pass: GlobalPass::Simple,
			index: 0,
			clearColor: *renderSetup.defaultClearColor(),
			depthClearValue: renderSetup.defaultDepthClearValue(),
			completionCallback: None.into(),
		}]
	}

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
		let renderState = RenderState::new(context, framebuffer, Some(format!("{name}_renderState").as_str()));

		// Construct
		Self {
			name, defaultClearColor: *renderSetup.defaultClearColor(),
			globalPasses: Self::declareRenderPasses(renderSetup),
			renderState,
			parameters: CameraParameters::defaultWithAspect(resolution.x as f32 / resolution.y as f32),
			dirty: true
		}
	}
}

impl Camera<MonoCamera> for CameraObject<MonoCamera>
{
	fn projection (&self, _: &GlobalPass) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.borrowData().projection
	}
	fn projectionAt (&self, _: glm::UVec2) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.borrowData().projection
	}

	fn view (&self, _: &GlobalPass) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.borrowData().view
	}
	fn viewAt (&self, _: glm::UVec2) -> &glm::Mat4 {
		&self.renderState.viewingUniforms.borrowData().view
	}

	fn parameters (&self) -> &CameraParameters {
		&self.parameters
	}

	fn parameters_mut (&mut self) -> &mut CameraParameters {
		self.dirty = true;
		&mut self.parameters
	}

	fn onRenderSetupChange (&mut self, renderSetup: &RenderSetup)
	{
		self.globalPasses[0].clearColor = *renderSetup.defaultClearColor();
		self.globalPasses = MonoCamera::declareRenderPasses(renderSetup);
	}

	fn resize (&mut self, context: &Context, viewportDims: glm::UVec2) {
		self.renderState.framebuffer.resize(context, viewportDims);
		self.parameters.intrinsics.aspect = viewportDims.x as f32 / viewportDims.y as f32;
		self.dirty = true;
	}

	fn overrideClearColor (&mut self, passes: Option<&[&GlobalPassInfo]>, clearColor: Option<wgpu::Color>)
	{
		// Sanity-check that the provided global pass declaration actually belongs to us
		if let Some(passes) = passes {
			if passes.len() == 0 {return};
			let reqPtr = passes[0] as *const GlobalPassInfo;
			let ourPtr = &self.globalPasses[0] as *const GlobalPassInfo;
			assert_eq!(reqPtr, ourPtr, "overrideClearColor received a reference to a pass we don't own!");
		}
		if let Some(clearColor) = clearColor {
			self.globalPasses[0].clearColor = clearColor;
		}
		else {
			self.globalPasses[0].clearColor = self.defaultClearColor;
		}
	}

	fn update (&mut self) -> bool
	{
		let this = &mut self.user;
		if this.dirty
		{
			let mats = this.renderState.viewingUniforms.borrowData_mut();
			mats.projection = match this.parameters.intrinsics.fovY
			{
				FoV::Perspective(fovY) => transformClipspaceOGL2WGPU(&glm::perspective(
					this.parameters.intrinsics.aspect, fovY, this.parameters.intrinsics.zNear,
					this.parameters.intrinsics.zFar
				)),

				FoV::Orthographic(height)
				=> {
					let halfHeight = height * 0.5;
					let halfWidth = halfHeight * this.parameters.intrinsics.aspect;
					transformClipspaceOGL2WGPU(&glm::ortho(
						-halfWidth, halfWidth, -halfHeight, halfHeight, this.parameters.intrinsics.zNear,
						this.parameters.intrinsics.zFar
					))
				}
			};
			mats.view = glm::look_at(
				&this.parameters.extrinsics.eye,
				&(this.parameters.extrinsics.eye + this.parameters.extrinsics.dir*this.parameters.intrinsics.f),
				&this.parameters.extrinsics.up
			);

			// TODO: all this matrix juggling to fill the viewing uniforms struct should really happen completely
			// outside the cameras. This would also require enabling the player to mutate the render states, so some
			// sort of re-design is in order. We could also use this opportunity to include a proper matrix stack.
			mats.projView = mats.projection * mats.view;
			mats.projection_inv = mats.projection.try_inverse().unwrap();
			mats.view_inv = mats.view.try_inverse().unwrap();
			mats.projView_inv = mats.view_inv * mats.projection_inv;
			(mats.normal_inv, mats.normal) = {
				let normal3x3 = glm::mat4_to_mat3(&mats.view).transpose();
				(glm::mat3_to_mat4(&normal3x3), glm::mat3_to_mat4(&normal3x3.try_inverse().unwrap()))
			};
			this.dirty = false;
			true
		}
		else {
			false
		}
	}

	fn globalPasses (&self) -> GlobalPasses<'_>
	{
		GlobalPasses{info: &self.globalPasses, renderStates: std::slice::from_ref(&self.renderState)}
	}
	fn framebuffer (&self) -> &hal::Framebuffer {
		&self.renderState.framebuffer
	}

	fn name (&self) -> &str {
		&self.name
	}

	fn getDepthReadbackDispatcher (&self, pixelCoords: glm::UVec2) -> Option<DepthReadbackDispatcher<'_>>
	{
		self.renderState.framebuffer.depthStencil().map(|depthStencil| { DepthReadbackDispatcher::new(
			&pixelCoords, &Viewport {
				min: glm::vec2(0u32, 0u32), extent: depthStencil.dimsWH()
			},
			self.projectionAt(pixelCoords), self.viewAt(pixelCoords), &depthStencil
		)})
	}
}
