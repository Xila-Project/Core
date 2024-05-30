#![allow(non_snake_case)]

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

fn main() -> Result<(), ()> {
    Test_passing_mutable_slice()?;

    Test_passing_slice()?;

    Test_passing_mutable_string()?;

    Test_passing_string()?;

    Ok(())
}
