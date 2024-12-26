use crate::LVGL;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum State_type {
    #[default]
    Released,
    Pressed,
}

impl From<State_type> for LVGL::lv_indev_state_t {
    fn from(Value: State_type) -> LVGL::lv_indev_state_t {
        match Value {
            State_type::Pressed => LVGL::lv_indev_state_t_LV_INDEV_STATE_PRESSED,
            State_type::Released => LVGL::lv_indev_state_t_LV_INDEV_STATE_RELEASED,
        }
    }
}

impl From<State_type> for u8 {
    fn from(Value: State_type) -> u8 {
        Value as u8
    }
}

impl TryFrom<u8> for State_type {
    type Error = ();

    fn try_from(Value: u8) -> Result<Self, Self::Error> {
        match Value {
            0 => Ok(Self::Released),
            1 => Ok(Self::Pressed),
            _ => Err(()),
        }
    }
}
