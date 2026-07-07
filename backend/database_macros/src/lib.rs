use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::{Field, ItemStruct, parse_macro_input};

#[proc_macro_attribute]
pub fn schema(_: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let struct_name = &item.ident;

    let create_name = format_ident!("Create{}", struct_name);
    let update_name = format_ident!("Update{}", struct_name);
    let patch_name = format_ident!("Patch{}", struct_name);

    let is_generated = |f| has_flag(f, "generated");
    let is_immutable = |f| has_flag(f, "immutable");

    let quote = |f: &Field, patch: bool| {
        let name = &f.ident;
        let ty = &f.ty;
        match patch {
            false => quote! { pub #name: #ty },
            true => quote! { pub #name: crate::PatchField<#ty> }, // this means it only works in the context of chdrms_database
        }
    };

    let fields: Vec<_> = item.fields.iter().map(|f| quote(f, false)).collect();
    let create_fields: Vec<_> = item
        .fields
        .iter()
        .filter(|f| !is_generated(f))
        .map(|f| quote(f, false))
        .collect();
    let update_fields: Vec<_> = item
        .fields
        .iter()
        .filter(|f| !is_immutable(f))
        .map(|f| quote(f, false))
        .collect();
    let patch_fields: Vec<_> = item
        .fields
        .iter()
        .filter(|f| !is_immutable(f))
        .map(|f| quote(f, true))
        .collect();

    quote! {
        #[derive(Debug, Clone, PartialEq, ::sqlx::FromRow)]
        pub struct #struct_name {
            #(#fields,)*
        }

        #[derive(Debug, Clone, PartialEq, ::sqlx::FromRow)]
        pub struct #create_name {
            #(#create_fields,)*
        }

        #[derive(Debug, Clone, PartialEq, ::sqlx::FromRow)]
        pub struct #update_name {
            #(#update_fields,)*
        }

        #[derive(Debug, Clone, PartialEq, ::sqlx::FromRow)]
        pub struct #patch_name {
            #(#patch_fields,)*
        }
    }
    .into()
}

fn has_flag(field: &Field, flag: &str) -> bool {
    field.attrs.iter().any(|attr| {
        attr.path().is_ident("schema")
            && attr
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated,
                )
                .map(|idents| idents.iter().any(|i| i == flag))
                .unwrap_or(false)
    })
}
