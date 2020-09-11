use darling::*;
use proc_macro2::TokenStream as SynTokenStream;
use quote::*;
use std::result::Result;
use syn::*;

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(aggregate), supports(struct_named))]
struct AggregateAttrs {
    ident: Ident,
    generics: Generics,

    identity: String,
}

pub fn generate_aggregate(
    input: &DeriveInput,
    _: &DataStruct,
) -> Result<SynTokenStream, SynTokenStream> {
    let container: AggregateAttrs = match FromDeriveInput::from_derive_input(input) {
        Ok(v) => v,
        Err(e) => return Err(e.write_errors()),
    };

    let struct_name = container.ident;
    let identity = container.identity;

    Ok(quote! {
        impl chekov::Aggregate for #struct_name {
            fn identity() -> &'static str {
                #identity
            }
        }
    })
}
