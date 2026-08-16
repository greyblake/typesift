use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DataEnum, DeriveInput, Error, Field, Fields, Index, Member, parse_macro_input,
    parse_quote,
};

/// Derives `typesift::TypeSift`: the value itself is offered first, then each field is visited in
/// declaration order.
///
/// Every type parameter gets a `TypeSift` bound. Types with lifetime parameters are rejected
/// because `TypeSift` requires `'static` types, and unions because the active field is unknown.
///
/// A field marked `#[typesift(skip)]` is not visited, so its type needs no `TypeSift` impl. The
/// attribute is only accepted on fields, and it does not remove the bound on type parameters.
#[proc_macro_derive(TypeSift, attributes(typesift))]
pub fn derive_type_sift(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand(mut input: DeriveInput) -> syn::Result<TokenStream> {
    reject_typesift_attributes(&input.attrs)?;

    if let Some(lifetime) = input.generics.lifetimes().next() {
        return Err(Error::new_spanned(
            lifetime,
            "`TypeSift` requires `'static` types, so it cannot be derived for types with lifetime parameters",
        ));
    }

    let body = match &input.data {
        Data::Struct(data) => visit_struct(&data.fields)?,
        Data::Enum(data) => visit_enum(data)?,
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

fn visit_struct(fields: &Fields) -> syn::Result<TokenStream> {
    let mut visits = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        if is_skipped(field)? {
            continue;
        }
        let member = match &field.ident {
            Some(ident) => Member::Named(ident.clone()),
            None => Member::Unnamed(Index::from(index)),
        };
        visits.push(visit_field(&quote!(&self.#member)));
    }

    Ok(quote! {
        #(#visits)*
        ::core::ops::ControlFlow::Continue(())
    })
}

fn visit_enum(data: &DataEnum) -> syn::Result<TokenStream> {
    if data.variants.is_empty() {
        return Ok(quote!(match *self {}));
    }

    let mut arms = Vec::new();
    for variant in &data.variants {
        reject_typesift_attributes(&variant.attrs)?;

        // Skipped fields are matched with `_`, so they get neither a binding nor a visit.
        let mut patterns = Vec::new();
        let mut visits = Vec::new();
        for (index, field) in variant.fields.iter().enumerate() {
            if is_skipped(field)? {
                patterns.push(quote!(_));
            } else {
                let binding = format_ident!("__field{}", index);
                visits.push(visit_field(&quote!(#binding)));
                patterns.push(quote!(#binding));
            }
        }

        let variant_name = &variant.ident;
        let pattern = match &variant.fields {
            Fields::Named(fields) => {
                let names = fields.named.iter().map(|field| &field.ident);
                quote!(Self::#variant_name { #(#names: #patterns),* })
            }
            Fields::Unnamed(_) => quote!(Self::#variant_name(#(#patterns),*)),
            Fields::Unit => quote!(Self::#variant_name),
        };

        arms.push(quote! {
            #pattern => {
                #(#visits)*
                ::core::ops::ControlFlow::Continue(())
            }
        });
    }

    Ok(quote! {
        match self {
            #(#arms)*
        }
    })
}

/// `field` must evaluate to a reference that lives as long as `self`.
fn visit_field(field: &TokenStream) -> TokenStream {
    quote! {
        ::typesift::TypeSift::visit::<__T, __B, __F>(#field, __visitor)?;
    }
}

/// Whether `field` is marked `#[typesift(skip)]`. Any other `typesift` argument is an error.
fn is_skipped(field: &Field) -> syn::Result<bool> {
    let mut skip = false;
    for attr in typesift_attributes(&field.attrs) {
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("skip") {
                return Err(meta.error("unknown `typesift` argument, expected `skip`"));
            }
            if skip {
                return Err(meta.error("duplicate `skip`"));
            }
            skip = true;
            Ok(())
        })?;
    }
    Ok(skip)
}

/// `typesift` attributes only have a meaning on fields, so anywhere else they are an error.
fn reject_typesift_attributes(attrs: &[Attribute]) -> syn::Result<()> {
    match typesift_attributes(attrs).next() {
        Some(attr) => Err(Error::new_spanned(
            attr,
            "`#[typesift(...)]` is only allowed on fields",
        )),
        None => Ok(()),
    }
}

fn typesift_attributes(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> {
    attrs.iter().filter(|attr| attr.path().is_ident("typesift"))
}

#[cfg(test)]
mod tests {
    use syn::{DeriveInput, parse_quote};

    use super::expand;

    fn error_message(input: DeriveInput) -> String {
        expand(input)
            .expect_err("the derive should fail")
            .to_string()
    }

    #[test]
    fn typesift_attribute_is_rejected_outside_fields() {
        let on_struct = parse_quote! {
            #[typesift(skip)]
            struct Session {
                user: u64,
            }
        };
        assert_eq!(
            error_message(on_struct),
            "`#[typesift(...)]` is only allowed on fields"
        );

        let on_variant = parse_quote! {
            enum Event {
                #[typesift(skip)]
                Login(u64),
            }
        };
        assert_eq!(
            error_message(on_variant),
            "`#[typesift(...)]` is only allowed on fields"
        );
    }

    #[test]
    fn skip_is_the_only_argument_and_appears_once() {
        let unknown = parse_quote! {
            struct Session {
                #[typesift(rename)]
                user: u64,
            }
        };
        assert_eq!(
            error_message(unknown),
            "unknown `typesift` argument, expected `skip`"
        );

        let duplicate = parse_quote! {
            struct Session {
                #[typesift(skip, skip)]
                user: u64,
            }
        };
        assert_eq!(error_message(duplicate), "duplicate `skip`");

        let duplicate_attributes = parse_quote! {
            enum Event {
                Login(#[typesift(skip)] #[typesift(skip)] u64),
            }
        };
        assert_eq!(error_message(duplicate_attributes), "duplicate `skip`");

        let without_arguments = parse_quote! {
            struct Session {
                #[typesift]
                user: u64,
            }
        };
        assert!(error_message(without_arguments).contains("expected attribute arguments"));
    }

    #[test]
    fn skipped_fields_are_left_out_of_the_generated_code() {
        let input = parse_quote! {
            struct Session {
                user: u64,
                #[typesift(skip)]
                cache: Cache,
            }
        };
        let generated = expand(input).expect("the derive succeeds").to_string();
        assert!(generated.contains("user"));
        assert!(!generated.contains("cache"));

        let input = parse_quote! {
            enum Event {
                Login { user: u64, #[typesift(skip)] token: Token },
            }
        };
        let generated = expand(input).expect("the derive succeeds").to_string();
        assert!(generated.contains("token : _"));
    }
}
