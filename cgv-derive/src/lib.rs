
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
		impl #implGenerics ::cgv::renderer::data::InterleavedElem
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
		impl #implGenerics ::cgv::renderer::data::_ElemNormalBase
			for #name #tyGenerics #whereClause
		{
			type _Iterator<'data> = ::cgv::renderer::data::StridedCopyIter<'data, glm::Vec3> where Self: 'data;
			#[inline(always)] fn _available () -> bool { true }
			#[inline(always)] fn _iter (&self, len: usize) -> Self::_Iterator<'_> { unsafe {
				// SAFETY: We will be originating from an array of structs, thus `len` will be known and can be trusted,
				// and elements will have been placed with appropriate alignment, so the validity of the fields that the
				// iterator accesses is guaranteed.
				::cgv::renderer::data::StridedCopyIter::new(#body, size_of::<Self>(), len)
			}}
			#[inline(always)] fn normal (&self) -> &::cgv::glm::Vec3 { #body }
		}
		impl #implGenerics ::cgv::renderer::data::ElemWithNormal
			for #name #tyGenerics #whereClause
		{}
	}
	.into()
}

/// Mark the element struct as not including a normal, causing the
/// [`host::CanHaveNormals`](cgv::renderer::data::host::CanHaveNormals) blanket implementation for slices over it to
/// report their absence and panic on access.
#[proc_macro_derive(NoNormal)]
pub fn deriveNoNormal(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();
	quote! {
		impl #implGenerics ::cgv::renderer::data::_ElemNormalBase
			for #name #tyGenerics #whereClause
		{
			type _Iterator<'data> = std::iter::Empty<glm::Vec3>;
			#[inline(always)] fn _available () -> bool { false }
			#[inline(always)] fn _iter (&self, len: usize) ->  Self::_Iterator<'_> {
				panic!("no normals available")
			}
			#[inline(always)] fn normal (&self) -> &::cgv::glm::Vec3 { panic!("no normal available") }
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
		impl #implGenerics ::cgv::renderer::data::_ElemTangentBase
			for #name #tyGenerics #whereClause
		{
			type _Iterator<'data> = ::cgv::renderer::data::StridedCopyIter<'data, glm::Vec3> where Self: 'data;
			#[inline(always)] fn _available () -> bool { true }
			#[inline(always)] fn _iter (&self, len: usize) -> Self::_Iterator<'_> { unsafe {
				// SAFETY: We will be originating from an array of structs, thus `len` will be known and can be trusted,
				// and elements will have been placed with appropriate alignment, so the validity of the fields that the
				// iterator accesses is guaranteed.
				::cgv::renderer::data::StridedCopyIter::new(#body, size_of::<Self>(), len)
			}}
			#[inline(always)] fn tangent (&self) -> &::cgv::glm::Vec3 { #body }
		}
		impl #implGenerics ::cgv::renderer::data::ElemWithTangent
			for #name #tyGenerics #whereClause
		{}
	}
	.into()
}

/// Mark the element struct as not including a tangent, causing the
/// [`host::CanHaveTangents`](cgv::renderer::data::host::CanHaveTangents) blanket implementation for slices over it to
/// report their absence and panic on access.
#[proc_macro_derive(NoTangent)]
pub fn deriveNoTangent(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();
	quote! {
		impl #implGenerics ::cgv::renderer::data::_ElemTangentBase
			for #name #tyGenerics #whereClause
		{
			type _Iterator<'data> = std::iter::Empty<glm::Vec3>;
			#[inline(always)] fn _available () -> bool { false }
			#[inline(always)] fn _iter (&self, len: usize) ->  Self::_Iterator<'_> {
				panic!("no tangents available")
			}
			#[inline(always)] fn tangent (&self) -> &::cgv::glm::Vec3 { panic!("no tangent available") }
		}
	}
	.into()
}

/// Derive [`cgv::renderer::ElemWithRadius`] for a struct.
///
/// Mark exactly one field with `#[cgv_renderAttr(radius)]`.
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

	let body = ref_body!(fieldIdent);
	quote! {
		impl #implGenerics ::cgv::renderer::data::_ElemRadiusBase
			for #name #tyGenerics #whereClause
		{
			type _Iterator<'data> = ::cgv::renderer::data::StridedCopyIter<'data, f32> where Self: 'data;
			#[inline(always)] fn _available () -> bool { true }
			#[inline(always)] fn _iter (&self, len: usize) -> Self::_Iterator<'_> { unsafe {
				::cgv::renderer::data::StridedCopyIter::new(#body, size_of::<Self>(), len)
			}}
			#[inline(always)] fn radius (&self) -> &f32 { #body }
		}
		impl #implGenerics ::cgv::renderer::data::ElemWithRadius
			for #name #tyGenerics #whereClause
		{}
	}
	.into()
}

