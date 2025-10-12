



use std::{
    fs::{create_dir, read_dir, rename, OpenOptions},
    io::{Read, Write},
};

#[export_name = "gcd"]
pub fn gcd(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn test_stdio() -> Result<(), ()> {
    println!("Test stdout");
    eprintln!("Test stderr");

    let mut input = String::new();

    std::io::stdin().read_line(&mut input).unwrap();

    println!("Input: {}", input);

    Ok(())
}

fn test_file() {
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("/test.txt")
            .unwrap();

        file.write_all(b"Hello World from WASM!").unwrap();
    }

    {
        let mut file = OpenOptions::new().read(true).open("/test.txt").unwrap();

        let mut string = String::new();

        file.read_to_string(&mut string).unwrap();

        assert_eq!(string, "Hello World from WASM!");
    }

    {
        rename("/test.txt", "/test2.txt").unwrap();

        let mut file = OpenOptions::new().read(true).open("/test2.txt").unwrap();

        let mut string = String::new();

        file.read_to_string(&mut string).unwrap();

        assert_eq!(string, "Hello World from WASM!");
    }
}

fn test_environment_variables() {
    println!("Environment variables:");

    let environment = std::env::vars();

    for (key, value) in environment {
        println!("{}: {}", key, value);
    }
}

fn test_directory() {
    create_dir("/test_dir").unwrap();

    for entry in read_dir("/").unwrap() {
        let entry = entry.unwrap();

        let r#type = entry.file_type().unwrap();

        let r#type = if r#type.is_dir() {
            "Directory"
        } else if r#type.is_file() {
            "File"
        } else if r#type.is_symlink() {
            "Symlink"
        } else {
            "Unknown"
        };

        println!("{:?} - {}", entry.file_name(), r#type);
    }
}

/// Allocate memory
/// 
/// # Safety
/// 
/// This function is unsafe because it may return an invalid pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Allocate(size: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align(size, std::mem::size_of::<usize>()).unwrap();

    std::alloc::alloc(layout)
}

/// Deallocate memory
/// 
/// # Safety
/// 
/// This function is unsafe because it may cause undefined behavior if the pointer is invalid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Deallocate(pointer: *mut u8, size: usize) {
    let layout = std::alloc::Layout::from_size_align(size, std::mem::size_of::<usize>()).unwrap();

    std::alloc::dealloc(pointer, layout)
}

fn main() -> Result<(), ()> {
    test_stdio()?;

    test_file();

    test_directory();

    test_environment_variables();

    Ok(())
}
