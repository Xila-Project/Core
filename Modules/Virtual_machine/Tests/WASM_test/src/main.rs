#![allow(non_snake_case)]

#[link(wasm_import_module = "host")]
extern "C" {
    pub fn Test_mutable_slice(Slice: *mut i32, Length: *mut usize, Size: usize);
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
    let mut Vector = vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0];

    Vector.reserve(1);

    let mut Vector_length = Vector.len();

    unsafe {
        Test_mutable_slice(
            Vector.as_mut_ptr(),
            &mut Vector_length as *mut usize,
            Vector.capacity(),
        );
        Vector.set_len(Vector_length);
    }

    if Vector != vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 42] {
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
    let mut String = Vec::new();

    String.reserve(40);

    let mut String_length = String.len();

    unsafe {
        Test_mutable_string(
            String.as_mut_ptr(),
            &mut String_length as *mut usize,
            String.capacity(),
        );
        String.set_len(String_length);
    }

    if String != "Hello".as_bytes() {
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

fn main() -> Result<(), ()> {
    Test_passing_mutable_slice()?;

    Test_passing_slice()?;

    Test_passing_mutable_string()?;

    Test_passing_string()?;

    Ok(())
}
