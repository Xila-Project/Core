use alloc::string::String;
use core::time::Duration;

use File_system::Type_type;
use Graphics::{
    Event_code_type,
    Palette::{self, Hue_type},
    Window_type, LVGL,
};

use crate::Error::Result_type;
use crate::Tabs::{General_tab_type, Password_tab_type, Tab_type};

pub struct Settings_type {
    window: Window_type,
    running: bool,
    tabs: [Tab_type; 2],
}

#[derive(Clone)]
pub struct File_item_type {
    pub name: String,
    pub Type: Type_type,
    pub size: u64,
}

impl Settings_type {
    pub async fn new() -> Result_type<Self> {
        let _lock = Graphics::Get_instance().Lock().await;

        let mut Window = Graphics::Get_instance().Create_window().await?;

        Window.Set_icon("Se", Palette::Get(Hue_type::Grey, Palette::Tone_type::MAIN));

        // Create tabview
        let Tabview = unsafe {
            let tabview = LVGL::lv_tabview_create(Window.Get_object());

            if tabview.is_null() {
                return Err(crate::Error::Error_type::Failed_to_create_UI_element);
            }
            tabview
        };

        // Create tabs
        let mut Tabs = [
            Tab_type::General_tab(General_tab_type::new()),
            Tab_type::Password_tab(Password_tab_type::new()),
        ];

        Tabs.iter_mut().for_each(|Tab| {
            Tab.create_ui(Tabview).expect("Failed to create tab UI");
        });

        let Manager = Self {
            window: Window,
            running: true,
            tabs: Tabs,
        };

        Ok(Manager)
    }

    pub async fn Run(&mut self) {
        while self.running {
            let event = match self.window.Pop_event() {
                Some(event) => event,
                None => {
                    Task::Manager_type::Sleep(Duration::from_millis(50)).await;
                    continue;
                }
            };

            if event.Get_code() == Event_code_type::Delete
                && event.Get_target() == self.window.Get_object()
            {
                self.running = false;
            } else {
                // Let each tab handle the event
                for Tab in &mut self.tabs {
                    if Tab.Handle_event(&event).await {
                        break; // Event was handled, no need to check other tabs
                    }
                }
            }
        }
    }
}
