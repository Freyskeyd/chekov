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

    let registry = proc_macro2::Ident::new(
        &format!("{}EventRegistration", struct_name.to_string()),
        proc_macro2::Span::call_site(),
    );

    Ok(quote! {

        pub struct #registry {
            resolver: Box<dyn Fn(&str, &actix::Context<chekov::prelude::AggregateInstance<#struct_name>>)>,
        }

        impl chekov::aggregate::EventRegistryItem<#struct_name> for #registry {
            fn get_resolver(&self) -> &Box<dyn Fn(&str, &actix::Context<chekov::prelude::AggregateInstance<#struct_name>>)> {
                &self.resolver
            }
        }

        inventory::collect!(#registry);

        impl chekov::Aggregate for #struct_name {
            type EventRegistry = #registry;

            fn identity() -> &'static str {
                #identity
            }
        }
    })
}
