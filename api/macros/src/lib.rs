use macros_lib::{c_struct, quote};
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn plugin(input: TokenStream) -> TokenStream {
    let (r#struct, input) = match c_struct(input.into()) {
        Ok(ok) => ok,
        Err(e) => return e.into(),
    };

    let patched = quote! {
        use nexus_api::Plugin;

        #[expect(unsafe_code)]
        #[unsafe(no_mangle)]
        #r#struct

        struct Instance;
        #[nexus_api::async_trait]
        impl Plugin for Instance {
            #input
        }

        #[expect(clippy::no_mangle_with_rust_abi)]
        #[expect(unsafe_code)]
        #[unsafe(no_mangle)]
        pub extern "Rust" fn _new_rust_impl(
        //    logger: std::sync::Arc<dyn tracing::Subscriber + Send + Sync>,
        ) -> Box<dyn Plugin> {
        //    tracing::subscriber::set_global_default(logger).unwrap();
            Box::new(Instance)
        }
    };

    patched.into()
}
