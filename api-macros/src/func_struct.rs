use quote::{quote, ToTokens};
use syn::{Error, Expr, ExprCall, ExprLit, ExprPath, Lit};

use crate::{str_to_cstr, HandledStruct};

pub fn handler(init: ExprCall) -> HandledStruct {
    let ExprCall {
        attrs: mut expr_attrs,
        func,
        args,
        ..
    } = init;

    let ExprPath { attrs, path, .. } = match *func {
        Expr::Path(path) => path,
        _ => {
            let err = Error::new_spanned(func, "expected path");
            return Err(err.to_compile_error().into());
        }
    };
    expr_attrs.extend(attrs);

    let args = args.into_iter().map(|f| match f {
        Expr::Lit(ExprLit {
            attrs,
            lit: Lit::Str(lit),
        }) => {
            let lit = str_to_cstr::convert(lit);
            quote! { #(#attrs)* #lit }
        }
        _ => f.into_token_stream(),
    });

    Ok((
        expr_attrs,
        path,
        quote! { (
            #(#args,)*
        ) },
    ))
}
