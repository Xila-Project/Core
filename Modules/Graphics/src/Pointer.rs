use core::mem::size_of;
use std::mem::transmute;

use super::LVGL;

use crate::Point_type;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(C)]
pub struct Input_data_type {
    pub Point: Point_type,
    pub State: State_type,
    pub Key: u32,
}

impl Input_data_type {
    pub const fn New(Point: Point_type, State: State_type, Key: u32) -> Self {
        Self { Point, State, Key }
    }

    pub const fn Get_point(&self) -> &Point_type {
        &self.Point
    }

    pub const fn Get_touch(&self) -> &State_type {
        &self.State
    }

    pub fn Set_point(&mut self, Point: Point_type) {
        self.Point = Point;
    }

    pub fn Set_touch(&mut self, Touch: State_type) {
        self.State = Touch;
    }

    pub fn Set(&mut self, Point: Point_type, Touch: State_type) {
        self.Point = Point;
        self.State = Touch;
    }
}

impl TryFrom<&mut [u8]> for &mut Input_data_type {
    type Error = ();

    fn try_from(Value: &mut [u8]) -> Result<Self, Self::Error> {
        if Value.len() != size_of::<Input_data_type>() {
            return Err(());
        }
        if Value.as_ptr() as usize % core::mem::align_of::<Input_data_type>() != 0 {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*mut u8, Self>(Value.as_mut_ptr()) })
    }
}

impl AsMut<[u8]> for Input_data_type {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, size_of::<Self>()) }
    }
}

impl From<Input_data_type> for LVGL::lv_indev_data_t {
    fn from(Value: Input_data_type) -> LVGL::lv_indev_data_t {
        let mut Input_device_data = LVGL::lv_indev_data_t::default();

        let State = Value.Get_touch();

        if *State == State_type::Pressed {
            Input_device_data.point = (*Value.Get_point()).into();
        }

        Input_device_data.state = (*State).into();
        Input_device_data.key = Value.Key;

        Input_device_data
    }
}

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
