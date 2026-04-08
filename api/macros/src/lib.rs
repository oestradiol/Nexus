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
        use nexus_api::{RuntimeRef, Plugin};
        use std::sync::OnceLock;

        #[expect(unsafe_code)]
        #[unsafe(no_mangle)]
        #r#struct

        struct Instance {
            runtime: OnceLock<RuntimeRef>,
        }

        impl Instance {
            fn runtime(&self) -> &RuntimeRef {
                self.runtime.get().expect("Plugin runtime not initialized. Call init() before main().")
            }

            /// Spawn a task on the runtime
            pub fn spawn<F>(&self, future: F) -> nexus_api::task::JoinHandle<()>
            where
                F: std::future::Future<Output = ()> + Send + 'static,
            {
                self.runtime().spawn(Box::pin(future))
            }

            /// Sleep for a duration
            pub fn sleep(&self, duration: nexus_api::Duration) -> nexus_api::Sleep {
                self.runtime().sleep(duration)
            }

            /// Get current instant
            pub fn now(&self) -> nexus_api::Instant {
                self.runtime().now()
            }
        }

        #[nexus_api::async_trait]
        impl Plugin for Instance {
            fn init(&mut self, runtime: RuntimeRef) {
                self.runtime.set(runtime).expect("Runtime already initialized");
            }

            #input
        }

        #[expect(clippy::no_mangle_with_rust_abi)]
        #[expect(unsafe_code)]
        #[unsafe(no_mangle)]
        pub extern "Rust" fn _new_rust_impl() -> Box<dyn Plugin> {
            Box::new(Instance {
                runtime: OnceLock::new(),
            })
        }
    };

    patched.into()
}