/// Mark the element struct as not including a radius, causing the
/// [`host::CanHaveRadii`](cgv::renderer::data::host::CanHaveRadii) blanket implementation for slices over it to report
/// their absence and panic on access.
#[proc_macro_derive(NoRadius)]
pub fn deriveNoRadius(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();
	quote! {
		impl #implGenerics ::cgv::renderer::data::_ElemRadiusBase
			for #name #tyGenerics #whereClause
		{
			type _Iterator<'data> = std::iter::Empty<f32>;
			#[inline(always)] fn _available () -> bool { false }
			#[inline(always)] fn _iter (&self, len: usize) -> Self::_Iterator<'_> {
				panic!("no radii available")
			}
			#[inline(always)] fn radius (&self) -> &f32 { panic!("no radius available") }
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

	let body = ref_body!(fieldIdent);
	quote! {
		impl #implGenerics ::cgv::renderer::data::_ElemRadiusDerivBase
			for #name #tyGenerics #whereClause
		{
			type _Iterator<'data> = ::cgv::renderer::data::StridedCopyIter<'data, f32> where Self: 'data;
			#[inline(always)] fn _available () -> bool { true }
			#[inline(always)] fn _iter (&self, len: usize) -> Self::_Iterator<'_> { unsafe {
				::cgv::renderer::data::StridedCopyIter::new(#body, size_of::<Self>(), len)
			}}
			#[inline(always)] fn radiusDeriv (&self) -> &f32 { #body }
		}
		impl #implGenerics ::cgv::renderer::data::ElemWithRadiusDeriv
			for #name #tyGenerics #whereClause
		{}
	}
	.into()
}

/// Mark the element struct as not including a radius derivative, causing the
/// [`host::CanHaveRadiusDerivs`](cgv::renderer::data::host::CanHaveRadiusDerivs) blanket implementation for slices over
/// it to report their absence and panic on access.
#[proc_macro_derive(NoRadiusDeriv)]
pub fn deriveNoRadiusDeriv(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();
	quote! {
		impl #implGenerics ::cgv::renderer::data::_ElemRadiusDerivBase
			for #name #tyGenerics #whereClause
		{
			type _Iterator<'data> = std::iter::Empty<f32>;
			#[inline(always)] fn _available () -> bool { false }
			#[inline(always)] fn _iter (&self, len: usize) -> Self::_Iterator<'_> {
				panic!("no radius derivatives available")
			}
			#[inline(always)] fn radiusDeriv (&self) -> &f32 { panic!("no radius derivative available") }
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
		impl #implGenerics ::cgv::renderer::data::_ElemOrientationBase
			for #name #tyGenerics #whereClause
		{
			type _Iterator<'data> = ::cgv::renderer::data::StridedCopyIter<'data, glm::Quat> where Self: 'data;
			#[inline(always)] fn _available () -> bool { true }
			#[inline(always)] fn _iter (&self, len: usize) -> Self::_Iterator<'_> { unsafe {
				::cgv::renderer::data::StridedCopyIter::new(#body, size_of::<Self>(), len)
			}}
			#[inline(always)] fn orientation (&self) -> &::cgv::glm::Quat { #body }
		}
		impl #implGenerics ::cgv::renderer::data::ElemWithOrientation
			for #name #tyGenerics #whereClause
		{}
	}
	.into()
}

/// Mark the element struct as not including an orientation, causing the
/// [`host::CanHaveOrientations`](cgv::renderer::data::host::CanHaveOrientations) blanket implementation for slices over
/// it to report their absence and panic on access.
#[proc_macro_derive(NoOrientation)]
pub fn deriveNoOrientation(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();
	quote! {
		impl #implGenerics ::cgv::renderer::data::_ElemOrientationBase
			for #name #tyGenerics #whereClause
		{
			type _Iterator<'data> = std::iter::Empty<glm::Quat>;
			#[inline(always)] fn _available () -> bool { false }
			#[inline(always)] fn _iter (&self, len: usize) -> Self::_Iterator<'_> {
				panic!("no orientations available")
			}
			#[inline(always)] fn orientation (&self) -> &::cgv::glm::Quat { panic!("no orientation available") }
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
		impl #implGenerics ::cgv::renderer::data::_ElemScalingBase
			for #name #tyGenerics #whereClause
		{
			type _Iterator<'data> = ::cgv::renderer::data::StridedCopyIter<'data, glm::Vec3> where Self: 'data;
			#[inline(always)] fn _available () -> bool { true }
			#[inline(always)] fn _iter (&self, len: usize) -> Self::_Iterator<'_> { unsafe {
				::cgv::renderer::data::StridedCopyIter::new(#body, size_of::<Self>(), len)
			}}
			#[inline(always)] fn scaling (&self) -> &::cgv::glm::Vec3 { #body }
		}
		impl #implGenerics ::cgv::renderer::data::ElemWithScaling
			for #name #tyGenerics #whereClause
		{}
	}
	.into()
}

