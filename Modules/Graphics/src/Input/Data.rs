use core::mem::transmute;

use crate::{Point_type, LVGL};

use super::{Key_type, State_type};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Input_data_type {
    pub Continue: bool,
    pub Point: Point_type,
    pub State: State_type,
    pub Key: Key_type,
}

impl Default for Input_data_type {
    fn default() -> Self {
        Self {
            Point: Point_type::default(),
            State: State_type::default(),
            Key: Key_type::Character(0),
            Continue: false,
        }
    }
}

impl Input_data_type {
    pub const fn New(Point: Point_type, State: State_type, Key: Key_type, Continue: bool) -> Self {
        Self {
            Point,
            State,
            Key,
            Continue,
        }
    }

    pub const fn Get_continue(&self) -> bool {
        self.Continue
    }

    pub const fn Get_point(&self) -> &Point_type {
        &self.Point
    }

    pub const fn Get_touch(&self) -> &State_type {
        &self.State
    }

    pub const fn Get_key(&self) -> Key_type {
        self.Key
    }

    pub const fn Set_continue(&mut self, Continue: bool) {
        self.Continue = Continue;
    }

    pub fn Set_point(&mut self, Point: Point_type) {
        self.Point = Point;
    }

    pub fn Set_state(&mut self, Touch: State_type) {
        self.State = Touch;
    }

    pub fn Set_key(&mut self, Key: Key_type) {
        self.Key = Key;
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
        Input_device_data.key = Value.Key.into();

        Input_device_data
    }
}
