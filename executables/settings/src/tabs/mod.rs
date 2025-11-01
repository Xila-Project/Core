use crate::error::Result;
use xila::graphics::{Event, lvgl};

/// Enum to hold all tab types (avoids dyn compatibility issues with async traits)
pub enum Tab {
    GeneralTab(GeneralTab),
    PasswordTab(PasswordTab),
    AboutTab(AboutTab),
}

impl Tab {
    pub fn create_ui(&mut self, parent: *mut lvgl::lv_obj_t) -> Result<*mut lvgl::lv_obj_t> {
        match self {
            Tab::GeneralTab(tab) => tab.create_ui(parent),
            Tab::PasswordTab(tab) => tab.create_ui(parent),
            Tab::AboutTab(tab) => tab.create_ui(parent),
        }
    }

    pub async fn handle_event(&mut self, event: &Event) -> bool {
        match self {
            Tab::GeneralTab(tab) => tab.handle_event(event).await,
            Tab::PasswordTab(tab) => tab.handle_event(event).await,
            Tab::AboutTab(tab) => tab.handle_event(event).await,
        }
    }
}

// Re-export tab modules
pub mod about;
pub mod general;
pub mod password;

pub use about::AboutTab;
pub use general::GeneralTab;
pub use password::PasswordTab;