/// Mark the element struct as not including a scaling, causing the
/// [`host::CanHaveScalings`](cgv::renderer::data::host::CanHaveScalings) blanket implementation for slices over it to
/// report their absence and panic on access.
#[proc_macro_derive(NoScaling)]
pub fn deriveNoScaling(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();
	quote! {
		impl #implGenerics ::cgv::renderer::data::_ElemScalingBase
			for #name #tyGenerics #whereClause
		{
			type _Iterator<'data> = std::iter::Empty<glm::Vec3>;
			#[inline(always)] fn _available () -> bool { false }
			#[inline(always)] fn _iter (&self, len: usize) -> Self::_Iterator<'_> {
				panic!("no scalings available")
			}
			#[inline(always)] fn scaling (&self) -> &::cgv::glm::Vec3 { panic!("no scaling available") }
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
		impl #implGenerics ::cgv::renderer::data::_ElemColorBase
			for #name #tyGenerics #whereClause
		{
			type _Iterator<'data> = ::cgv::renderer::data::StridedCopyIter<'data, ::cgv::RGBA> where Self: 'data;
			#[inline(always)] fn _available () -> bool { true }
			#[inline(always)] fn _iter (&self, len: usize) -> Self::_Iterator<'_> { unsafe {
				::cgv::renderer::data::StridedCopyIter::new(#body, size_of::<Self>(), len)
			}}
			#[inline(always)] fn color (&self) -> &::cgv::RGBA { #body }
		}
		impl #implGenerics ::cgv::renderer::data::ElemWithColor
			for #name #tyGenerics #whereClause
		{}
	}
	.into()
}

/// Mark the element struct as not including color, causing the
/// [`host::CanHaveColors`](cgv::renderer::data::host::CanHaveColors) blanket implementation for slices over it to
/// report their absence and panic on access.
#[proc_macro_derive(NoColor)]
pub fn deriveNoColor(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();
	quote! {
		impl #implGenerics ::cgv::renderer::data::_ElemColorBase
			for #name #tyGenerics #whereClause
		{
			type _Iterator<'data> = std::iter::Empty<::cgv::RGBA>;
			#[inline(always)] fn _available () -> bool { false }
			#[inline(always)] fn _iter (&self, len: usize) -> Self::_Iterator<'_> {
				panic!("no color available")
			}
			#[inline(always)] fn color (&self) -> &::cgv::RGBA { panic!("no color available") }
		}
	}
	.into()
}

/// Derive a "no normals" impl of [`cgv::renderer::data::host::CanHaveNormals`].
///
/// `hasNormals()` will return `false`; the other methods will panic if invoked.
#[proc_macro_derive(NoNormals)]
pub fn deriveNoNormals(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();
	quote! {
		impl #implGenerics ::cgv::renderer::data::host::CanHaveNormals
			for #name #tyGenerics #whereClause
		{
			type NormalIterator<'data> = ::std::iter::Empty<&'data ::cgv::glm::Vec3> where Self: 'data;
			fn hasNormals (&self) -> bool { false }
			fn normals (&self) -> Self::NormalIterator<'_> { panic!("no normals available") }
			fn normal (&self, _index: u32) -> &::cgv::glm::Vec3 { panic!("no normals available") }
		}
	}
	.into()
}

/// Derive a "no tangents" impl of [`cgv::renderer::data::host::CanHaveTangents`].
///
/// `hasTangents()` will return `false`; the other methods will panic if invoked.
#[proc_macro_derive(NoTangents)]
pub fn deriveNoTangents(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();
	quote! {
		impl #implGenerics ::cgv::renderer::data::host::CanHaveTangents
			for #name #tyGenerics #whereClause
		{
			type TangentIterator<'data> = ::std::iter::Empty<&'data ::cgv::glm::Vec3> where Self: 'data;
			fn hasTangents (&self) -> bool { false }
			fn tangents (&self) -> Self::TangentIterator<'_> { panic!("no tangents available") }
			fn tangent (&self, _index: u32) -> &::cgv::glm::Vec3 { panic!("no tangents available") }
		}
	}
	.into()
}

