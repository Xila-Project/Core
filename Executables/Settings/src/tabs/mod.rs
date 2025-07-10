use crate::error::Result;
use graphics::{lvgl, Event};

/// Enum to hold all tab types (avoids dyn compatibility issues with async traits)
pub enum TabType {
    GeneralTab(GeneralTabType),
    PasswordTab(PasswordTabType),
}

impl TabType {
    pub fn create_ui(&mut self, parent: *mut lvgl::lv_obj_t) -> Result<*mut lvgl::lv_obj_t> {
        match self {
            TabType::GeneralTab(tab) => tab.create_ui(parent),
            TabType::PasswordTab(tab) => tab.create_ui(parent),
        }
    }

    pub async fn handle_event(&mut self, event: &Event) -> bool {
        match self {
            TabType::GeneralTab(tab) => tab.handle_event(event).await,
            TabType::PasswordTab(tab) => tab.handle_event(event).await,
        }
    }
}

// Re-export tab modules
pub mod general;
pub mod password;

pub use general::GeneralTabType;
pub use password::PasswordTabType;
