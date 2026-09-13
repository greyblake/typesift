use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DataEnum, DeriveInput, Error, Field, Fields, Index, Member, Path,
    parse_macro_input, parse_quote,
};

/// Derives `typesift::TypeSift`: the value itself is offered first, then each field is visited in
/// declaration order.
///
/// Every type parameter gets a `TypeSift` bound. Types with lifetime parameters are rejected
/// because `TypeSift` requires `'static` types, and unions because the active field is unknown.
///
/// Fields accept one argument, and none of them removes the bound on type parameters:
///
/// - `#[typesift(skip)]` leaves the field out of the traversal, so its type needs no impl.
/// - `#[typesift(leaf)]` offers the field itself but does not look inside it, so its type needs
///   only to be `'static`. This is how a type from another crate can still be found.
/// - `#[typesift(with = path)]` hands the field to a function generic over `typesift::Sifter`,
///   which decides what of it the search sees.
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
                ::typesift::TypeSift::visit_self::<__T, __B, __F>(self, __visitor)?;
                #body
            }
        }
    })
}

fn visit_struct(fields: &Fields) -> syn::Result<TokenStream> {
    let mut visits = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let mode = field_mode(field)?;
        if matches!(mode, FieldMode::Skip) {
            continue;
        }
        let member = match &field.ident {
            Some(ident) => Member::Named(ident.clone()),
            None => Member::Unnamed(Index::from(index)),
        };
        visits.push(mode.apply(&quote!(&self.#member)));
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
            let mode = field_mode(field)?;
            if matches!(mode, FieldMode::Skip) {
                patterns.push(quote!(_));
            } else {
                let binding = format_ident!("__field{}", index);
                visits.push(mode.apply(&quote!(#binding)));
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

/// What the derive does with one field.
enum FieldMode {
    /// Hand the field to its own `TypeSift` impl.
    Visit,
    /// Leave the field out entirely: `#[typesift(skip)]`.
    Skip,
    /// Offer the field without looking inside it: `#[typesift(leaf)]`.
    Leaf,
    /// Hand the field to a function of the user's: `#[typesift(with = path)]`.
    With(Path),
}

impl FieldMode {
    /// The argument that selected this mode, for error messages.
    fn name(&self) -> &'static str {
        match self {
            FieldMode::Visit => "",
            FieldMode::Skip => "skip",
            FieldMode::Leaf => "leaf",
            FieldMode::With(_) => "with",
        }
    }

    /// The code that visits one field. `field` must evaluate to a reference that lives as long as
    /// `self`.
    fn apply(&self, field: &TokenStream) -> TokenStream {
        match self {
            FieldMode::Visit => quote! {
                ::typesift::TypeSift::visit::<__T, __B, __F>(#field, __visitor)?;
            },
            // The field's type needs no impl here, only `'static`, so the type check that
            // `TypeSift::visit_self` performs is inlined instead of called.
            FieldMode::Leaf => quote! {
                if let ::core::option::Option::Some(__matched) =
                    (#field as &dyn ::core::any::Any).downcast_ref::<__T>()
                {
                    __visitor(__matched)?;
                }
            },
            // `__visitor` is reborrowed, so later fields can still use it.
            FieldMode::With(path) => quote! {
                #path(
                    #field,
                    &mut ::typesift::Sift::<__T, __B, __F>::new(&mut *__visitor),
                )?;
            },
            FieldMode::Skip => TokenStream::new(),
        }
    }
}

/// Reads the `typesift` attributes of one field.
fn field_mode(field: &Field) -> syn::Result<FieldMode> {
    let mut mode = FieldMode::Visit;
    for attr in typesift_attributes(&field.attrs) {
        attr.parse_nested_meta(|meta| {
            let found = if meta.path.is_ident("skip") {
                FieldMode::Skip
            } else if meta.path.is_ident("leaf") {
                FieldMode::Leaf
            } else if meta.path.is_ident("with") {
                FieldMode::With(meta.value()?.parse()?)
            } else {
                return Err(
                    meta.error("unknown `typesift` argument, expected `skip`, `leaf` or `with`")
                );
            };

            let conflict = match (&mode, &found) {
                (FieldMode::Visit, _) => None,
                (existing, found) if existing.name() == found.name() => {
                    Some(format!("duplicate `{}`", found.name()))
                }
                (existing, found) => Some(format!(
                    "`{}` and `{}` cannot be combined",
                    existing.name(),
                    found.name()
                )),
            };

            match conflict {
                Some(message) => Err(meta.error(message)),
                None => {
                    mode = found;
                    Ok(())
                }
            }
        })?;
    }
    Ok(mode)
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

    fn generated_code(input: DeriveInput) -> String {
        expand(input)
            .expect("the derive should succeed")
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
                #[typesift(leaf)]
                Login(u64),
            }
        };
        assert_eq!(
            error_message(on_variant),
            "`#[typesift(...)]` is only allowed on fields"
        );
    }

    #[test]
    fn arguments_are_known_and_appear_once() {
        let unknown = parse_quote! {
            struct Session {
                #[typesift(rename)]
                user: u64,
            }
        };
        assert_eq!(
            error_message(unknown),
            "unknown `typesift` argument, expected `skip`, `leaf` or `with`"
        );

        let duplicate_skip = parse_quote! {
            struct Session {
                #[typesift(skip, skip)]
                user: u64,
            }
        };
        assert_eq!(error_message(duplicate_skip), "duplicate `skip`");

        let duplicate_leaf = parse_quote! {
            struct Session {
                #[typesift(leaf)]
                #[typesift(leaf)]
                user: u64,
            }
        };
        assert_eq!(error_message(duplicate_leaf), "duplicate `leaf`");

        let combined = parse_quote! {
            enum Event {
                Login(#[typesift(skip, leaf)] u64),
            }
        };
        assert_eq!(
            error_message(combined),
            "`skip` and `leaf` cannot be combined"
        );

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
        let generated = generated_code(parse_quote! {
            struct Session {
                user: u64,
                #[typesift(skip)]
                cache: Cache,
            }
        });
        assert!(generated.contains("user"));
        assert!(!generated.contains("cache"));

        let generated = generated_code(parse_quote! {
            enum Event {
                Login { user: u64, #[typesift(skip)] token: Token },
            }
        });
        assert!(generated.contains("token : _"));
    }

    #[test]
    fn leaf_fields_are_checked_by_type_instead_of_visited() {
        let generated = generated_code(parse_quote! {
            struct Invoice {
                #[typesift(leaf)]
                id: Uuid,
                amount: u64,
            }
        });
        assert!(generated.contains("downcast_ref"));
        assert!(generated.contains("self . id"));
        // The field is offered, never walked into.
        assert!(!generated.contains("visit :: < __T , __B , __F > (& self . id"));
        assert!(generated.contains("visit :: < __T , __B , __F > (& self . amount"));
    }

    #[test]
    fn with_takes_a_path_and_no_other_argument() {
        let duplicate = parse_quote! {
            struct Request {
                #[typesift(with = visit_headers, with = visit_headers)]
                headers: Headers,
            }
        };
        assert_eq!(error_message(duplicate), "duplicate `with`");

        let combined = parse_quote! {
            struct Request {
                #[typesift(skip, with = visit_headers)]
                headers: Headers,
            }
        };
        assert_eq!(
            error_message(combined),
            "`skip` and `with` cannot be combined"
        );

        let without_a_path = parse_quote! {
            struct Request {
                #[typesift(with)]
                headers: Headers,
            }
        };
        assert!(error_message(without_a_path).contains("expected"));

        let generated = generated_code(parse_quote! {
            struct Request {
                path: String,
                #[typesift(with = visit_headers)]
                headers: Headers,
            }
        });
        assert!(generated.contains("visit_headers"));
        assert!(generated.contains("Sift"));
        assert!(generated.contains("visit :: < __T , __B , __F > (& self . path"));
    }
}
