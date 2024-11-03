use quote::ToTokens;
use syn::LitStr;

pub fn convert(lit: LitStr) -> proc_macro2::TokenStream {
    // TODO: Convert `lit` from &str to &CStr
    lit.into_token_stream() // Temporary

    // let bytes = s.as_bytes();
    // let len = bytes.len();
    // let bytes = bytes.iter().map(|b| {
    //     let b = *b as u8;
    //     quote! { #b, }
    // });

    // quote! {
    //     unsafe {
    //         std::ffi::CStr::from_bytes_with_nul_unchecked(&[
    //             #(#bytes)*
    //             0,
    //         ])
    //     }
    // }
}
