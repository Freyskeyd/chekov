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
    app: proc_macro2::Ident,
    event: proc_macro2::Ident,
    apply_to: proc_macro2::Ident,
    closure: proc_macro2::Ident,
}

impl syn::parse::Parse for ApplyEvent {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let app: syn::Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let apply_to: syn::Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let event: syn::Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let closure: syn::Ident = input.parse()?;

        Ok(ApplyEvent {
            app,
            event,
            apply_to,
            closure,
        })
    }
}

#[proc_macro]
pub fn apply_event(item: TokenStream) -> TokenStream {
    let apply: ApplyEvent = parse_macro_input!(item);

    let app = apply.app;
    let event = apply.event;
    let apply_to = apply.apply_to;
    let closure = apply.closure;

    let registry = proc_macro2::Ident::new(
        &format!("{}EventRegistration", apply_to.to_string()),
        proc_macro2::Span::call_site(),
    );
    let toks = quote! {

        inventory::submit! {
            use actix::AsyncContext;
            use actix::SystemService;
            #registry { resolver: Box::new(|stream: &str, ctx: &actix::Context<AggregateInstance<#apply_to>>|{
                chekov::SubscriberManager::<#app>::from_registry().do_send(Subscribe(stream.into(), ctx.address().recipient::<chekov::message::EventEnvelope<#event>>(), stream.into()));
            })}
        }

        impl chekov::prelude::EventApplier<#event> for #apply_to {
            fn apply(&mut self, event: &#event) -> Result<(), chekov::prelude::ApplyError> {
                #closure(self, event)
            }
        }
    };

    toks.into()
}
