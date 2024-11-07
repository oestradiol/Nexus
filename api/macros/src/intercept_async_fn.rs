use macros_lib::{
    proc_macro2,
    quote::{self, ToTokens},
    syn::{self, spanned::Spanned, Item, ItemFn, Signature},
};
use quote::quote;

pub fn all(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let (Ok(merged) | Err(merged)) = syn::parse2::<syn::File>(input)
        .map(|f| {
            let intercepted = f.items.into_iter().map(|i| match i {
                Item::Fn(r#fn) => one(r#fn),
                _ => i.into_token_stream(),
            });
            quote! {
                #(#intercepted)*
            }
        })
        .map_err(|e| e.to_compile_error());
    merged
}

pub fn one(
    ItemFn {
        attrs,
        vis,
        sig:
            Signature {
                constness,
                mut asyncness,
                unsafety,
                abi,
                fn_token,
                ident,
                generics,
                inputs,
                variadic,
                output,
                ..
            },
        block,
    }: ItemFn,
) -> macros_lib::proc_macro2::TokenStream {
    if asyncness.is_some() {
        if let Some(constness) = constness {
            return syn::Error::new(
                constness.span(),
                "functions cannot be both `const` and `async`",
            )
            .to_compile_error();
        }

        asyncness = None;
        let qt = quote! {
            #(#attrs)*
            #vis #constness #asyncness #unsafety #abi #fn_token #ident #generics
        };

        let ty = match output {
            syn::ReturnType::Default => quote! { () },
            syn::ReturnType::Type(_, ty) => quote! { #ty },
        };

        quote! {
            #qt (#inputs #variadic) -> tokio::task::JoinHandle<#ty> {
                let body = async #block;
                self.runtime.spawn(body)
            }
        }
    } else {
        quote! {
            #(#attrs)*
            #vis #constness #asyncness #unsafety #abi #fn_token #ident #generics (#inputs #variadic) #output #block
        }
    }
}
