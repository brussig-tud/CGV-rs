
//////
//
// Imports
//

// WGPU API
use wgpu;

// Local imports
use crate::*;



//////
//
// Classes
//

////
// ViewportCompositor

/// A helper handling the final compositing of the rendered scene onto the egui viewport panel.
pub(crate) struct ViewportCompositor
{
	pub(crate) invGamma_all: f32,
	pub(crate) invGamma: glm::Vec3,
	pub(crate) gammaUniform: hal::UniformGroup<glm::Vec3>,
	texBindGroupName: Option<String>,
	sampler: wgpu::Sampler,
	texBindGroupLayout: wgpu::BindGroupLayout,
	texBindGroup: wgpu::BindGroup,
	pipeline: wgpu::RenderPipeline
}
impl ViewportCompositor
{
	pub fn new (context: &Context, renderSetup: &RenderSetup, source: &hal::Texture, name: Option<&str>) -> Result<Self>
	{
		let name = name.map(String::from);

		let invGamma_all = 2.2;
		let invGamma = glm::Vec3::from_element(invGamma_all);
		let gammaUniform = hal::UniformGroup::create(
			context, wgpu::ShaderStages::FRAGMENT, util::concatIfSome(&name, "_gammaUniform").as_deref()
		);
		*gammaUniform.borrowData_mut() = invGamma.map(|c| 1./c);
		gammaUniform.upload(context);

		let shader = shader::Package::deserialize(
			util::sourceGeneratedBytes!("/shader/player/viewport.spk")
		)?.createShaderModuleFromBestInstance(
			context.device(), None, util::concatIfSome(&name, "_shaderModule").as_deref()
		).ok_or_else(|| {
			let msg = "Failed to create shader module";
			tracing::error!(msg);
			anyhow!(msg)
		})?;

		// ToDo: introduce a sampler library and put this there
		let sampler = context.device().create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Nearest,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		let texBindGroupLayout = context.device().create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				label: util::concatIfSome(&name, "_texBindGroupLayout").as_deref(),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
						count: None,
					},
				]
			}
		);
		let texBindGroupName = util::concatIfSome(&name, "_texBindGroup");
		let texBindGroup = context.device().create_bind_group(&wgpu::BindGroupDescriptor {
			label: texBindGroupName.as_deref(),
			layout: &texBindGroupLayout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&source.view()),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&sampler),
				}
			]
		});

		let pipelineLayout = context.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: util::concatIfSome(&name, "_pipelineLayout").as_deref(),
			bind_group_layouts: &[&texBindGroupLayout, &gammaUniform.bindGroupLayout],
			push_constant_ranges: &[],
		});

		let pipeline = context.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: util::concatIfSome(&name, "_pipeline").as_deref(),
			layout: Some(&pipelineLayout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: Some("vertexMain"),
				buffers: &[],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: Some("fragmentMain"),//Some("fs_non_premultiplied"),
				targets: &[Some(wgpu::ColorTargetState {
					format: renderSetup.surfaceFormat(),
					blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
					write_mask: wgpu::ColorWrites::ALL,
				})],
				compilation_options: wgpu::PipelineCompilationOptions::default(),
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleStrip,
				cull_mode: None,
				..Default::default()
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState::default(),
			multiview: None,
			cache: None,
		});

		Ok(Self { invGamma_all, invGamma, gammaUniform, texBindGroupName, sampler, texBindGroupLayout, texBindGroup, pipeline })
	}

	pub fn updateSource (&mut self, context: &Context, source: &hal::Texture)
	{
		self.texBindGroup = context.device().create_bind_group(&wgpu::BindGroupDescriptor {
			label: self.texBindGroupName.as_deref(),
			layout: &self.texBindGroupLayout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&source.view()),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&self.sampler),
				}
			]
		});
	}

	pub fn composit (&self, renderPass: &mut wgpu::RenderPass) {
		renderPass.set_pipeline(&self.pipeline);
		renderPass.set_bind_group(0, &self.texBindGroup, &[]);
		renderPass.set_bind_group(1, &self.gammaUniform.bindGroup, &[]);
		renderPass.draw(0..4, 0..1);
	}
}
