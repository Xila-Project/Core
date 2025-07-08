use core::ptr::null_mut;

use Graphics::LVGL;

use crate::{Desk::Desk_type, Error::Result_type};

pub struct Home_type {
    button: *mut LVGL::lv_obj_t,
}

impl Drop for Home_type {
    fn drop(&mut self) {
        unsafe {
            LVGL::lv_obj_delete_async(self.button);
        }
    }
}

impl Home_type {
    pub async fn new(desk: *mut LVGL::lv_obj_t) -> Result_type<Self> {
        let _lock = Graphics::get_instance().lock().await; // Lock the graphics

        let button = unsafe {
            let button = LVGL::lv_obj_create(LVGL::lv_layer_top());

            if button.is_null() {
                return Err(crate::Error::Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_size(button, LVGL::lv_pct(40), 8);
            LVGL::lv_obj_set_style_bg_color(button, LVGL::lv_color_white(), LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_bg_opa(button, LVGL::LV_OPA_50 as u8, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_align(button, LVGL::lv_align_t_LV_ALIGN_BOTTOM_MID);
            LVGL::lv_obj_set_y(button, -5);

            LVGL::lv_obj_set_style_pad_all(button, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_border_width(button, 0, LVGL::LV_STATE_DEFAULT);

            LVGL::lv_obj_remove_flag(button, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_GESTURE_BUBBLE);

            LVGL::lv_obj_add_event_cb(
                button,
                Some(handle_pressing),
                LVGL::lv_event_code_t_LV_EVENT_PRESSING,
                null_mut(),
            );

            LVGL::lv_obj_add_event_cb(
                button,
                Some(handle_released),
                LVGL::lv_event_code_t_LV_EVENT_RELEASED,
                desk as *mut core::ffi::c_void,
            );

            button
        };

        Ok(Self { button })
    }
}

unsafe extern "C" fn handle_pressing(event: *mut LVGL::lv_event_t) {
    let object = LVGL::lv_event_get_target(event) as *mut LVGL::lv_obj_t;

    let input_device = LVGL::lv_indev_active();

    let mut vector = LVGL::lv_point_t::default();

    LVGL::lv_indev_get_vect(input_device, &mut vector as *mut LVGL::lv_point_t);

    let y = LVGL::lv_obj_get_y_aligned(object) + vector.y;

    let y = y.max(-40);

    LVGL::lv_obj_set_y(object, y);
}

unsafe extern "C" fn handle_released(event: *mut LVGL::lv_event_t) {
    let object = LVGL::lv_event_get_target(event) as *mut LVGL::lv_obj_t;

    let y = LVGL::lv_obj_get_y_aligned(object);

    LVGL::lv_obj_set_y(object, -5);

    if y < -20 {
        let desk = LVGL::lv_event_get_user_data(event) as *mut LVGL::lv_obj_t;

        LVGL::lv_obj_remove_flag(desk, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_HIDDEN);
        LVGL::lv_obj_move_foreground(desk);

        LVGL::lv_obj_send_event(desk, Desk_type::HOME_EVENT.into_lvgl_code(), null_mut());
    }
}
