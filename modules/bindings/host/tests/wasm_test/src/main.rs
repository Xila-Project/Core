



use wasm_bindings::*;

use std::thread::sleep;

fn main() {
    println!("Hello, world!");

    let mut window = Xila_graphics_object_t::MAX;
    let mut button = Xila_graphics_object_t::MAX;
    let mut label = Xila_graphics_object_t::MAX;

    unsafe {
        println!("Window : {:x}", window);

        Xila_graphics_window_create(&mut window as *mut _);

        println!("Window : {:x}", window);

        Xila_graphics_button_create(window, &mut button as *mut _);

        println!("Button : {:x}", button);

        Xila_graphics_label_create(button, &mut label as *mut _);
        Xila_graphics_label_set_text(label, c"Hello, world!".as_ptr() as *mut _);

        println!("Label : {:x}", label);

        loop {
            let mut Code = Xila_graphics_event_code_t_LV_EVENT_ALL;

            Xila_graphics_window_get_event_code(window, &mut Code as *mut _);

            if Code != Xila_graphics_event_code_t_LV_EVENT_ALL {
                match Code {
                    Xila_graphics_event_code_t_LV_EVENT_CLICKED => {
                        println!("Button pressed!");

                        let mut Target = Xila_graphics_object_t::MAX;

                        Xila_graphics_window_get_event_target(window, &mut Target as *mut _);

                        if Target == button {
                            Xila_graphics_label_set_text(
                                label,
                                c"Button pressed!".as_ptr() as *mut _,
                            );
                        }
                    }
                    Event => {
                        println!("Event : {}", Event);
                    }
                }

                Xila_graphics_window_next_event(window);
            }

            sleep(std::time::Duration::from_millis(10));
        }
    }
}
