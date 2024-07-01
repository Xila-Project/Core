use cstr_core::CString;
use lvgl::{
    style::{FlexFlow, Style},
    widgets::Label,
    LvError, NativeObject, Obj,
};

pub type Windows_indentifier_type = u16;

struct Window_type<'a> {
    Window: Obj<'a>,
}

impl<'a> Window_type<'a> {
    pub fn New(Parent: &'a mut impl NativeObject, Title_string: &str) -> Result<Self, LvError> {
        let mut Window = Obj::create(Parent)?;

        let mut Header = Obj::create(&mut Window)?;

        let mut Title = Label::create(&mut Header)?;

        Title.set_text(&CString::new(Title_string).unwrap())?;

        let Body = Obj::create(&mut Window)?;

        // TODO : Wait for lv_binding_rust to implement correct drop and lifetime management

        let Window = Self { Window };

        Ok(Window)
    }
}

pub struct Windows_manager_type<'a> {
    Windows: Vec<Window_type<'a>>,
}
