//! # Virtual Machine Module
//!
//! This crate provides a WebAssembly (WASM) virtual machine implementation for the Xila operating system.
//! It is built on top of the WAMR (WebAssembly Micro Runtime) and provides a high-level interface
//! for executing WASM modules in a no_std environment.
//!
//! ## Features
//!
//! - **Module Management**: Load, instantiate, and execute WASM modules
//! - **Runtime Environment**: Provides isolated execution environments for WASM instances
//! - **Host Function Registration**: Register native functions that can be called from WASM
//! - **Memory Management**: Safe pointer conversion between WASM and native address spaces
//! - **Error Handling**: Comprehensive error types for debugging and error recovery
//! - **XIP Support**: Execute-in-place for AOT compiled modules
//! - **WASI Integration**: WebAssembly System Interface support with custom I/O redirection
//!
//! ## Architecture
//!
//! The crate is organized into several key components:
//!
//! - [`Manager_type`]: Global singleton that manages the WASM runtime and loaded modules
//! - [`Runtime_type`]: Represents a WASM runtime instance with registered host functions
//! - [`Module_type`]: Represents a loaded WASM module ready for instantiation
//! - [`Instance_type`]: An instantiated WASM module that can execute functions
//! - [`Environment_type`]: Execution environment providing context for function calls
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use Virtual_machine::*;
//!
//! // Define host functions that WASM can call
//! struct MyRegistrable;
//! impl Registrable_trait for MyRegistrable {
//!     fn Get_functions(&self) -> &[Function_descriptor_type] {
//!         &[Function_descriptor!(my_host_function)]
//!     }
//!     fn Get_name(&self) -> &'static str { "my_module" }
//! }
//!
//! // Initialize the virtual machine manager
//! let manager = Initialize(&[&MyRegistrable]);
//!
//! // Execute a WASM module
//! let result = manager.Execute(
//!     wasm_bytes,
//!     stack_size,
//!     stdin_fd,
//!     stdout_fd,
//!     stderr_fd
//! ).await;
//! ```

#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
extern crate alloc;

mod Custom_data;
mod Environment;
mod Error;
mod Instance;
mod Manager;
mod Module;
mod Registrable;
mod Runtime;

// Re-export key types from WAMR
pub use wamr_rust_sdk::value::WasmValue;

// Re-export all public types from modules
pub use Custom_data::*;
pub use Environment::*;
pub use Error::*;
pub use Instance::*;
pub use Manager::*;
pub use Module::*;
pub use Registrable::*;
pub use Runtime::*;

/// Type alias for WASM pointer addresses in the 32-bit WASM address space
pub type WASM_pointer_type = u32;

/// Type alias for WASM size values (32-bit)
pub type WASM_usize_type = u32;
