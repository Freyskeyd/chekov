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

    let aggregate_event_resolver = proc_macro2::Ident::new(
        &format!("{}EventResolverRegistry", struct_name.to_string()),
        proc_macro2::Span::call_site(),
    );

    Ok(quote! {

        pub struct #registry {
            resolver: Box<dyn Fn(&str, &actix::Context<chekov::prelude::AggregateInstance<#struct_name>>)>,
        }

        impl chekov::aggregate::EventRegistryItem<#struct_name> for #registry {
            fn get_resolver(&self) -> &dyn Fn(&str, &actix::Context<chekov::prelude::AggregateInstance<#struct_name>>) {
                &self.resolver
            }
        }

        inventory::collect!(#registry);

        pub struct #aggregate_event_resolver {
            name: &'static str,
            deserializer: fn(event_store::prelude::RecordedEvent, actix::Addr<chekov::prelude::AggregateInstance<#struct_name>>) -> std::result::Result<(), ()>,
        }

        impl chekov::aggregate::EventResolverItem<#struct_name> for #aggregate_event_resolver {
            fn get_resolver(&self) -> fn(event_store::prelude::RecordedEvent, actix::Addr<chekov::prelude::AggregateInstance<#struct_name>>) -> std::result::Result<(), ()> {
                self.deserializer
            }

            fn get_name(&self) -> &'static str {
                self.name
            }
        }

        inventory::collect!(#aggregate_event_resolver);

        impl chekov::Aggregate for #struct_name {
            type EventRegistry = #registry;
            type EventResolver = #aggregate_event_resolver;

            fn identity() -> &'static str {
                #identity
            }
        }
    })
}
