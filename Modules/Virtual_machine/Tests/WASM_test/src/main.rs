#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::{
    fs::{create_dir, read_dir, rename, OpenOptions},
    io::{Read, Write},
};

#[link(wasm_import_module = "host")]
extern "C" {
    pub fn Test_mutable_slice(Slice: *mut i32, Size: usize);
    pub fn Test_slice(Slice: *const i32, Length: usize);
    pub fn Test_mutable_string(String: *mut u8, Length: *mut usize, Size: usize);
    pub fn Test_string(String: *const u8, Length: usize);
}

#[export_name = "GCD"]
pub fn GCD(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn Test_passing_mutable_slice() -> Result<(), ()> {
    let mut Slice = [9, 8, 7, 6, 5, 4, 3, 2, 1, 0];

    unsafe {
        Test_mutable_slice(Slice.as_mut_ptr(), Slice.len());
    }

    if Slice != [42; 10] {
        return Err(());
    }

    Ok(())
}

fn Test_passing_slice() -> Result<(), ()> {
    let Slice = [9, 8, 7, 6, 5, 4, 3, 2, 1, 0];

    unsafe {
        Test_slice(Slice.as_ptr(), Slice.len());
    }

    Ok(())
}

fn Test_passing_mutable_string() -> Result<(), ()> {
    let mut String = String::from("Hello");

    String.reserve(40);

    let mut String_vector = String.into_bytes();

    let mut String_length = String_vector.len();

    unsafe {
        Test_mutable_string(
            String_vector.as_mut_ptr(),
            &mut String_length as *mut usize,
            String_vector.capacity(),
        );
        String_vector.set_len(String_length);
    }

    let String = unsafe { String::from_utf8_unchecked(String_vector) };

    if String != "Hello World from WASM!" {
        return Err(());
    }

    Ok(())
}

fn Test_passing_string() -> Result<(), ()> {
    let String = "Hello World from WASM!".as_bytes();

    unsafe {
        Test_string(String.as_ptr(), String.len());
    }

    Ok(())
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

fn main() -> Result<(), ()> {
    Test_passing_mutable_slice()?;

    Test_passing_slice()?;

    Test_passing_mutable_string()?;

    Test_passing_string()?;

    Test_stdio()?;

    Test_file();

    Test_directory();

    Test_environment_variables();

    Ok(())
}
