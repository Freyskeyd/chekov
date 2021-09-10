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

    let aggregate_event_resolver =
        format_ident!("{}EventResolverRegistry", struct_name.to_string());

    let aggregate_static_resolver = format_ident!(
        "{}_STATIC_EVENT_RESOLVER",
        struct_name.to_string().to_uppercase()
    );

    Ok(quote! {

        pub struct #aggregate_event_resolver {
            names: Vec<&'static str>,
            type_id: std::any::TypeId,
            applier: chekov::aggregate::resolver::EventApplierFn<#struct_name>
        }

        impl chekov::aggregate::resolver::EventResolverItem<#struct_name> for #aggregate_event_resolver {
            fn get_names(&self) -> &[&'static str] {
                self.names.as_ref()
            }
        }

        chekov::inventory::collect!(#aggregate_event_resolver);

        chekov::lazy_static! {
            #[derive(Clone, Debug)]
            static ref #aggregate_static_resolver: chekov::aggregate::resolver::EventResolverRegistry<#struct_name> = {
                let mut appliers = std::collections::BTreeMap::new();
                let mut names = std::collections::BTreeMap::new();

                for registered in chekov::inventory::iter::<#aggregate_event_resolver> {
                    appliers.insert(registered.type_id, registered.applier);

                    registered.names.iter().for_each(|name|{
                        names.insert(name.clone(), registered.type_id);
                    });
                }

                chekov::aggregate::resolver::EventResolverRegistry {
                    names,
                    appliers
                }
            };
        }
        impl chekov::Aggregate for #struct_name {

            fn apply_recorded_event(&mut self, event: chekov::RecordedEvent) -> Result<(), chekov::prelude::ApplyError> {
                if let Some(resolver) = #aggregate_static_resolver.get_applier(&event.event_type) {
                    let _ = (resolver)(self, event);
                }

                Ok(())
            }

            fn identity() -> &'static str {
                #identity
            }
        }
    })
}
