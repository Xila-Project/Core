
#![allow(non_camel_case_types)]


use wasm_bindings::*;

use std::thread::sleep;

fn main() {
    println!("Hello, world!");

    let mut Window = Xila_graphics_object_t::MAX;
    let mut Button = Xila_graphics_object_t::MAX;
    let mut Label = Xila_graphics_object_t::MAX;

    unsafe {
        println!("Window : {:x}", Window);

        Xila_graphics_window_create(&mut Window as *mut _);

        println!("Window : {:x}", Window);

        Xila_graphics_button_create(Window, &mut Button as *mut _);

        println!("Button : {:x}", Button);

        Xila_graphics_label_create(Button, &mut Label as *mut _);
        Xila_graphics_label_set_text(Label, c"Hello, world!".as_ptr() as *mut _);

        println!("Label : {:x}", Label);

        loop {
            let mut Code = Xila_graphics_event_code_t_LV_EVENT_ALL;

            Xila_graphics_window_get_event_code(Window, &mut Code as *mut _);

            if Code != Xila_graphics_event_code_t_LV_EVENT_ALL {
                match Code {
                    Xila_graphics_event_code_t_LV_EVENT_CLICKED => {
                        println!("Button pressed!");

                        let mut Target = Xila_graphics_object_t::MAX;

                        Xila_graphics_window_get_event_target(Window, &mut Target as *mut _);

                        if Target == Button {
                            Xila_graphics_label_set_text(
                                Label,
                                c"Button pressed!".as_ptr() as *mut _,
                            );
                        }
                    }
                    Event => {
                        println!("Event : {}", Event);
                    }
                }

                Xila_graphics_window_next_event(Window);
            }

            sleep(std::time::Duration::from_millis(10));
        }
    }
}
