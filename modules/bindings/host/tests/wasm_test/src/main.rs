#[cfg(target_arch = "wasm32")]
fn main() {
    use std::thread::sleep;

    println!("Hello, world!");

    unsafe {
        let window = wasm_bindings::window_create().unwrap();

        let button = wasm_bindings::button_create(window).unwrap();

        let label = wasm_bindings::label_create(button).unwrap();
        wasm_bindings::label_set_text(label, c"Hello, world!".as_ptr() as *mut _).unwrap();

        loop {
            let mut code = wasm_bindings::EventCode::All;
            let mut target: *mut wasm_bindings::Object = core::ptr::null_mut();

            let _ = wasm_bindings::window_pop_event(
                window,
                &mut code as *mut _ as *mut _,
                &mut target as *mut _ as *mut _,
            );

            match code {
                wasm_bindings::EventCode::Clicked => {
                    println!("Button pressed!");

                    if target == button {
                        wasm_bindings::label_set_text(label, c"Button pressed!".as_ptr() as *mut _)
                            .unwrap();
                    }
                }
                wasm_bindings::EventCode::All => {}
                event => {
                    println!("Event : {:?}", event);
                }
            }

            sleep(std::time::Duration::from_millis(10));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("This test is only for the wasm32 target.");
}
