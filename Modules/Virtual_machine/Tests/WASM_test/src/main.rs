#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::{
    fs::{create_dir, read_dir, rename, OpenOptions},
    io::{Read, Write},
};

#[export_name = "GCD"]
pub fn GCD(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn Test_stdio() -> Result<(), ()> {
    println!("Test stdout");
    eprintln!("Test stderr");

    let mut Input = String::new();

    std::io::stdin().read_line(&mut Input).unwrap();

    println!("Input: {}", Input);

    Ok(())
}

fn Test_file() {
    {
        let mut File = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("/test.txt")
            .unwrap();

        File.write_all(b"Hello World from WASM!").unwrap();
    }

    {
        let mut File = OpenOptions::new().read(true).open("/test.txt").unwrap();

        let mut String = String::new();

        File.read_to_string(&mut String).unwrap();

        assert_eq!(String, "Hello World from WASM!");
    }

    {
        rename("/test.txt", "/test2.txt").unwrap();

        let mut File = OpenOptions::new().read(true).open("/test2.txt").unwrap();

        let mut String = String::new();

        File.read_to_string(&mut String).unwrap();

        assert_eq!(String, "Hello World from WASM!");
    }
}

fn Test_environment_variables() {
    println!("Environment variables:");

    let Environment = std::env::vars();

    for (Key, Value) in Environment {
        println!("{}: {}", Key, Value);
    }
}

fn Test_directory() {
    create_dir("/test_dir").unwrap();

    for Entry in read_dir("/").unwrap() {
        let Entry = Entry.unwrap();

        let Type = Entry.file_type().unwrap();

        let Type = if Type.is_dir() {
            "Directory"
        } else if Type.is_file() {
            "File"
        } else if Type.is_symlink() {
            "Symlink"
        } else {
            "Unknown"
        };

        println!("{:?} - {}", Entry.file_name(), Type);
    }
}

/// Allocate memory
/// 
/// # Safety
/// 
/// This function is unsafe because it may return an invalid pointer.
#[no_mangle]
pub unsafe extern "C" fn Allocate(Size: usize) -> *mut u8 {
    let Layout = std::alloc::Layout::from_size_align(Size, std::mem::size_of::<usize>()).unwrap();

    std::alloc::alloc(Layout)
}

/// Deallocate memory
/// 
/// # Safety
/// 
/// This function is unsafe because it may cause undefined behavior if the pointer is invalid.
#[no_mangle]
pub unsafe extern "C" fn Deallocate(Pointer: *mut u8, Size: usize) {
    let Layout = std::alloc::Layout::from_size_align(Size, std::mem::size_of::<usize>()).unwrap();

    std::alloc::dealloc(Pointer, Layout)
}

fn main() -> Result<(), ()> {
    Test_stdio()?;

    Test_file();

    Test_directory();

    Test_environment_variables();

    Ok(())
}
