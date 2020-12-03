use darling::*;
use proc_macro2::TokenStream as SynTokenStream;
use quote::*;
use std::result::Result;
use syn::*;

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(event))]
struct EventAttrs {
    ident: Ident,
    generics: Generics,
    data: darling::ast::Data<EventVariantAttrs, EventFieldAttrs>,
    #[darling(default)]
    event_type: Option<String>,
}

#[derive(Debug, Clone, FromVariant)]
#[darling(attributes(event))]
struct EventVariantAttrs {
    ident: Ident,
    attrs: Vec<Attribute>,

    #[darling(default)]
    event_type: Option<String>,
}

#[derive(Debug, Clone, FromField)]
#[darling(attributes(event))]
struct EventFieldAttrs {
    ident: Option<Ident>,
    attrs: Vec<Attribute>,

    #[darling(default)]
    event_type: Option<String>,
}

pub fn generate_event(input: &DeriveInput) -> Result<SynTokenStream, SynTokenStream> {
    let container: EventAttrs = match FromDeriveInput::from_derive_input(input) {
        Ok(v) => v,
        Err(e) => {
            return Err(e.write_errors());
        }
    };

    let struct_name = container.ident.clone();
    let struct_event_type = if let Some(event_type) = container.event_type {
        event_type
    } else {
        struct_name.to_string()
    };

    let (event_type, all_event_types, try_from) = if container.data.as_ref().is_enum() {
        if let Some(variants) = container.data.take_enum() {
            let vars: Vec<SynTokenStream> = variants
                .iter()
                .map(|x| {
                    let name = struct_name.clone();
                    let v_ident = x.ident.clone();
                    let string_representation = if let Some(event_type) = &x.event_type {
                        format!("{}::{}", struct_event_type, event_type)
                    } else {
                        format!("{}::{}", struct_event_type, v_ident)
                    };
                    quote! {
                        #name::#v_ident { .. } => #string_representation
                    }
                })
                .collect();

            let strings: Vec<SynTokenStream> = variants
                .iter()
                .map(|x| {
                    let _name = struct_name.clone();
                    let v_ident = x.ident.clone();
                    let string_representation = if let Some(event_type) = &x.event_type {
                        format!("{}::{}", struct_event_type, event_type)
                    } else {
                        format!("{}::{}", struct_event_type, v_ident)
                    };
                    quote! {
                        #string_representation
                    }
                })
                .collect();
            (
                quote! {
                    match *self {
                        #(
                            #vars,
                        )*
                    }
                },
                quote! {
                    vec![#(#strings,)*]
                },
                quote! {
                    Err(())
                },
            )
        } else {
            panic!()
        }
    } else {
        (
            quote! {
                #struct_event_type
            },
            quote! {
                vec![#struct_event_type]
            },
            quote! {
                ::serde_json::from_value::<#struct_name>(e.data).map_err(|_|())
            },
        )
    };

    Ok(quote! {
        impl std::convert::TryFrom<event_store::prelude::RecordedEvent> for #struct_name {
            type Error = ();
            fn try_from(e: event_store::prelude::RecordedEvent) -> Result<Self, Self::Error> {
                #try_from
            }
        }

        impl chekov::event::Event for #struct_name {}
        impl event_store::prelude::Event for #struct_name {
            fn event_type(&self) -> &'static str {
                #event_type
            }

            fn all_event_types() -> Vec<&'static str> {
                #all_event_types
            }
        }
    })
}
