use darling::*;
use proc_macro2::TokenStream as SynTokenStream;
use quote::*;
use std::result::Result;
use syn::*;

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(command), supports(struct_named))]
struct CommandAttrs {
    ident: Ident,
    generics: Generics,

    event: Path,
    aggregate: Path,
    #[darling(default)]
    handler: Option<Path>,
    #[darling(default)]
    identifier: Option<String>,
}

#[derive(Debug, Clone, FromField)]
#[darling(attributes(command), forward_attrs(doc))]
struct CommandFieldAttrs {
    ident: Option<Ident>,
    attrs: Vec<Attribute>,

    #[darling(default)]
    identifier: Option<bool>,
}

pub fn generate_command(
    input: &DeriveInput,
    data: &DataStruct,
) -> Result<SynTokenStream, SynTokenStream> {
    let container: CommandAttrs = match FromDeriveInput::from_derive_input(input) {
        Ok(v) => v,
        Err(e) => return Err(e.write_errors()),
    };

    let struct_name = container.ident;
    let event = container.event;
    let executor = container.aggregate.get_ident().unwrap();
    let attrs = data
        .fields
        .iter()
        .filter_map(|i| {
            let v: Option<CommandFieldAttrs> = match FromField::from_field(i) {
                Ok(v) => Some(v),
                Err(_) => None,
            };

            v
        })
        .collect::<Vec<CommandFieldAttrs>>();

    let identifiers: Vec<&CommandFieldAttrs> =
        attrs.iter().filter(|x| x.identifier.is_some()).collect();

    if identifiers.len() > 1 {
        panic!("Can't use multiple identifier on fields, use identifier on struct instead.");
    }

    let identifier_value = identifiers.first().unwrap().ident.clone();
    Ok(quote! {

        #[async_trait::async_trait]
        impl chekov::Command for #struct_name {
            type Event = #event;
            type Executor = #executor;
            type ExecutorRegistry = ::chekov::AggregateInstanceRegistry<#executor>;

            fn identifier(&self) -> ::std::string::String {
                self.#identifier_value.to_string()
            }
            async fn dispatch(&self) -> Result<(), ()>{

                Ok(())
            }
        }
    })
}
