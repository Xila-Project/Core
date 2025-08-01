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
//! - [`Manager`]: Global singleton that manages the WASM runtime and loaded modules
//! - [`Runtime_type`]: Represents a WASM runtime instance with registered host functions
//! - [`Module_type`]: Represents a loaded WASM module ready for instantiation
//! - [`Instance_type`]: An instantiated WASM module that can execute functions
//! - [`Environment_type`]: Execution environment providing context for function calls
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use virtual_machine::*;
//!
//! // Define host functions that WASM can call
//! struct MyRegistrable;
//! impl Registrable_trait for MyRegistrable {
//!     fn get_functions(&self) -> &[Function_descriptor_type] {
//!         &[Function_descriptor!(my_host_function)]
//!     }
//!     fn get_name(&self) -> &'static str { "my_module" }
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
#![allow(unused_imports)]
extern crate alloc;

mod custom_data;
mod environment;
mod error;
mod instance;
mod manager;
mod module;
mod registrable;
mod runtime;

// Re-export key types from WAMR
pub use wamr_rust_sdk::value::WasmValue;

// Re-export all public types from modules
pub use custom_data::*;
pub use environment::*;
pub use error::*;
pub use instance::*;
pub use manager::*;
pub use module::*;
pub use registrable::*;
pub use runtime::*;

/// Type alias for WASM pointer addresses in the 32-bit WASM address space
pub type WasmPointer = u32;

/// Type alias for WASM size values (32-bit)
pub type WasmUsize = u32;
