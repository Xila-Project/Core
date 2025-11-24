use alloc::string::String;
use xila::file_system::Kind;
use xila::graphics::{
    self, EventKind, Window, lvgl,
    palette::{self, Hue},
};

use crate::error::Result;
use crate::tabs::{AboutTab, GeneralTab, PasswordTab, Tab};

pub struct Settings {
    window: Window,
    tabs: [Tab; 3],
}

#[derive(Clone)]
pub struct FileItem {
    pub name: String,
    pub r#type: Kind,
    pub size: u64,
}

impl Settings {
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
            Tab::GeneralTab(GeneralTab::new()),
            Tab::PasswordTab(PasswordTab::new()),
            Tab::AboutTab(AboutTab::new()),
        ];

        tabs.iter_mut().for_each(|tab| {
            tab.create_ui(tabview).expect("Failed to create tab UI");
        });

        let manager = Self { window, tabs };

        Ok(manager)
    }

    pub async fn handle_events(&mut self) -> bool {
        let mut running = true;

        graphics::lock!({
            while let Some(event) = self.window.pop_event() {
                if event.code == EventKind::Delete && event.target == self.window.get_object() {
                    running = false;
                } else {
                    // Let each tab handle the event
                    for tab in &mut self.tabs {
                        if tab.handle_event(&event).await {
                            break; // Event was handled, no need to check other tabs
                        }
                    }
                }
            }
        });

        running
    }
}
