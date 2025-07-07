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
    Window: Window_type,
    Running: bool,
    Tabs: [Tab_type; 2],
}

#[derive(Clone)]
pub struct File_item_type {
    pub Name: String,
    pub Type: Type_type,
    pub Size: u64,
}

impl Settings_type {
    pub async fn New() -> Result_type<Self> {
        let _Lock = Graphics::Get_instance().Lock().await;

        let mut Window = Graphics::Get_instance().Create_window().await?;

        Window.Set_icon("Se", Palette::Get(Hue_type::Grey, Palette::Tone_type::MAIN));

        // Create tabview
        let Tabview = unsafe {
            let Tabview = LVGL::lv_tabview_create(Window.Get_object());

            if Tabview.is_null() {
                return Err(crate::Error::Error_type::Failed_to_create_UI_element);
            }
            Tabview
        };

        // Create tabs
        let mut Tabs = [
            Tab_type::General_tab(General_tab_type::New()),
            Tab_type::Password_tab(Password_tab_type::New()),
        ];

        Tabs.iter_mut().for_each(|Tab| {
            Tab.Create_UI(Tabview).expect("Failed to create tab UI");
        });

        let Manager = Self {
            Window,
            Running: true,
            Tabs,
        };

        Ok(Manager)
    }

    pub async fn Run(&mut self) {
        while self.Running {
            let Event = match self.Window.Pop_event() {
                Some(Event) => Event,
                None => {
                    Task::Manager_type::Sleep(Duration::from_millis(50)).await;
                    continue;
                }
            };

            if Event.Get_code() == Event_code_type::Delete
                && Event.Get_target() == self.Window.Get_object()
            {
                self.Running = false;
            } else {
                // Let each tab handle the event
                for Tab in &mut self.Tabs {
                    if Tab.Handle_event(&Event).await {
                        break; // Event was handled, no need to check other tabs
                    }
                }
            }
        }
    }
}