/// Derive a "no radii" impl of [`cgv::renderer::data::host::CanHaveRadii`].
///
/// `hasRadii()` will return `false`; the other methods will panic if invoked.
#[proc_macro_derive(NoRadii)]
pub fn deriveNoRadii(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();
	quote! {
		impl #implGenerics ::cgv::renderer::data::host::CanHaveRadii
			for #name #tyGenerics #whereClause
		{
			type RadiusIterator<'data> = ::std::iter::Empty<&'data f32> where Self: 'data;
			fn hasRadii (&self) -> bool { false }
			fn radii (&self) -> Self::RadiusIterator<'_> { panic!("no radii available") }
			fn radius (&self, _index: u32) -> f32 { panic!("no radii available") }
		}
	}
	.into()
}

/// Derive a "no radius derivatives" impl of [`cgv::renderer::data::host::CanHaveRadiusDerivs`].
///
/// `hasRadiusDerivs()` will return `false`; the other methods will panic if invoked.
#[proc_macro_derive(NoRadiusDerivs)]
pub fn deriveNoRadiusDerivs(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();
	quote! {
		impl #implGenerics ::cgv::renderer::data::host::CanHaveRadiusDerivs
			for #name #tyGenerics #whereClause
		{
			type RadiusDerivIterator<'data> = ::std::iter::Empty<&'data f32> where Self: 'data;
			fn hasRadiusDerivs (&self) -> bool { false }
			fn radiusDerivs (&self) -> Self::RadiusDerivIterator<'_> { panic!("no radius derivatives available") }
			fn radiusDeriv (&self, _index: u32) -> f32 { panic!("no radius derivatives available") }
		}
	}
	.into()
}

/// Derive a "no orientations" impl of [`cgv::renderer::data::host::CanHaveOrientations`].
///
/// `hasOrientations()` will return `false`; the other methods will panic if invoked.
#[proc_macro_derive(NoOrientations)]
pub fn deriveNoOrientations(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();
	quote! {
		impl #implGenerics ::cgv::renderer::data::host::CanHaveOrientations
			for #name #tyGenerics #whereClause
		{
			type OrientationIterator<'data> = ::std::iter::Empty<&'data ::cgv::glm::Quat> where Self: 'data;
			fn hasOrientations (&self) -> bool { false }
			fn orientations (&self) -> Self::OrientationIterator<'_> { panic!("no orientations available") }
			fn orientation (&self, _index: u32) -> &::cgv::glm::Quat { panic!("no orientations available") }
		}
	}
	.into()
}

/// Derive a "no scalings" impl of [`cgv::renderer::data::host::CanHaveScalings`].
///
/// `hasScalings()` will return `false`; the other methods will panic if invoked.
#[proc_macro_derive(NoScalings)]
pub fn deriveNoScalings(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();
	quote! {
		impl #implGenerics ::cgv::renderer::data::host::CanHaveScalings
			for #name #tyGenerics #whereClause
		{
			type ScaleIterator<'data> = ::std::iter::Empty<&'data ::cgv::glm::Vec3> where Self: 'data;
			fn hasScalings (&self) -> bool { false }
			fn scalings (&self) -> Self::ScaleIterator<'_> { panic!("no scalings available") }
			fn scaling (&self, _index: u32) -> &::cgv::glm::Vec3 { panic!("no scalings available") }
		}
	}
	.into()
}

/// Derive a "no colors" impl of [`cgv::renderer::data::host::CanHaveColors`].
///
/// `hasColors()` will return `false`; the other methods will panic if invoked.
#[proc_macro_derive(NoColors)]
pub fn deriveNoColors(input: TokenStream) -> TokenStream
{
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (implGenerics, tyGenerics, whereClause) = input.generics.split_for_impl();
	quote! {
		impl #implGenerics ::cgv::renderer::data::host::CanHaveColors
			for #name #tyGenerics #whereClause
		{
			type ColorIterator<'data> = ::std::iter::Empty<&'data ::cgv::RGBA> where Self: 'data;
			fn hasColors (&self) -> bool { false }
			fn colors (&self) -> Self::ColorIterator<'_> { panic!("no colors available") }
			fn color (&self, _index: u32) -> &::cgv::RGBA { panic!("no colors available") }
		}
	}
	.into()
}
