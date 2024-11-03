use syn::{
    parse::{Parse, ParseStream},
    Attribute, Error, Expr, Ident, Token, Type, Visibility,
};

pub struct Parser {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub ty: Type,
    pub init: Expr,
}
impl Parse for Parser {
    fn parse(input: ParseStream<'_>) -> Result<Self, Error> {
        let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        input.parse::<Token![static]>()?;
        let ident: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        input.parse::<Token![=]>()?;
        let init: Expr = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(Parser {
            attrs,
            vis,
            ident,
            ty,
            init,
        })
    }
}
