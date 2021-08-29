use darling::ast::Data;
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

struct EventState {
    event_name: Ident,
    event_type: String,
    data: Data<EventVariantAttrs, EventFieldAttrs>,
}

impl EventState {
    fn from_container(container: EventAttrs) -> Self {
        Self {
            event_name: container.ident.clone(),
            event_type: if let Some(event_type) = container.event_type {
                event_type
            } else {
                container.ident.to_string()
            },
            data: container.data,
        }
    }

    /// Get a reference to the event state's event name.
    fn event_name(&self) -> &Ident {
        &self.event_name
    }

    /// Get a reference to the event state's event type.
    fn event_type(&self) -> &str {
        self.event_type.as_str()
    }

    fn try_from_trait(&self) -> SynTokenStream {
        let struct_name = self.event_name();
        let input_name = format_ident!("event");

        let try_from = if self.data.is_enum() {
            quote! {
                Err(())
            }
        } else {
            quote! {
                ::serde_json::from_value::<#struct_name>(#input_name.data).map_err(|_|())
            }
        };

        quote! {
            impl std::convert::TryFrom<event_store::prelude::RecordedEvent> for #struct_name {
                type Error = ();
                fn try_from(#input_name: event_store::prelude::RecordedEvent) -> std::result::Result<Self, Self::Error> {
                    #try_from
                }
            }
        }
    }

    fn event_store_event_trait(&self) -> SynTokenStream {
        let struct_name = self.event_name();
        quote! {    impl chekov::event::Event for #struct_name {}}
    }

    fn chekov_event_trait(&self) -> SynTokenStream {
        let struct_name = self.event_name();
        let base_event_type = self.event_type();

        let event_type = if self.data.is_enum() {
            if let Some(variants) = self.data.as_ref().take_enum() {
                let vars: Vec<SynTokenStream> = variants
                    .iter()
                    .map(|x| {
                        let v_ident = x.ident.clone();
                        let string_representation = if let Some(variant_event_type) = &x.event_type
                        {
                            format!("{}::{}", base_event_type, variant_event_type)
                        } else {
                            format!("{}::{}", base_event_type, v_ident)
                        };
                        quote! {
                            #struct_name::#v_ident { .. } => #string_representation
                        }
                    })
                    .collect();

                quote! {
                    match *self {
                        #(
                            #vars,
                        )*
                    }
                }
            } else {
                panic!("Event defined as enum must have at least one variant.");
            }
        } else {
            quote! {
                #base_event_type
            }
        };

        let all_event_types = if self.data.is_enum() {
            if let Some(variants) = self.data.as_ref().take_enum() {
                let strings: Vec<SynTokenStream> = variants
                    .iter()
                    .map(|x| {
                        let v_ident = x.ident.clone();
                        let string_representation = if let Some(variant_event_type) = &x.event_type
                        {
                            format!("{}::{}", base_event_type, variant_event_type)
                        } else {
                            format!("{}::{}", base_event_type, v_ident)
                        };
                        quote! {
                            #string_representation
                        }
                    })
                    .collect();

                quote! {
                    vec![#(#strings,)*]
                }
            } else {
                panic!("Event defined as enum must have at least one variant.");
            }
        } else {
            quote! {
                vec![#base_event_type]
            }
        };

        quote! {
            impl event_store::prelude::Event for #struct_name {
                fn event_type(&self) -> &'static str {
                    #event_type
                }

                fn all_event_types() -> Vec<&'static str> {
                    #all_event_types
                }
            }
        }
    }
}

impl From<EventState> for Result<SynTokenStream, SynTokenStream> {
    fn from(event: EventState) -> Self {
        let try_from_trait = event.try_from_trait();
        let event_trait = event.chekov_event_trait();
        let event_store_trait = event.event_store_event_trait();

        Ok(quote! {
            #try_from_trait
            #event_trait
            #event_store_trait
        })
    }
}

pub fn generate_event(input: &DeriveInput) -> Result<SynTokenStream, SynTokenStream> {
    match FromDeriveInput::from_derive_input(input) {
        Ok::<EventAttrs, _>(container) => EventState::from_container(container.clone()).into(),
        Err(e) => {
            return Err(e.write_errors());
        }
    }
}
