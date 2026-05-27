use alloc::string::String;
use xila::file_system::Kind;
use xila::graphics::OwnedWindow;
use xila::graphics::{
    self, EventKind, lvgl,
    palette::{self, Hue},
};

use crate::error::Result;
use crate::tabs::{AboutTab, GeneralTab, NetworkTab, PasswordTab, Tab};

pub struct Settings {
    window: OwnedWindow,
    tabs: [Tab; 4],
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
            let tabview = lvgl::lv_tabview_create(window.as_object_mutable());

            if tabview.is_null() {
                return Err(crate::error::Error::FailedToCreateUiElement);
            }
            tabview
        };

        // Create tabs
        let mut tabs = [
            Tab::General(GeneralTab::new()),
            Tab::Password(PasswordTab::new()),
            Tab::Network(NetworkTab::new()),
            Tab::About(AboutTab::new()),
        ];

        for tab in &mut tabs {
            tab.create_ui(tabview).await?;
        }

        let manager = Self { window, tabs };

        Ok(manager)
    }

    pub async fn handle_events(&mut self) -> bool {
        graphics::lock!({
            while let Some(event) = self.window.pop_event() {
                // Logique de filtrage spécifique à Settings
                if (event.code == EventKind::Delete || event.code == EventKind::CloseRequested)
                    && event.target == self.window.as_object_mutable()
                {
                    return false;
                }

                for tab in &mut self.tabs {
                    if tab.handle_event(&event).await {
                        break;
                    }
                }
            }
        });

        self.window.yield_now().await;

        true
    }
}
