use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Data, DataEnum, DeriveInput, Error, Fields, Index, Member, parse_macro_input, parse_quote,
};

/// Derives `typesift::TypeSift`: the value itself is offered first, then each field is visited in
/// declaration order.
///
/// Every type parameter gets a `TypeSift` bound. Types with lifetime parameters are rejected
/// because `TypeSift` requires `'static` types, and unions because the active field is unknown.
#[proc_macro_derive(TypeSift)]
pub fn derive_type_sift(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand(mut input: DeriveInput) -> syn::Result<TokenStream> {
    if let Some(lifetime) = input.generics.lifetimes().next() {
        return Err(Error::new_spanned(
            lifetime,
            "`TypeSift` requires `'static` types, so it cannot be derived for types with lifetime parameters",
        ));
    }

    let body = match &input.data {
        Data::Struct(data) => visit_struct(&data.fields),
        Data::Enum(data) => visit_enum(data),
        Data::Union(data) => {
            return Err(Error::new_spanned(
                data.union_token,
                "`TypeSift` cannot be derived for unions",
            ));
        }
    };

    for param in input.generics.type_params_mut() {
        param.bounds.push(parse_quote!(::typesift::TypeSift));
    }

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Generic names are prefixed with `__` so they cannot clash with the type's own parameters.
    Ok(quote! {
        impl #impl_generics ::typesift::TypeSift for #name #ty_generics #where_clause {
            fn visit<'__a, __T: 'static, __B, __F>(
                &'__a self,
                __visitor: &mut __F,
            ) -> ::core::ops::ControlFlow<__B>
            where
                __F: ::core::ops::FnMut(&'__a __T) -> ::core::ops::ControlFlow<__B>,
            {
                ::typesift::visit_self::<Self, __T, __B, __F>(self, __visitor)?;
                #body
            }
        }
    })
}

fn visit_struct(fields: &Fields) -> TokenStream {
    let visits = fields.iter().enumerate().map(|(index, field)| {
        let member = match &field.ident {
            Some(ident) => Member::Named(ident.clone()),
            None => Member::Unnamed(Index::from(index)),
        };
        visit_field(&quote!(&self.#member))
    });

    quote! {
        #(#visits)*
        ::core::ops::ControlFlow::Continue(())
    }
}

fn visit_enum(data: &DataEnum) -> TokenStream {
    if data.variants.is_empty() {
        return quote!(match *self {});
    }

    let arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let bindings: Vec<_> = (0..variant.fields.len())
            .map(|index| format_ident!("__field{}", index))
            .collect();
        let pattern = match &variant.fields {
            Fields::Named(fields) => {
                let names = fields.named.iter().map(|field| &field.ident);
                quote!(Self::#variant_name { #(#names: #bindings),* })
            }
            Fields::Unnamed(_) => quote!(Self::#variant_name(#(#bindings),*)),
            Fields::Unit => quote!(Self::#variant_name),
        };
        let visits = bindings
            .iter()
            .map(|binding| visit_field(&quote!(#binding)));

        quote! {
            #pattern => {
                #(#visits)*
                ::core::ops::ControlFlow::Continue(())
            }
        }
    });

    quote! {
        match self {
            #(#arms)*
        }
    }
}

/// `field` must evaluate to a reference that lives as long as `self`.
fn visit_field(field: &TokenStream) -> TokenStream {
    quote! {
        ::typesift::TypeSift::visit::<__T, __B, __F>(#field, __visitor)?;
    }
}
