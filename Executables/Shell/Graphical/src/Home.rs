use core::ptr::null_mut;

use Graphics::LVGL;

use crate::{Desk::Desk_type, Error::Result_type};

pub struct Home_type {
    Button: *mut LVGL::lv_obj_t,
}

impl Drop for Home_type {
    fn drop(&mut self) {
        unsafe {
            LVGL::lv_obj_delete_async(self.Button);
        }
    }
}

impl Home_type {
    pub async fn New(Desk: *mut LVGL::lv_obj_t) -> Result_type<Self> {
        let _Lock = Graphics::Get_instance().Lock().await; // Lock the graphics

        let Button = unsafe {
            let Button = LVGL::lv_obj_create(LVGL::lv_layer_top());

            if Button.is_null() {
                return Err(crate::Error::Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_size(Button, LVGL::lv_pct(40), 8);
            LVGL::lv_obj_set_style_bg_color(Button, LVGL::lv_color_white(), LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_bg_opa(Button, LVGL::LV_OPA_50 as u8, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_align(Button, LVGL::lv_align_t_LV_ALIGN_BOTTOM_MID);
            LVGL::lv_obj_set_y(Button, -5);

            LVGL::lv_obj_set_style_pad_all(Button, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_border_width(Button, 0, LVGL::LV_STATE_DEFAULT);

            LVGL::lv_obj_remove_flag(Button, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_GESTURE_BUBBLE);

            LVGL::lv_obj_add_event_cb(
                Button,
                Some(Handle_pressing),
                LVGL::lv_event_code_t_LV_EVENT_PRESSING,
                null_mut(),
            );

            LVGL::lv_obj_add_event_cb(
                Button,
                Some(Handle_released),
                LVGL::lv_event_code_t_LV_EVENT_RELEASED,
                Desk as *mut core::ffi::c_void,
            );

            Button
        };

        Ok(Self { Button })
    }
}

unsafe extern "C" fn Handle_pressing(Event: *mut LVGL::lv_event_t) {
    let Object = LVGL::lv_event_get_target(Event) as *mut LVGL::lv_obj_t;

    let Input_device = LVGL::lv_indev_active();

    let mut Vector = LVGL::lv_point_t::default();

    LVGL::lv_indev_get_vect(Input_device, &mut Vector as *mut LVGL::lv_point_t);

    let y = LVGL::lv_obj_get_y_aligned(Object) + Vector.y;

    let y = y.max(-40);

    LVGL::lv_obj_set_y(Object, y);
}

unsafe extern "C" fn Handle_released(Event: *mut LVGL::lv_event_t) {
    let Object = LVGL::lv_event_get_target(Event) as *mut LVGL::lv_obj_t;

    let y = LVGL::lv_obj_get_y_aligned(Object);

    LVGL::lv_obj_set_y(Object, -5);

    if y < -20 {
        let Desk = LVGL::lv_event_get_user_data(Event) as *mut LVGL::lv_obj_t;

        LVGL::lv_obj_remove_flag(Desk, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_HIDDEN);
        LVGL::lv_obj_move_foreground(Desk);

        LVGL::lv_obj_send_event(Desk, Desk_type::Home_event.Into_LVGL_code(), null_mut());
    }
}
