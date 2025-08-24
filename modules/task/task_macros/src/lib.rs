use darling::{FromMeta, ast::NestedMeta};
use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{ItemFn, parse_macro_input, parse_str};

fn default_task_path() -> syn::Expr {
    parse_str("task").unwrap()
}

fn default_executor() -> syn::Expr {
    parse_str("drivers::standard_library::executor::instantiate_static_executor!()").unwrap()
}

#[derive(Debug, FromMeta, Clone)]
struct TaskArguments {
    #[darling(default = "default_task_path")]
    pub task_path: syn::Expr,

    #[darling(default = "default_executor")]
    pub executor: syn::Expr,
}

impl TaskArguments {
    fn from_token_stream(arguments: TokenStream) -> Result<Self, darling::Error> {
        let arguments = NestedMeta::parse_meta_list(arguments.into()).unwrap();
        Self::from_list(&arguments.clone())
    }
}

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
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn test(arguments: TokenStream, input: TokenStream) -> TokenStream {
    let arguments = match TaskArguments::from_token_stream(arguments) {
        Ok(o) => o,
        Err(e) => return e.write_errors().into(),
    };
    let input_function = parse_macro_input!(input as ItemFn);

    let executor = arguments.executor;
    let task_path = arguments.task_path;

    // Extract function details
    let function_name = &input_function.sig.ident;

    let function_name_string = function_name.to_string();

    // Check if function is async
    let is_asynchronous = input_function.sig.asyncness.is_some();

    if !is_asynchronous {
        return syn::Error::new_spanned(
            input_function.sig.fn_token,
            "Test functions must be async",
        )
        .to_compile_error()
        .into();
    }

    // Check if function has no arguments
    if !input_function.sig.inputs.is_empty() {
        return syn::Error::new_spanned(
            &input_function.sig.inputs,
            "Test functions must not have any arguments",
        )
        .to_compile_error()
        .into();
    }

    // Check if function has no return type (or returns unit type)
    if let syn::ReturnType::Type(_, return_type) = &input_function.sig.output {
        // Allow unit type () but reject any other return type
        if let syn::Type::Tuple(tuple) = return_type.as_ref() {
            if !tuple.elems.is_empty() {
                return syn::Error::new_spanned(
                    return_type,
                    "Test functions must not have a return type",
                )
                .to_compile_error()
                .into();
            }
        } else {
            return syn::Error::new_spanned(
                return_type,
                "Test functions must not have a return type",
            )
            .to_compile_error()
            .into();
        }
    }

    // Change ident to __inner to avoid name conflicts
    let mut Input_function = input_function.clone();
    Input_function.sig.ident = format_ident!("__inner");

    // Generate the new function
    quote! {
        #[std::prelude::v1::test]
        fn #function_name() {
            #Input_function

            static mut __SPAWNER : usize = 0;

            unsafe {
                let __EXECUTOR = #executor;

                __EXECUTOR.run(|Spawner, __executor| {
                    let manager = #task_path::initialize();

                    unsafe {
                        __SPAWNER = manager.register_spawner(Spawner).expect("Failed to register spawner");
                    }

                    #task_path::futures::block_on(async move {
                        manager.spawn(
                            #task_path::Manager::ROOT_TASK_IDENTIFIER,
                            #function_name_string,
                            Some(__SPAWNER),
                            async move |_task| {
                                __inner().await;
                                __executor.stop();
                            }
                        ).await
                    }).expect("Failed to spawn task");
                });
            }
            unsafe {
                #task_path::get_instance().unregister_spawner(__SPAWNER).expect("Failed to unregister spawner");
            }

        }
    }
    .to_token_stream()
    .into()
}

/// A procedural macro to annotate functions that should run with a specific executor.
///
/// This macro wraps the annotated async function to be executed with a provided
/// executor, handling the registration, spawning, and cleanup automatically.
///
/// # Requirements
///
/// Functions must:
/// - Be async
/// - Have no arguments
/// - Have no return type (or return unit type `()`)
///
/// # Usage
///
/// The macro accepts an executor expression as a parameter:
///
/// ```rust
/// #[Run_with_executor(drivers::standard_library::Executor::Executor_type::new())]
/// async fn my_function() {
///     println!("Running with custom executor!");
/// }
/// ```
///
/// You can also use any executor expression:
/// ```rust
/// #[Run_with_executor(my_custom_executor)]
/// async fn my_function() { ... }
/// ```
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn run(Arguments: TokenStream, Input: TokenStream) -> TokenStream {
    let Arguments = match TaskArguments::from_token_stream(Arguments) {
        Ok(o) => o,
        Err(e) => return e.write_errors().into(),
    };
    let Input_function = parse_macro_input!(Input as ItemFn);

    let Task_path = Arguments.task_path;
    let Executor_expression = Arguments.executor;

    // Extract function details
    let Function_name = &Input_function.sig.ident;
    let Function_name_string = Function_name.to_string();

    // Check if function is async
    let is_asynchronous = Input_function.sig.asyncness.is_some();

    if !is_asynchronous {
        return syn::Error::new_spanned(
            Input_function.sig.fn_token,
            "Functions with Run_with_executor must be async",
        )
        .to_compile_error()
        .into();
    }

    // Check if function has no arguments
    if !Input_function.sig.inputs.is_empty() {
        return syn::Error::new_spanned(
            &Input_function.sig.inputs,
            "Functions with Run_with_executor must not have any arguments",
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
                    "Functions with Run_with_executor must not have a return type",
                )
                .to_compile_error()
                .into();
            }
        } else {
            return syn::Error::new_spanned(
                Return_type,
                "Functions with Run_with_executor must not have a return type",
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
        fn #Function_name() {
            #Input_function

            static mut __SPAWNER : usize = 0;

            unsafe {
                let __EXECUTOR : &'static mut _ = #Executor_expression;

                __EXECUTOR.start(|Spawner| {
                    let manager = #Task_path::initialize();

                    unsafe {
                        __SPAWNER = manager.register_spawner(Spawner).expect("Failed to register spawner");
                    }

                    #Task_path::futures::block_on(async move {
                        manager.spawn(
                            #Task_path::Manager::ROOT_TASK_IDENTIFIER,
                            #Function_name_string,
                            Some(__SPAWNER),
                            async move |_task| {
                                __inner().await;
                            }
                        ).await
                    }).expect("Failed to spawn task");
                });
            }
        }
    }
    .to_token_stream()
    .into()
}
