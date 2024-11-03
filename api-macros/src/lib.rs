mod func_struct;

mod static_parser;
mod str_to_cstr;
use static_parser::Parser;

mod struct_literal;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Error, Expr, Path};

type HandledStruct =
    Result<(Vec<Attribute>, Path, proc_macro2::TokenStream), TokenStream>;
#[proc_macro]
pub fn struct_c(input: TokenStream) -> TokenStream {
    let Parser {
        attrs,
        vis,
        ident,
        ty,
        init,
    } = parse_macro_input!(input as Parser);
    let base = quote! {
        #(#attrs)*
        #vis static #ident: #ty =
    };

    let res = if let Expr::Struct(init) = init {
        struct_literal::handler(init)
    } else if let Expr::Call(init) = init {
        func_struct::handler(init)
    } else {
        let err = Error::new_spanned(init, "expected a struct");
        Err(err.to_compile_error().into())
    };
    let (expr_attrs, path, defs) = match res {
        Ok(ok) => ok,
        Err(err) => return err,
    };

    let expanded = quote! {
        #base
            #(#expr_attrs)*
            #path #defs;
    };

    TokenStream::from(expanded)
}
