use macros_lib::{proc_macro2, syn};
use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    Error, ItemFn,
};

pub struct Parser {
    pub r#fn: ItemFn,
    pub input: TokenStream,
}
impl Parse for Parser {
    fn parse(input: ParseStream<'_>) -> Result<Self, Error> {
        let r#fn = input.parse()?;
        let input = input.parse()?;
        let parser = Self { r#fn, input };
        Ok(parser)
    }
}
