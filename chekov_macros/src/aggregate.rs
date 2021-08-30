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

    let registry = format_ident!("{}EventRegistration", struct_name.to_string());

    let aggregate_event_resolver =
        format_ident!("{}EventResolverRegistry", struct_name.to_string());

    let aggregate_static_resolver = format_ident!(
        "{}_STATIC_EVENT_RESOLVER",
        struct_name.to_string().to_uppercase()
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
            names: Vec<&'static str>,
            type_id: std::any::TypeId,
            deserializer: chekov::event::EventResolverFn<chekov::prelude::AggregateInstance<#struct_name>>,
        }

        impl chekov::aggregate::EventResolverItem<#struct_name> for #aggregate_event_resolver {
            fn get_resolver(&self) -> chekov::event::EventResolverFn<chekov::prelude::AggregateInstance<#struct_name>> {
                self.deserializer
            }

            fn get_names(&self) -> &[&'static str] {
                self.names.as_ref()
            }
        }

        inventory::collect!(#aggregate_event_resolver);

        chekov::lazy_static! {
            #[derive(Clone, Debug)]
            static ref #aggregate_static_resolver: chekov::event::EventResolverRegistry<chekov::prelude::AggregateInstance<#struct_name>> = {
                let mut resolvers = std::collections::BTreeMap::new();
                let mut names = std::collections::BTreeMap::new();

                for registered in inventory::iter::<#aggregate_event_resolver> {
                    resolvers.insert(registered.type_id, registered.deserializer);

                    registered.names.iter().for_each(|name|{
                        names.insert(name.clone(), registered.type_id);
                    });
                }

                chekov::event::EventResolverRegistry {
                    names,
                    resolvers
                }
            };
        }
        impl chekov::Aggregate for #struct_name {
            type EventRegistry = #registry;

            fn get_event_resolver(event_name: &str) -> Option<&fn(event_store::prelude::RecordedEvent, actix::Addr<chekov::prelude::AggregateInstance<#struct_name>>) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<(), ()>> + Send>>> {
                #aggregate_static_resolver.get(event_name)
            }

            fn identity() -> &'static str {
                #identity
            }
        }
    })
}
