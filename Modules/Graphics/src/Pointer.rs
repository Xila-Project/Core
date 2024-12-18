use core::mem::size_of;
use std::mem::transmute;

use super::LVGL;

use crate::Point_type;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(C)]
pub struct Pointer_data_type {
    pub Point: Point_type,
    pub Touch: Touch_type,
}

impl Pointer_data_type {
    pub const fn New(Point: Point_type, Touch: Touch_type) -> Self {
        Self { Point, Touch }
    }

    pub const fn Get_point(&self) -> &Point_type {
        &self.Point
    }

    pub const fn Get_touch(&self) -> &Touch_type {
        &self.Touch
    }

    pub fn Set_point(&mut self, Point: Point_type) {
        self.Point = Point;
    }

    pub fn Set_touch(&mut self, Touch: Touch_type) {
        self.Touch = Touch;
    }

    pub fn Set(&mut self, Point: Point_type, Touch: Touch_type) {
        self.Point = Point;
        self.Touch = Touch;
    }
}

impl TryFrom<&mut [u8]> for &mut Pointer_data_type {
    type Error = ();

    fn try_from(Value: &mut [u8]) -> Result<Self, Self::Error> {
        if Value.len() != size_of::<Pointer_data_type>() {
            return Err(());
        }
        if Value.as_ptr() as usize % core::mem::align_of::<Pointer_data_type>() != 0 {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { transmute::<*mut u8, Self>(Value.as_mut_ptr()) })
    }
}

impl AsMut<[u8]> for Pointer_data_type {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, size_of::<Self>()) }
    }
}

impl From<Pointer_data_type> for LVGL::lv_indev_data_t {
    fn from(Value: Pointer_data_type) -> LVGL::lv_indev_data_t {
        let mut Input_device_data = LVGL::lv_indev_data_t::default();

        let State = Value.Get_touch();

        if *State == Touch_type::Pressed {
            Input_device_data.point = (*Value.Get_point()).into();
        }

        Input_device_data.state = (*State).into();

        Input_device_data
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Touch_type {
    #[default]
    Released,
    Pressed,
}

impl From<Touch_type> for LVGL::lv_indev_state_t {
    fn from(Value: Touch_type) -> LVGL::lv_indev_state_t {
        match Value {
            Touch_type::Pressed => LVGL::lv_indev_state_t_LV_INDEV_STATE_PRESSED,
            Touch_type::Released => LVGL::lv_indev_state_t_LV_INDEV_STATE_RELEASED,
        }
    }
}

impl From<Touch_type> for u8 {
    fn from(Value: Touch_type) -> u8 {
        Value as u8
    }
}

impl TryFrom<u8> for Touch_type {
    type Error = ();

    fn try_from(Value: u8) -> Result<Self, Self::Error> {
        match Value {
            0 => Ok(Self::Released),
            1 => Ok(Self::Pressed),
            _ => Err(()),
        }
    }
}
