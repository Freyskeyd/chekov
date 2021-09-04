extern crate proc_macro;

mod aggregate;
mod command;
mod event;

use proc_macro::TokenStream;
use quote::*;
use syn::*;

#[proc_macro_derive(Command, attributes(command))]
pub fn derive_command(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    if let Data::Struct(data) = &input.data {
        match command::generate_command(&input, data) {
            Ok(toks) => toks.into(),
            Err(toks) => toks.into(),
        }
    } else {
        panic!("")
    }
}

#[proc_macro_derive(Aggregate, attributes(aggregate))]
pub fn derive_aggregate(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    if let Data::Struct(data) = &input.data {
        match aggregate::generate_aggregate(&input, data) {
            Ok(toks) => toks.into(),
            Err(toks) => toks.into(),
        }
    } else {
        panic!("")
    }
}

#[proc_macro_derive(Event, attributes(event))]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    match &input.data {
        Data::Struct(_) | Data::Enum(_) => match event::generate_event(&input) {
            Ok(toks) => toks.into(),
            Err(toks) => toks.into(),
        },
        _ => panic!(""),
    }
}

#[derive(Debug)]
struct ApplyEvent {
    event: proc_macro2::Ident,
    apply_to: proc_macro2::Ident,
}

impl syn::parse::Parse for ApplyEvent {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let apply_to: syn::Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let event: syn::Ident = input.parse()?;

        Ok(ApplyEvent { event, apply_to })
    }
}

#[proc_macro]
pub fn apply_event(item: TokenStream) -> TokenStream {
    let apply: ApplyEvent = parse_macro_input!(item);

    let event = apply.event;
    let apply_to = apply.apply_to;

    let aggregate_event_resolver = format_ident!("{}EventResolverRegistry", apply_to.to_string(),);

    let toks = quote! {

        inventory::submit! {
            use event_store::Event;
            #aggregate_event_resolver {
                names: #event::all_event_types(),
                type_id: std::any::TypeId::of::<#event>(),
                applier: |aggregate: &mut #apply_to, event: event_store::prelude::RecordedEvent| -> Result<(), chekov::prelude::ApplyError> {
                    use chekov::Event;
                    use futures::TryFutureExt;

                    let e = #event::into_envelope(event).unwrap();

                    aggregate.apply(&e.event)
                }
            }
        }
    };

    toks.into()
}
