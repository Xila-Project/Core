use core::mem::transmute;

use crate::{lvgl, Point_type};

use super::{Key_type, State_type};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Input_data_type {
    pub Continue: bool,
    pub point: Point_type,
    pub state: State_type,
    pub key: Key_type,
}

impl Default for Input_data_type {
    fn default() -> Self {
        Self {
            point: Point_type::default(),
            state: State_type::default(),
            key: Key_type::Character(0),
            Continue: false,
        }
    }
}

impl Input_data_type {
    pub const fn new(point: Point_type, State: State_type, Key: Key_type, Continue: bool) -> Self {
        Self {
            point,
            state: State,
            key: Key,
            Continue,
        }
    }

    pub const fn get_continue(&self) -> bool {
        self.Continue
    }

    pub const fn get_point(&self) -> &Point_type {
        &self.point
    }

    pub const fn get_touch(&self) -> &State_type {
        &self.state
    }

    pub const fn get_key(&self) -> Key_type {
        self.key
    }

    pub const fn Set_continue(&mut self, Continue: bool) {
        self.Continue = Continue;
    }

    pub fn Set_point(&mut self, Point: Point_type) {
        self.point = Point;
    }

    pub fn Set_state(&mut self, Touch: State_type) {
        self.state = Touch;
    }

    pub fn Set_key(&mut self, Key: Key_type) {
        self.key = Key;
    }

    pub fn Set(&mut self, Point: Point_type, Touch: State_type) {
        self.point = Point;
        self.state = Touch;
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

impl From<Input_data_type> for lvgl::lv_indev_data_t {
    fn from(value: Input_data_type) -> lvgl::lv_indev_data_t {
        let mut input_device_data = lvgl::lv_indev_data_t::default();

        let State = value.get_touch();

        if *State == State_type::Pressed {
            input_device_data.point = (*value.get_point()).into();
        }

        input_device_data.state = (*State).into();
        input_device_data.key = value.key.into();

        input_device_data
    }
}
