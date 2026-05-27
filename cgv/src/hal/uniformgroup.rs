
//////
//
// Imports
//

// Standard library
use std::ops::{Deref, DerefMut};

// WGPU API
use wgpu;

use bytemuck::NoUninit;

// Local imports
use crate::*;



//////
//
// Classes
//

#[derive(Debug)]
pub struct UniformGroup<UniformsStruct: Default+NoUninit> {
	data: UniformsStruct,
	buffer: wgpu::Buffer,
	pub bindGroupLayout: wgpu::BindGroupLayout,
	pub bindGroup: wgpu::BindGroup,
}
impl<UniformsStruct: Default+NoUninit> UniformGroup<UniformsStruct>
{
	pub fn create (context: &Context, visibility: wgpu::ShaderStages, name: Option<&str>) -> Self
	{
		// Create device objects
		// - buffer
		let buffer = context.device().create_buffer(&wgpu::BufferDescriptor {
			label: util::concatIfSome(&name, "_buffer").as_deref(),
			size: size_of::<UniformsStruct>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});
		// - bind group layout
		let bindGroupLayout = Self::createBindGroupLayout(context, visibility, name);
		// - bind group
		let bindGroup = context.device().create_bind_group(&wgpu::BindGroupDescriptor {
			label: util::concatIfSome(&name, "_bindGroup").as_deref(),
			layout: &bindGroupLayout,
			entries: &[wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }]
		});

		// Done!
		Self {data: Default::default(), buffer, bindGroupLayout, bindGroup}
	}

	/// [Create](Self::create) the uniform group and schedule an [upload](Self::upload) of the default values to the GPU
	/// before returning.
	#[inline]
	pub fn createAndUpload (context: &Context, visibility: wgpu::ShaderStages, name: Option<&str>) -> Self {
		let new = Self::create(context, visibility, name);
		new.upload(context);
		new
	}

	pub(crate) fn createBindGroupLayout (context: &Context, visibility: wgpu::ShaderStages, groupName: Option<&str>)
		-> wgpu::BindGroupLayout
	{
		context.device().create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				label: util::concatIfSome(&groupName, "_bindGroupLayout").as_deref(),
				entries: &[wgpu::BindGroupLayoutEntry {
					binding: 0, visibility,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				}]
			}
		)
	}

	#[inline(always)]
	pub fn borrowData (&self) -> &UniformsStruct {
		&self.data
	}

	#[inline(always)]
	pub fn borrowData_mut (&mut self) -> &mut UniformsStruct {
		&mut self.data
	}

	/// Perform the actions defined by the user closure on the uniform data and [upload](Self::upload) before returning.
	#[inline]
	pub fn update<R, Writer: FnOnce(&mut UniformsStruct)->R> (&mut self, context: &Context, writer: Writer) -> R {
		let retVal = writer(&mut self.data);
		self.upload(context);
		retVal
	}

	/// Perform the actions defined by the user closure on the uniform data and
	/// [upload immediately](Self::uploadImmediately) before returning.
	#[inline]
	pub fn updateImmediately<R, Writer: FnOnce(&mut UniformsStruct)->R> (
		&mut self, context: &Context, writer: Writer
	) -> R {
		let retVal = writer(&mut self.data);
		self.uploadImmediately(context);
		retVal
	}

	#[inline(always)]
	pub fn upload (&self, context: &Context) {
		context.queue().write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.data));
	}

	#[inline(always)]
	pub fn uploadImmediately (&self, context: &Context) {
		context.queue().write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.data));
		context.queue().submit([]);
	}
}
impl<UniformsStruct: Default+NoUninit> Deref for UniformGroup<UniformsStruct> {
	type Target = UniformsStruct;

	#[inline(always)]
	fn deref (&self) -> &Self::Target {
		&self.data
	}
}
impl<UniformsStruct: Default+NoUninit> DerefMut for UniformGroup<UniformsStruct> {
	#[inline(always)]
	fn deref_mut (&mut self) -> &mut Self::Target {
		&mut self.data
	}
}
