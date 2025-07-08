use alloc::string::String;
use core::time::Duration;

use File_system::Type_type;
use Graphics::{
    Event_code_type,
    Palette::{self, Hue_type},
    Window_type, LVGL,
};

use crate::error::Result_type;
use crate::tabs::{General_tab_type, Password_tab_type, Tab_type};

pub struct Settings_type {
    window: Window_type,
    running: bool,
    tabs: [Tab_type; 2],
}

#[derive(Clone)]
pub struct File_item_type {
    pub name: String,
    pub r#type: Type_type,
    pub size: u64,
}

impl Settings_type {
    pub async fn new() -> Result_type<Self> {
        let _lock = Graphics::get_instance().lock().await;

        let mut window = Graphics::get_instance().create_window().await?;

        window.set_icon("Se", Palette::Get(Hue_type::Grey, Palette::Tone_type::MAIN));

        // Create tabview
        let tabview = unsafe {
            let tabview = LVGL::lv_tabview_create(window.get_object());

            if tabview.is_null() {
                return Err(crate::error::Error_type::Failed_to_create_UI_element);
            }
            tabview
        };

        // Create tabs
        let mut tabs = [
            Tab_type::General_tab(General_tab_type::new()),
            Tab_type::Password_tab(Password_tab_type::new()),
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
                    Task::Manager_type::Sleep(Duration::from_millis(50)).await;
                    continue;
                }
            };

            if event.get_code() == Event_code_type::Delete
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
