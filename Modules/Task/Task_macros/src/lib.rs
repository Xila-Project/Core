#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{ItemFn, parse_macro_input};

/// A procedural macro to annotate test functions.
///
/// This macro wraps the annotated async function to be executed in a blocking context
/// using embassy_futures::block_on, similar to how other modules handle async operations.
///
/// # Requirements
///
/// Test functions must:
/// - Be async
/// - Have no arguments
/// - Have no return type (or return unit type `()`)
///
/// # Usage
///
/// The macro accepts an optional path parameter to specify the Task module location:
///
/// Within the Task crate itself:
/// ```rust
/// #[Test] // Uses crate:: internally
/// async fn my_test() { ... }
/// ```
///
/// Outside the Task crate:
/// ```rust
/// #[Test("Task")] // Specify the module path
/// async fn my_test() { ... }
/// ```
///
/// You can also use any other path:
/// ```rust
/// #[Test("my_project::Task")]
/// async fn my_test() { ... }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Test(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input as a function
    let Input_function = parse_macro_input!(input as ItemFn);

    // Determine the module path from arguments
    let Task_path = if args.is_empty() {
        // Default: assume we're are not in the Task crate
        quote! { ::Task }
    } else {
        // Parse the path from the arguments
        let Path_string = args.to_string().trim_matches('"').to_string();
        let Path_identifier = format_ident!("{}", Path_string);
        quote! { #Path_identifier }
    };

    // Extract function details
    let Function_name = &Input_function.sig.ident;

    let Function_name_string = Function_name.to_string();

    // Check if function is async
    let Is_asynchronous = Input_function.sig.asyncness.is_some();

    if !Is_asynchronous {
        return syn::Error::new_spanned(
            Input_function.sig.fn_token,
            "Test functions must be async",
        )
        .to_compile_error()
        .into();
    }

    // Check if function has no arguments
    if !Input_function.sig.inputs.is_empty() {
        return syn::Error::new_spanned(
            &Input_function.sig.inputs,
            "Test functions must not have any arguments",
        )
        .to_compile_error()
        .into();
    }

    // Check if function has no return type (or returns unit type)
    if let syn::ReturnType::Type(_, Return_type) = &Input_function.sig.output {
        // Allow unit type () but reject any other return type
        if let syn::Type::Tuple(tuple) = Return_type.as_ref() {
            if !tuple.elems.is_empty() {
                return syn::Error::new_spanned(
                    Return_type,
                    "Test functions must not have a return type",
                )
                .to_compile_error()
                .into();
            }
        } else {
            return syn::Error::new_spanned(
                Return_type,
                "Test functions must not have a return type",
            )
            .to_compile_error()
            .into();
        }
    }

    // Change ident to __inner to avoid name conflicts
    let mut Input_function = Input_function.clone();
    Input_function.sig.ident = format_ident!("__inner");

    // Generate the new function
    quote! {
        #[test]
        fn #Function_name() {
            #Input_function

            static mut __Executor: Option<#Task_path::Executor_type> = None;
            static mut __Spawner : usize = 0;

            unsafe {
                if __Executor.is_none() {
                    __Executor = Some(#Task_path::Executor_type::New());
                }

                __Executor.as_mut().unwrap().Run(|Spawner| {
                    let Manager = #Task_path::Initialize();

                    unsafe {
                        __Spawner = Manager.Register_spawner(Spawner).expect("Failed to register spawner");
                    }

                    ::embassy_futures::block_on(async move {
                        Manager.Spawn(
                            #Task_path::Manager_type::Root_task_identifier,
                            #Function_name_string,
                            Some(__Spawner),
                            async move |_task| {
                                __inner().await;
                                __Executor.as_mut().unwrap().Stop();
                            }
                        ).await
                    }).expect("Failed to spawn task");
                });
            }
            unsafe {
                #Task_path::Initialize().Unregister_spawner(__Spawner).expect("Failed to unregister spawner");
            }

        }
    }
    .to_token_stream()
    .into()
}
