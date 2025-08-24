use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::HtmlCanvasElement;

use crate::wasm::devices::graphics::get_window_size;

pub fn new() -> Result<HtmlCanvasElement, String> {
    let window = web_sys::window().ok_or("Failed to get window")?;

    let document = window.document().ok_or("Failed to get document")?;

    let body = document.body().ok_or("Failed to get body")?;

    let canvas: HtmlCanvasElement = document
        .create_element("canvas")
        .map_err(|_| "Failed to create canvas")?
        .dyn_into()
        .map_err(|_| "Failed to cast to HtmlCanvasElement")?;

    let (width, height) = get_window_size(&window).ok_or("Failed to get window size")?;

    // Set body margin to 0 to avoid scrollbars
    body.set_attribute("style", "margin: 0")
        .map_err(|_| "Failed to set body margin")?;

    body.append_child(&canvas)
        .map_err(|_| "Failed to append canvas to body")?;

    canvas.set_id("canvas-screen");
    canvas.set_width(width);
    canvas.set_height(height);

    let canvas_clone = canvas.clone();
    let window_clone = window.clone();
    let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
        if let Some((width, height)) = get_window_size(&window_clone) {
            canvas_clone.set_width(width);
            canvas_clone.set_height(height);
        } else {
            log::error!("Failed to get window size");
        }
    }) as Box<dyn FnMut(_)>);

    window
        .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
        .unwrap();

    closure.forget(); // Prevent memory leak by keeping the closure alive
    Ok(canvas)
}
