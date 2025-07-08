use crate::error::Result_type;
use graphics::{Event_type, LVGL};

/// Enum to hold all tab types (avoids dyn compatibility issues with async traits)
pub enum Tab_type {
    General_tab(General_tab_type),
    Password_tab(Password_tab_type),
}

impl Tab_type {
    pub fn create_ui(&mut self, parent: *mut LVGL::lv_obj_t) -> Result_type<*mut LVGL::lv_obj_t> {
        match self {
            Tab_type::General_tab(tab) => tab.create_ui(parent),
            Tab_type::Password_tab(tab) => tab.create_ui(parent),
        }
    }

    pub async fn handle_event(&mut self, event: &Event_type) -> bool {
        match self {
            Tab_type::General_tab(tab) => tab.Handle_event(event).await,
            Tab_type::Password_tab(tab) => tab.handle_event(event).await,
        }
    }
}

// Re-export tab modules
pub mod password;
pub mod general;

pub use general::General_tab_type;
pub use password::Password_tab_type;
