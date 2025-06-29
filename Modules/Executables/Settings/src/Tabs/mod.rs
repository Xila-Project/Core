use crate::Error::Result_type;
use Graphics::{Event_type, LVGL};

/// Enum to hold all tab types (avoids dyn compatibility issues with async traits)
pub enum Tab_type {
    General_tab(General_tab_type),
    Password_tab(Password_tab_type),
}

impl Tab_type {
    pub fn Create_UI(&mut self, Parent: *mut LVGL::lv_obj_t) -> Result_type<*mut LVGL::lv_obj_t> {
        match self {
            Tab_type::General_tab(Tab) => Tab.Create_UI(Parent),
            Tab_type::Password_tab(Tab) => Tab.Create_UI(Parent),
        }
    }

    pub async fn Handle_event(&mut self, event: &Event_type) -> bool {
        match self {
            Tab_type::General_tab(tab) => tab.Handle_event(event).await,
            Tab_type::Password_tab(tab) => tab.Handle_event(event).await,
        }
    }
}

// Re-export tab modules
pub mod General;
pub mod Password;

pub use General::General_tab_type;
pub use Password::Password_tab_type;
