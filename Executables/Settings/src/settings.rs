use alloc::string::String;
use core::time::Duration;

use file_system::Kind;
use graphics::{
    lvgl,
    palette::{self, Hue},
    EventKind, Window,
};

use crate::error::Result;
use crate::tabs::{GeneralTabType, PasswordTabType, TabType};

pub struct SettingsType {
    window: Window,
    running: bool,
    tabs: [TabType; 2],
}

#[derive(Clone)]
pub struct FileItemType {
    pub name: String,
    pub r#type: Kind,
    pub size: u64,
}

impl SettingsType {
    pub async fn new() -> Result<Self> {
        let _lock = graphics::get_instance().lock().await;

        let mut window = graphics::get_instance().create_window().await?;

        window.set_icon("Se", palette::get(Hue::Grey, palette::Tone::MAIN));

        // Create tabview
        let tabview = unsafe {
            let tabview = lvgl::lv_tabview_create(window.get_object());

            if tabview.is_null() {
                return Err(crate::error::Error::FailedToCreateUiElement);
            }
            tabview
        };

        // Create tabs
        let mut tabs = [
            TabType::GeneralTab(GeneralTabType::new()),
            TabType::PasswordTab(PasswordTabType::new()),
        ];

        tabs.iter_mut().for_each(|tab| {
            tab.create_ui(tabview).expect("Failed to create tab UI");
        });

        let manager = Self {
            window,
            running: true,
            tabs,
        };

        Ok(manager)
    }

    pub async fn run(&mut self) {
        while self.running {
            let event = match self.window.pop_event() {
                Some(event) => event,
                None => {
                    task::Manager::sleep(Duration::from_millis(50)).await;
                    continue;
                }
            };

            if event.get_code() == EventKind::Delete
                && event.get_target() == self.window.get_object()
            {
                self.running = false;
            } else {
                // Let each tab handle the event
                for tab in &mut self.tabs {
                    if tab.handle_event(&event).await {
                        break; // Event was handled, no need to check other tabs
                    }
                }
            }
        }
    }
}
