use quote::{quote, ToTokens};
use syn::{Expr, ExprLit, ExprStruct, FieldValue, Lit};

use crate::{str_to_cstr, HandledStruct};

pub fn handler(init: ExprStruct) -> HandledStruct {
    let ExprStruct {
        attrs: expr_attrs,
        path,
        fields,
        rest,
        dot2_token,
        ..
    } = init;

    let fields = fields.into_iter().map(|f| match f {
        FieldValue {
            attrs,
            member,
            colon_token,
            expr:
                Expr::Lit(ExprLit {
                    attrs: lit_attrs,
                    lit: Lit::Str(lit),
                }),
        } => {
            let lit = str_to_cstr::convert(lit);
            quote! {
                #(#attrs)*
                #member #colon_token #(#lit_attrs)* #lit,
            }
        }
        f => f.into_token_stream(),
    });

    Ok((
        expr_attrs,
        path,
        quote! { {
            #(#fields,)*
            #dot2_token #rest
        } },
    ))
}
