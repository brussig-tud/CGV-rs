
//////
//
// Language config
//

// Eff this convention.
#![allow(non_snake_case)]



//////
//
// Imports
//

// Standard library
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields};



//////
//
// Macros
//

/// Given a field ident, generate `&self.<field>`.
macro_rules! ref_body {
	($field:ident) => { quote! { &self.#$field } };
}

/// Given a field ident, generate `self.<field>` (value copy, for `f32` etc.).
macro_rules! copy_body {
	($field:ident) => { quote! { self.#$field } };
}



//////
//
// Functions
//

/// Find the first named field annotated with `#[cgv_renderAttr(<attr_name>)]` and return a reference to both the field
/// and its identifier.
fn findField_renderAttr<'a> (fields: &'a syn::FieldsNamed, attrName: &str) -> Option<(&'a Field, &'a syn::Ident)>
{
	for field in &fields.named
	{
		for attr in &field.attrs
		{
			if !attr.path().is_ident("cgv_renderAttr") {
				continue;
			}
			// The attribute must look like `#[cgv_renderAttr(some_ident)]`
			if let Ok(ident) = attr.parse_args::<syn::Ident>() {
				if ident == attrName {
					return Some((field, field.ident.as_ref().unwrap()));
				}
			}
		}
	}
	None
}

/// Extract the named fields from a struct `DeriveInput`, or return a compile-error token stream.
fn getNamedFields (input: &DeriveInput) -> Result<&syn::FieldsNamed, TokenStream2>
{
	match &input.data
	{
		Data::Struct(s) => match &s.fields {
			Fields::Named(fields) => Ok(fields),
			_ => Err(quote! {
				compile_error!("cgv derive macros only support structs with named fields");
			}),
		},
		_ => Err(quote! {
			compile_error!("cgv derive macros only support structs");
		}),
	}
}



//////
//
// Procedural macros
//

/// Derive [`cgv::renderer::InterleavedElem`] for a struct.
///
/// Mark exactly one field with `#[cgv_renderAttr(pos)]`; that field will be returned by the generated `pos()` method
/// (as a `&glm::Vec3`).
#[proc_macro_derive(InterleavedElem, attributes(cgv_renderAttr))]
pub fn deriveInterleavedElem (input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (
		implGenerics, tyGenerics, whereClause
	) = input.generics.split_for_impl();

	let fields = match getNamedFields(&input) {
		Ok(f) => f,
		Err(e) => return e.into(),
	};
	let fieldIdent = match findField_renderAttr(fields, "pos") {
		Some((_, ident)) => ident,
		None => {
			return quote! {
				compile_error!(
					"`#[derive(InterleavedElem)]` requires exactly one field annotated \
					with `#[cgv_renderAttr(pos)]`"
				);
			}
			.into()
		}
	};

	let body = ref_body!(fieldIdent);
	quote! {
		impl #implGenerics ::cgv::renderer::InterleavedElem
			for #name #tyGenerics #whereClause
		{
			fn pos(&self) -> &::cgv::glm::Vec3 { #body }
		}
	}
	.into()
}

/// Derive [`cgv::renderer::ElemWithNormal`] for a struct.
///
/// Mark exactly one field with `#[cgv_renderAttr(normal)]`.
#[proc_macro_derive(ElemWithNormal, attributes(cgv_renderAttr))]
pub fn deriveElemWithNormal(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();

	let fields = match getNamedFields(&input) {
		Ok(f) => f,
		Err(e) => return e.into(),
	};
	let fieldIdent = match findField_renderAttr(fields, "normal") {
		Some((_, ident)) => ident,
		None => {
			return quote! {
				compile_error!(
					"`#[derive(ElemWithNormal)]` requires exactly one field annotated \
					with `#[cgv_renderAttr(normal)]`"
				);
			}
			.into()
		}
	};

	let body = ref_body!(fieldIdent);
	quote! {
		impl #implGenerics ::cgv::renderer::ElemWithNormal
			for #name #tyGenerics #whereClause
		{
			fn normal(&self) -> &::cgv::glm::Vec3 { #body }
		}
	}
	.into()
}

/// Derive [`cgv::renderer::ElemWithTangent`] for a struct.
///
/// Mark exactly one field with `#[cgv_renderAttr(tangent)]`.
#[proc_macro_derive(ElemWithTangent, attributes(cgv_renderAttr))]
pub fn deriveElemWithTangent(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();

	let fields = match getNamedFields(&input) {
		Ok(f) => f,
		Err(e) => return e.into(),
	};
	let fieldIdent = match findField_renderAttr(fields, "tangent") {
		Some((_, ident)) => ident,
		None => {
			return quote! {
				compile_error!(
					"`#[derive(ElemWithTangent)]` requires exactly one field annotated \
					with `#[cgv_renderAttr(tangent)]`"
				);
			}
			.into()
		}
	};

	let body = ref_body!(fieldIdent);
	quote! {
		impl #implGenerics ::cgv::renderer::ElemWithTangent
			for #name #tyGenerics #whereClause
		{
			fn tangent(&self) -> &::cgv::glm::Vec3 { #body }
		}
	}
	.into()
}

/// Derive [`cgv::renderer::ElemWithRadius`] for a struct.
///
/// Mark exactly one field with `#[cgv_renderAttr(radius)]`.  The field must be `f32` (or
/// any `Copy` type that satisfies the trait's return type – the compiler will
/// check this).
#[proc_macro_derive(ElemWithRadius, attributes(cgv_renderAttr))]
pub fn deriveElemWithRadius(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();

	let fields = match getNamedFields(&input) {
		Ok(f) => f,
		Err(e) => return e.into(),
	};
	let fieldIdent = match findField_renderAttr(fields, "radius") {
		Some((_, ident)) => ident,
		None => {
			return quote! {
				compile_error!(
					"`#[derive(ElemWithRadius)]` requires exactly one field annotated \
					with `#[cgv_renderAttr(radius)]`"
				);
			}
			.into()
		}
	};

	let body = copy_body!(fieldIdent);
	quote! {
		impl #implGenerics ::cgv::renderer::ElemWithRadius
			for #name #tyGenerics #whereClause
		{
			fn radius(&self) -> &f32 { &#body }
		}
	}
	.into()
}

/// Derive [`cgv::renderer::ElemWithRadiusDeriv`] for a struct.
///
/// Mark exactly one field with `#[cgv_renderAttr(radiusDeriv)]`.
#[proc_macro_derive(ElemWithRadiusDeriv, attributes(cgv_renderAttr))]
pub fn deriveElemWithRadiusDeriv(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();

	let fields = match getNamedFields(&input) {
		Ok(f) => f,
		Err(e) => return e.into(),
	};
	let fieldIdent = match findField_renderAttr(fields, "radiusDeriv") {
		Some((_, ident)) => ident,
		None => {
			return quote! {
				compile_error!(
					"`#[derive(ElemWithRadiusDeriv)]` requires exactly one field annotated \
					with `#[cgv_renderAttr(radiusDeriv)]`"
				);
			}
			.into()
		}
	};

	let body = copy_body!(fieldIdent);
	quote! {
		impl #implGenerics ::cgv::renderer::ElemWithRadiusDeriv
			for #name #tyGenerics #whereClause
		{
			fn radiusDeriv(&self) -> &f32 { &#body }
		}
	}
	.into()
}

/// Derive [`cgv::renderer::ElemWithOrientation`] for a struct.
///
/// Mark exactly one field with `#[cgv_renderAttr(orientation)]`.
#[proc_macro_derive(ElemWithOrientation, attributes(cgv_renderAttr))]
pub fn deriveElemWithOrientation(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();

	let fields = match getNamedFields(&input) {
		Ok(f) => f,
		Err(e) => return e.into(),
	};
	let fieldIdent = match findField_renderAttr(fields, "orientation") {
		Some((_, ident)) => ident,
		None => {
			return quote! {
				compile_error!(
					"`#[derive(ElemWithOrientation)]` requires exactly one field annotated \
					with `#[cgv_renderAttr(orientation)]`"
				);
			}
			.into()
		}
	};

	let body = ref_body!(fieldIdent);
	quote! {
		impl #implGenerics ::cgv::renderer::ElemWithOrientation
			for #name #tyGenerics #whereClause
		{
			fn orientation(&self) -> &::cgv::glm::Quat { #body }
		}
	}
	.into()
}

/// Derive [`cgv::renderer::ElemWithScaling`] for a struct.
///
/// Mark exactly one field with `#[cgv_renderAttr(scaling)]`.
#[proc_macro_derive(ElemWithScaling, attributes(cgv_renderAttr))]
pub fn deriveElemWithScaling(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();

	let fields = match getNamedFields(&input) {
		Ok(f) => f,
		Err(e) => return e.into(),
	};
	let fieldIdent = match findField_renderAttr(fields, "scaling") {
		Some((_, ident)) => ident,
		None => {
			return quote! {
				compile_error!(
					"`#[derive(ElemWithScaling)]` requires exactly one field annotated \
					with `#[cgv_renderAttr(scaling)]`"
				);
			}
			.into()
		}
	};

	let body = ref_body!(fieldIdent);
	quote! {
		impl #implGenerics ::cgv::renderer::ElemWithScaling
			for #name #tyGenerics #whereClause
		{
			fn scaling(&self) -> &::cgv::glm::Vec3 { #body }
		}
	}
	.into()
}

/// Derive [`cgv::renderer::ElemWithColor`] for a struct.
///
/// Mark exactly one field with `#[cgv_renderAttr(color)]`.
#[proc_macro_derive(ElemWithColor, attributes(cgv_renderAttr))]
pub fn deriveElemWithColor(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();

	let fields = match getNamedFields(&input) {
		Ok(f) => f,
		Err(e) => return e.into(),
	};
	let fieldIdent = match findField_renderAttr(fields, "color") {
		Some((_, ident)) => ident,
		None => {
			return quote! {
				compile_error!(
					"`#[derive(ElemWithColor)]` requires exactly one field annotated \
					with `#[cgv_renderAttr(color)]`"
				);
			}
			.into()
		}
	};

	let body = ref_body!(fieldIdent);
	quote! {
		impl #implGenerics ::cgv::renderer::ElemWithColor
			for #name #tyGenerics #whereClause
		{
			fn color(&self) -> &::cgv::RGBA { #body }
		}
	}
	.into()
}
