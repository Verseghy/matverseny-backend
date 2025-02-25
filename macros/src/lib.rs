use proc_macro::TokenStream;
use syn::{Error, LitStr, parse_macro_input};

#[proc_macro]
pub fn error_code_to_ident(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as LitStr);
    error_code_to_ident::expand(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

mod error_code_to_ident {
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};
    use syn::{Error, LitStr, Result};

    pub fn expand(input: LitStr) -> Result<TokenStream> {
        let code = input.value();

        if !code.is_ascii() {
            return Err(Error::new(input.span(), "error code must be valid ascii"));
        }

        let Some(first_char) = code.chars().nth(1) else {
            return Err(Error::new(input.span(), "error code cannot be empty"));
        };

        if first_char.is_ascii_alphabetic() {
            return Err(Error::new(
                input.span(),
                "error code must start with a letter",
            ));
        }

        let ident = format_ident!("{}", code);

        Ok(quote! {
            const #ident: () = ();
        })
    }
}
