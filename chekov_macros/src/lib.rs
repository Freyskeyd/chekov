extern crate proc_macro;

mod aggregate;
mod command;
mod event;

use proc_macro::TokenStream;
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

use quote::*;

#[derive(Debug)]
struct Options {
    consistency: Option<proc_macro2::Ident>,
    // bracket_token: token::Bracket,
    // content: TokenStream,
}

impl syn::parse::Parse for Options {
    fn parse(_input: syn::parse::ParseStream) -> Result<Self> {
        // let name = if let Ok(option) = input.parse::<syn::Ident>() {
        //     input.parse::<Token![:]>()?;
        //     input.parse::<Token![:]>()?;
        //     input.parse()?
        // } else {
        //     panic!()
        // };

        // if let Ok(_) = input.parse::<Token![,]>() {
        //     input.parse::<syn::Ident>()?;
        //     input.parse::<Token![:]>()?;
        //     input.parse::<syn::LitInt>()?;
        // }

        Ok(Options { consistency: None })
    }
}

#[derive(Debug)]
struct Dispatch {
    cmd: proc_macro2::Ident,
    // bracket_token: token::Bracket,
    // content: TokenStream,
}

impl syn::parse::Parse for Dispatch {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let cmd: syn::Ident = input.parse()?;

        Ok(Dispatch { cmd })
    }
}

#[proc_macro]
pub fn dispatch(item: TokenStream) -> TokenStream {
    let _input: Dispatch = parse_macro_input!(item);

    let toks = quote! {
        "ok"
    };

    toks.into()
}
