use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{ItemFn, parse_macro_input};

/// A procedural macro to annotate test functions.
///
/// This macro wraps the annotated async function to be executed in a blocking context
/// using embassy_futures::block_on, similar to how other modules handle async operations.
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Test(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input as a function
    let input_fn = parse_macro_input!(input as ItemFn);

    // Extract function details
    let fn_name = &input_fn.sig.ident;

    // Check if function is async
    let is_async = input_fn.sig.asyncness.is_some();

    if !is_async {
        return syn::Error::new_spanned(input_fn.sig.fn_token, "Test functions must be async")
            .to_compile_error()
            .into();
    }

    // Change ident to __inner to avoid name conflicts
    let mut input_fn = input_fn.clone();
    input_fn.sig.ident = format_ident!("__inner");

    // Generate the new function that uses embassy_futures::block_on
    quote! {

        #[test]
        fn #fn_name() {
            #input_fn

            let mut executor = ::embassy_executor::Executor::new();
            let executor = unsafe { ::core::mem::transmute::<_, &'static mut ::embassy_executor::Executor>(&mut executor) };
            executor.run(|spawner| {
                let Manager = crate::Initialize();

                Manager.Register_spawner(spawner.make_send());

                Manager.Spawn(
                    crate::Manager_type::Root_task_identifier,
                    "Root",
                    async move |_task|
                    {
                        __inner().await;

                        // Exit with libc::pthread_exit(0) to avoid hanging
                        #[cfg(target_os = "linux")]
                        {
                            println!("Exiting test thread");
                            unsafe { libc::pthread_exit(0 as *mut core::ffi::c_void) };
                        }
                    }
            ).unwrap();
            });
        }
    }.to_token_stream().into()
}
