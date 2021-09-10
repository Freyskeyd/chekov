extern crate proc_macro;

mod aggregate;
mod command;
mod event;
mod event_handler;

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

#[proc_macro_derive(EventHandler, attributes(event_handler))]
pub fn derive_event_handler(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    if let Data::Struct(data) = &input.data {
        match event_handler::generate_event_handler(&input, data) {
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

#[proc_macro_attribute]
pub fn applier(args: TokenStream, input: TokenStream) -> TokenStream {
    expand(args, input)
}

#[proc_macro_attribute]
pub fn event_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    expand_event_handler(args, input)
}

use syn::parse::{Parse, ParseStream};
use syn::{Error, ItemImpl, TypePath};

#[allow(dead_code)]
pub(crate) enum TraitArgs {
    External,
    Internal { tag: LitStr },
    Adjacent { tag: LitStr, content: LitStr },
}

pub(crate) struct ImplArgs {
    #[allow(dead_code)]
    pub name: Option<LitStr>,
}

pub(crate) enum Input {
    Trait(ItemTrait),
    Impl(ItemImpl),
}

mod kw {
    syn::custom_keyword!(tag);
    syn::custom_keyword!(content);
    syn::custom_keyword!(name);
}
impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = Attribute::parse_outer(input)?;

        let ahead = input.fork();
        ahead.parse::<Visibility>()?;
        ahead.parse::<Option<Token![unsafe]>>()?;

        if ahead.peek(Token![trait]) {
            let mut item: ItemTrait = input.parse()?;
            attrs.extend(item.attrs);
            item.attrs = attrs;
            Ok(Input::Trait(item))
        } else if ahead.peek(Token![impl]) {
            let mut item: ItemImpl = input.parse()?;
            if item.trait_.is_none() {
                let impl_token = item.impl_token;
                let ty = item.self_ty;
                let span = quote!(#impl_token #ty);
                let msg = "expected impl Trait for Type";
                return Err(Error::new_spanned(span, msg));
            }
            attrs.extend(item.attrs);
            item.attrs = attrs;
            Ok(Input::Impl(item))
        } else {
            Err(input.error("expected trait or impl block"))
        }
    }
}

impl Parse for ImplArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = if input.is_empty() {
            None
        } else {
            input.parse::<kw::name>()?;
            input.parse::<Token![=]>()?;
            let name: LitStr = input.parse()?;
            Some(name)
        };
        Ok(ImplArgs { name })
    }
}

fn expand(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);

    match input {
        Input::Trait(_) => {
            panic!("`applier` can't be used on Trait.")
        }
        Input::Impl(input) => {
            let args = parse_macro_input!(args as ImplArgs);
            expand_applier(args, input)
        }
    }
}

pub(crate) fn expand_applier(_args: ImplArgs, input: ItemImpl) -> TokenStream {
    let object = &input.trait_.as_ref().unwrap().1;
    let this = &input.self_ty;

    let apply_to = if let syn::Type::Path(TypePath { ref path, .. }) = **this {
        path.get_ident().unwrap()
    } else {
        panic!("Mehhhhhh")
    };

    let aggregate_event_resolver = format_ident!("{}EventResolverRegistry", apply_to.to_string(),);

    let mut event: Option<Ident> = None;
    for segment in &object.segments {
        let PathSegment { ident, arguments } = segment;
        if ident == "EventApplier" {
            if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
                arguments
            {
                if let Some(GenericArgument::Type(syn::Type::Path(syn::TypePath {
                    path, ..
                }))) = args.first()
                {
                    event = path.get_ident().cloned();
                }
            }
        }
    }
    let mut expanded = quote! {
        #input
    };

    let event = event.unwrap();
    expanded.extend(quote! {
            chekov::inventory::submit! {
                #![crate = chekov]
                use chekov::event_store::Event;
                #aggregate_event_resolver {
                    names: #event::all_event_types(),
                    type_id: std::any::TypeId::of::<#event>(),
                    applier: |aggregate: &mut #apply_to, event: chekov::RecordedEvent| -> Result<(), ApplyError> {
                        use chekov::Event;
                        use futures::TryFutureExt;

                        let e = #event::into_envelope(event).unwrap();

                        aggregate.apply(&e.event)
                    }
                }
            }
        });

    expanded.into()
}

fn expand_event_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);

    match input {
        Input::Trait(_) => {
            panic!("`event_handler` can't be used on Trait.")
        }
        Input::Impl(input) => {
            let args = parse_macro_input!(args as ImplArgs);
            expand_event_handler_do(args, input)
        }
    }
}

pub(crate) fn expand_event_handler_do(_args: ImplArgs, input: ItemImpl) -> TokenStream {
    let object = &input.trait_.as_ref().unwrap().1;
    let this = &input.self_ty;

    let apply_to = if let syn::Type::Path(TypePath { ref path, .. }) = **this {
        path.get_ident().unwrap()
    } else {
        panic!()
    };

    let aggregate_event_resolver =
        format_ident!("{}EventHandlerEventRegistry", apply_to.to_string(),);

    let mut event: Option<Ident> = None;
    for segment in &object.segments {
        let PathSegment { ident, arguments } = segment;
        if ident == "Handler" {
            if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
                arguments
            {
                if let Some(GenericArgument::Type(syn::Type::Path(syn::TypePath {
                    path, ..
                }))) = args.first()
                {
                    event = path.get_ident().cloned();
                }
            }
        }
    }
    let mut expanded = quote! {
        #input
    };

    let event = event.unwrap();
    expanded.extend(quote! {
        chekov::inventory::submit! {
            #![crate = chekov]
            use chekov::event_store::Event;
            #aggregate_event_resolver {
                names: #event::all_event_types(),
                type_id: std::any::TypeId::of::<#event>(),
                handler: |handler: &mut #apply_to, event: chekov::RecordedEvent|  -> BoxFuture<Result<(), ()>> {
                    use chekov::Event;
                    use chekov::event::Handler;
                    use futures::TryFutureExt;

                    let e = #event::into_envelope(event).unwrap();

                    handler.handle(&e.event)
                }
            }
        }
    });

    expanded.into()
}
