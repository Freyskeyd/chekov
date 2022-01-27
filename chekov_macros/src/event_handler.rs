use darling::*;
use proc_macro2::TokenStream as SynTokenStream;
use quote::*;
use std::result::Result;
use syn::*;

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(aggregate), supports(struct_named))]
struct EventHandlerAttrs {
    ident: Ident,
    #[allow(dead_code)]
    generics: Generics,
}

pub fn generate_event_handler(
    input: &DeriveInput,
    _: &DataStruct,
) -> Result<SynTokenStream, SynTokenStream> {
    let container: EventHandlerAttrs = match FromDeriveInput::from_derive_input(input) {
        Ok(v) => v,
        Err(e) => return Err(e.write_errors()),
    };

    let struct_name = container.ident;

    let aggregate_event_resolver =
        format_ident!("{}EventHandlerEventRegistry", struct_name.to_string(),);

    let aggregate_static_resolver = format_ident!(
        "{}_STATIC_EVENT_HANDLER_EVENT_RESOLVER",
        struct_name.to_string().to_uppercase()
    );

    Ok(quote! {

        pub struct #aggregate_event_resolver {
            names: Vec<&'static str>,
            type_id: std::any::TypeId,
            handler: fn(&mut #struct_name, chekov::RecordedEvent)  -> BoxFuture<Result<(), ()>>
        }

        chekov::inventory::collect!(#aggregate_event_resolver);

        chekov::lazy_static! {
            #[derive(Clone, Debug)]
            static ref #aggregate_static_resolver: chekov::event::resolver::EventHandlerResolverRegistry<#struct_name> = {
                let mut handlers = std::collections::BTreeMap::new();
                let mut names = std::collections::BTreeMap::new();

                for registered in chekov::inventory::iter::<#aggregate_event_resolver> {
                    handlers.insert(registered.type_id, registered.handler);

                    registered.names.iter().for_each(|name|{
                        names.insert(name.clone(), registered.type_id);
                    });
                }

                chekov::event::resolver::EventHandlerResolverRegistry {
                    names,
                    handlers
                }
            };
        }

        #[chekov::async_trait::async_trait]
        impl chekov::EventHandler for #struct_name {

            async fn handle_recorded_event(state: &mut Self, event: chekov::RecordedEvent) -> Result<(), ()> {
                if let Some(resolver) = #aggregate_static_resolver.get(&event.event_type) {
                    return (resolver)(state, event).await;
                }

                Ok(())
            }
        }
    })
}
