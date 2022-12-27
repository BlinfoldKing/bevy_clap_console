use darling::FromDeriveInput;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(command), forward_attrs(allow, doc, cfg))]
struct Opts {
    name: String,
    alias: Option<String>,
}

#[proc_macro_derive(ConsoleCommand, attributes(command))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("invalid command");
    let DeriveInput { ident, .. } = input;

    let name = opts.name;
    let name_check = match opts.alias {
        None => quote! { line[0] != #name },
        Some(alias) => quote! { line[0] != #name && line[0] != #alias},
    };

    let output = quote! {
        impl ConsoleCommand for #ident {
            fn get(line: Vec<String>) -> Result<Self, ConsoleError> {
                if #name_check {
                    return Err(ConsoleError::Unknown)
                }
                match Self::try_parse_from(line) {
                    Ok(res) => Ok(res),
                    Err(err) => Err(ConsoleError::ClapError(err.to_string()))
                }
            }
        }
    };
    output.into()
}
