use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parenthesized, Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Generics, Meta, Token};



#[proc_macro_derive(TypeIter, attributes(type_iter))]
pub fn type_iter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand_type_iter(input)
        .unwrap_or_else(|e| syn::Error::to_compile_error(&e))
        .into()
}


fn expand_type_iter(input: proc_macro::TokenStream) -> Result<TokenStream, syn::Error> {
    let derive_input: DeriveInput = syn::parse(input)?;
    let needle_types = parse_needle_types(&derive_input)?;
    gen_impl(&derive_input, &needle_types)
}


fn parse_needle_types(input: &DeriveInput) -> Result<Vec<Ident>, syn::Error> {
    let maybe_attr = input.attrs.iter().find(|attr| attr.path().is_ident("type_iter"));
    let Some(attr) = maybe_attr else {
        return Ok(Vec::new());
    };
    let list = match &attr.meta {
        Meta::List(list) => list,
        _ => {
            return Err(syn::Error::new(
                attr.span(),
                "Expected `type_iter` attribute to be a list",
            ));
        }
    };
    let idents: Vec<Ident> = list.tokens.clone().into_iter().filter_map(|token| {
        if let TokenTree::Ident(ident) = token {
            Some(ident)
        } else {
            None
        }
    }).collect();

    Ok(idents)
}

fn gen_impl(input: &DeriveInput, needle_types: &[Ident]) -> Result<TokenStream, syn::Error> {
    let impl_for_self = gen_impl_for_self(&input.ident);
    let impl_for_needle_types = needle_types.iter().map(|needle_type| {
        gen_impl_for_type(&input.ident, &input.data, needle_type)
    });

    Ok(quote!{
        #impl_for_self
        #(#impl_for_needle_types)*
    })
}

fn gen_impl_for_self(type_name: &Ident) -> TokenStream {
    quote! {
        impl TypeIter<#type_name> for #type_name {
            fn type_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a #type_name> + 'a> {
                Box::new(std::iter::once(self))
            }
        }
    }
}

fn gen_impl_for_type(container_type: &Ident, data: &Data, needle_type: &Ident) -> TokenStream {
    match data {
        Data::Struct(data_struct) => gen_impl_for_struct(container_type, data_struct, needle_type),
        Data::Enum(data_enum) => gen_impl_for_enum(container_type, data_enum, needle_type),
        Data::Union(_) => unimplemented!(),
    }
}

fn gen_impl_for_struct(container_type: &Ident, data_struct: &DataStruct, needle_type: &Ident) -> TokenStream {
    match &data_struct.fields {
        Fields::Named(fields) => gen_impl_for_struct_named(container_type, needle_type, fields),
        Fields::Unnamed(fields) => gen_impl_for_struct_unnamed(container_type, needle_type, fields),
        Fields::Unit => todo!(),
    }
}

fn gen_impl_for_struct_named(container_type: &Ident, needle_type: &Ident, fields: &FieldsNamed) -> TokenStream {
    let field_impls = fields.named.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Field with name");
        quote! {
            .chain(self.#field_name.type_values::<#needle_type>())
        }
    });

    quote! {
        impl TypeIter<#needle_type> for #container_type {
            fn type_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a #needle_type> + 'a> {
                let empty_iter = std::iter::empty();
                let iter = empty_iter
                    #(#field_impls)*;
                Box::new(iter)
            }
        }
    }
}

fn gen_impl_for_struct_unnamed(container_type: &Ident, needle_type: &Ident, fields: &FieldsUnnamed) -> TokenStream {
    let field_impls = fields.unnamed.iter().enumerate().map(|(i, _)| {
        let index = syn::Index::from(i);
        quote! {
            .chain(self.#index.type_values::<#needle_type>())
        }
    });

    quote! {
        impl TypeIter<#needle_type> for #container_type {
            fn type_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a #needle_type> + 'a> {
                let empty_iter = std::iter::empty();
                let iter = empty_iter
                    #(#field_impls)*;
                Box::new(iter)
            }
        }
    }
}

fn gen_impl_for_enum(container_type: &Ident, data_enum: &DataEnum, needle_type: &Ident) -> TokenStream {

    quote! {
        impl TypeIter<#needle_type> for #container_type {
            fn type_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a #needle_type> + 'a> {
                let empty_iter = std::iter::empty();
                todo!("Implement for enum");
                Box::new(empty_iter)
            }
        }
    }

    // let variant_impls = data_enum.variants.iter().map(|variant| {
    //     let variant_name = &variant.ident;
    //     let variant_impl = gen_impl_for_enum_variant(container_type, variant, needle_type);
    //     quote! {
    //         #variant_impl
    //     }
    // });

    // quote! {
    //     #(#variant_impls)*
    // }
}

// fn gen_impl_for_enum_variant(container_type: &Ident, variant: &syn::Variant, needle_type: &Ident) -> TokenStream {
//     match &variant.fields {
//         Fields::Named(fields) => gen_impl_for_enum_variant_named(container_type, &variant.ident, needle_type, fields),
//         Fields::Unnamed(fields) => gen_impl_for_enum_variant_unnamed(container_type, &variant.ident, needle_type, fields),
//         Fields::Unit => gen_impl_for_enum_variant_unit(container_type, &variant.ident, needle_type),
//     }
// }
