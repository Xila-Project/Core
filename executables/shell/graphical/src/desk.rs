use core::{ffi::c_void, mem::forget};

use crate::{
    error::{Error, Result},
    icon::create_icon,
    shortcut::{SHORTCUT_PATH, Shortcut},
};

use alloc::{
    collections::btree_map::BTreeMap,
    ffi::CString,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use executable::Standard;
use file_system::{Kind, Mode};
use futures::block_on;
use graphics::{Color, EventKind, Logo, Point, Window, lvgl};
use log::error;
use virtual_file_system::Directory;

pub const WINDOWS_PARENT_CHILD_CHANGED: graphics::EventKind = graphics::EventKind::Custom2;

pub struct Desk {
    window: Window,
    tile_view: *mut lvgl::lv_obj_t,
    drawer_tile: *mut lvgl::lv_obj_t,
    desk_tile: *mut lvgl::lv_obj_t,
    dock: *mut lvgl::lv_obj_t,
    main_button: *mut lvgl::lv_obj_t,
    shortcuts: BTreeMap<*mut lvgl::lv_obj_t, String>,
}

unsafe extern "C" fn event_handler(event: *mut lvgl::lv_event_t) {
    unsafe {
        let raw_code = lvgl::lv_event_get_code(event);
        let code = EventKind::from_lvgl_code(raw_code);

        if code == EventKind::ChildCreated || code == EventKind::ChildDeleted {
            let target = lvgl::lv_event_get_target(event) as *mut lvgl::lv_obj_t;
            let target_parent = lvgl::lv_obj_get_parent(target);

            let current_target = lvgl::lv_event_get_current_target(event) as *mut lvgl::lv_obj_t;

            // If the event is not for the current target, ignore it (not the parent window)
            if target_parent != current_target {
                return;
            }

            let desk = lvgl::lv_event_get_user_data(event) as *mut lvgl::lv_obj_t;

            lvgl::lv_obj_send_event(
                desk,
                WINDOWS_PARENT_CHILD_CHANGED as u32,
                target as *mut c_void,
            );
        }
    }
}

impl Drop for Desk {
    fn drop(&mut self) {
        unsafe {
            let _lock = block_on(graphics::get_instance().lock());

            lvgl::lv_obj_delete(self.dock);
        }
    }
}

impl Desk {
    const DOCK_ICON_SIZE: Point = Point::new(32, 32);
    const DRAWER_ICON_SIZE: Point = Point::new(48, 48);

    pub const HOME_EVENT: EventKind = EventKind::Custom1;

    pub fn get_window_object(&self) -> *mut lvgl::lv_obj_t {
        self.window.get_object()
    }

    pub fn is_hidden(&self) -> bool {
        unsafe { lvgl::lv_obj_has_flag(self.dock, lvgl::lv_obj_flag_t_LV_OBJ_FLAG_HIDDEN) }
    }

    pub async fn new(windows_parent: *mut lvgl::lv_obj_t) -> Result<Self> {
        let graphics = graphics::get_instance();

        // - Lock the graphics
        let _lock = graphics.lock().await; // Lock the graphics

        // - Create a window
        let mut window = graphics.create_window().await?;

        window.set_icon("De", Color::BLACK);

        unsafe {
            lvgl::lv_obj_set_style_pad_all(window.get_object(), 0, lvgl::LV_STATE_DEFAULT);

            lvgl::lv_obj_add_event_cb(
                windows_parent,
                Some(event_handler),
                EventKind::All as u32,
                window.get_object() as *mut core::ffi::c_void,
            );
        }

        // - Create the logo
        unsafe {
            // Create the logo in the background of the window
            let logo = Logo::new(window.get_object(), 4, Color::BLACK)?;
            let logo_inner_object = logo.get_inner_object();
            forget(logo); // Prevent the logo from being dropped

            lvgl::lv_obj_set_align(logo_inner_object, lvgl::lv_align_t_LV_ALIGN_CENTER);
            lvgl::lv_obj_add_flag(
                logo_inner_object,
                lvgl::lv_obj_flag_t_LV_OBJ_FLAG_OVERFLOW_VISIBLE,
            );

            // Set shadow color according to BG color
            for i in 0..4 {
                let part = lvgl::lv_obj_get_child(logo_inner_object, i);

                lvgl::lv_obj_set_style_bg_opa(part, lvgl::LV_OPA_0 as u8, lvgl::LV_STATE_DEFAULT);

                lvgl::lv_obj_set_style_border_width(part, 2, lvgl::LV_STATE_DEFAULT);
            }
        }

        // - Create a tile view
        let tile_view = unsafe {
            let tile_view = lvgl::lv_tileview_create(window.get_object());

            if tile_view.is_null() {
                return Err(Error::FailedToCreateObject);
            }

            lvgl::lv_obj_set_style_bg_opa(tile_view, lvgl::LV_OPA_0 as u8, lvgl::LV_STATE_DEFAULT);
            lvgl::lv_obj_set_scrollbar_mode(
                tile_view,
                lvgl::lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_OFF,
            );

            tile_view
        };

        // - Create the desk tile
        let desk_tile = unsafe {
            let desk = lvgl::lv_tileview_add_tile(tile_view, 0, 0, lvgl::lv_dir_t_LV_DIR_BOTTOM);

            if desk.is_null() {
                return Err(Error::FailedToCreateObject);
            }

            lvgl::lv_obj_set_style_pad_all(desk, 20, lvgl::LV_STATE_DEFAULT);

            desk
        };

        // - Create the drawer tile
        let drawer_tile = unsafe {
            let drawer = lvgl::lv_tileview_add_tile(tile_view, 0, 1, lvgl::lv_dir_t_LV_DIR_TOP);

            if drawer.is_null() {
                return Err(Error::FailedToCreateObject);
            }

            lvgl::lv_obj_set_style_pad_top(drawer, 40, lvgl::LV_STATE_DEFAULT);
            lvgl::lv_obj_set_style_pad_bottom(drawer, 40, lvgl::LV_STATE_DEFAULT);
            lvgl::lv_obj_set_style_pad_left(drawer, 40, lvgl::LV_STATE_DEFAULT);
            lvgl::lv_obj_set_flex_flow(drawer, lvgl::lv_flex_flow_t_LV_FLEX_FLOW_ROW_WRAP);

            drawer
        };

        // - Create a dock
        let dock = unsafe {
            let dock = lvgl::lv_obj_create(desk_tile);

            if dock.is_null() {
                return Err(Error::FailedToCreateObject);
            }

            lvgl::lv_obj_set_style_bg_color(dock, Color::BLACK.into(), lvgl::LV_STATE_DEFAULT);

            lvgl::lv_obj_set_align(dock, lvgl::lv_align_t_LV_ALIGN_BOTTOM_MID);
            lvgl::lv_obj_set_size(dock, lvgl::LV_SIZE_CONTENT, lvgl::LV_SIZE_CONTENT);
            lvgl::lv_obj_set_style_border_width(dock, 0, lvgl::LV_STATE_DEFAULT);
            lvgl::lv_obj_set_flex_flow(dock, lvgl::lv_flex_flow_t_LV_FLEX_FLOW_ROW);
            lvgl::lv_obj_set_style_bg_opa(dock, lvgl::LV_OPA_50 as u8, lvgl::LV_STATE_DEFAULT);

            lvgl::lv_obj_set_style_pad_all(dock, 12, lvgl::LV_STATE_DEFAULT);

            dock
        };

        // - Create the main button
        let main_button = unsafe {
            let logo = Logo::new(dock, 1, Color::WHITE)?;
            let inner_object = logo.get_inner_object();
            forget(logo); // Prevent the logo from being dropped
            inner_object
        };

        let shortcuts = BTreeMap::new();

        let desk: Desk = Self {
            window,
            tile_view,
            desk_tile,
            drawer_tile,
            dock,
            main_button,
            shortcuts,
        };

        Ok(desk)
    }

    unsafe fn create_drawer_shortcut(
        &mut self,
        entry_name: &str,
        name: &str,
        icon_color: Color,
        icon_string: &str,
        drawer: *mut lvgl::lv_obj_t,
    ) -> Result<()> {
        let icon = unsafe {
            let container = lvgl::lv_obj_create(drawer);

            lvgl::lv_obj_set_size(container, 12 * 8, 11 * 8);
            lvgl::lv_obj_set_style_bg_opa(container, lvgl::LV_OPA_0 as u8, lvgl::LV_STATE_DEFAULT);
            lvgl::lv_obj_set_style_border_width(container, 0, lvgl::LV_STATE_DEFAULT);
            lvgl::lv_obj_set_flex_flow(container, lvgl::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN);
            lvgl::lv_obj_set_style_pad_all(container, 0, lvgl::LV_STATE_DEFAULT);
            lvgl::lv_obj_set_flex_align(
                container,
                lvgl::lv_flex_align_t_LV_FLEX_ALIGN_SPACE_EVENLY,
                lvgl::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                lvgl::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
            );

            let icon = create_icon(container, icon_color, icon_string, Self::DRAWER_ICON_SIZE)?;

            let label = lvgl::lv_label_create(container);

            let name = CString::new(name).map_err(Error::NullCharacterInString)?;

            lvgl::lv_label_set_text(label, name.as_ptr());

            icon
        };

        self.shortcuts.insert(icon, entry_name.to_string());

        Ok(())
    }

    async unsafe fn create_drawer_interface(&mut self, drawer: *mut lvgl::lv_obj_t) -> Result<()> {
        unsafe {
            let task = task::get_instance().get_current_task_identifier().await;

            let virtual_file_system = virtual_file_system::get_instance();

            let _ = virtual_file_system
                .create_directory(&SHORTCUT_PATH, task)
                .await;

            let mut buffer: Vec<u8> = vec![];

            let shortcuts_directory = Directory::open(virtual_file_system, SHORTCUT_PATH)
                .await
                .map_err(Error::FailedToReadShortcutDirectory)?;

            for shortcut_entry in shortcuts_directory {
                if shortcut_entry.get_type() != Kind::File {
                    continue;
                }

                if !shortcut_entry.get_name().ends_with(".json") {
                    continue;
                }

                match Shortcut::read(shortcut_entry.get_name(), &mut buffer).await {
                    Ok(shortcut) => {
                        self.create_drawer_shortcut(
                            shortcut_entry.get_name(),
                            shortcut.get_name(),
                            shortcut.get_icon_color(),
                            shortcut.get_icon_string(),
                            drawer,
                        )?;
                    }
                    Err(e) => {
                        error!(
                            "Failed to read shortcut {}: {e:?}",
                            shortcut_entry.get_name()
                        );
                        continue;
                    }
                }
            }

            Ok(())
        }
    }

    async fn execute_shortcut(&self, shortcut_name: &str) -> Result<()> {
        let task = task::get_instance().get_current_task_identifier().await;

        let mut buffer = vec![];

        let shortcut = Shortcut::read(shortcut_name, &mut buffer).await?;

        let standard_in = virtual_file_system::get_instance()
            .open(&"/devices/null", Mode::READ_ONLY.into(), task)
            .await
            .map_err(Error::FailedToOpenStandardFile)?;

        let standard_out = virtual_file_system::get_instance()
            .open(&"/devices/null", Mode::WRITE_ONLY.into(), task)
            .await
            .map_err(Error::FailedToOpenStandardFile)?;

        let standard_err = virtual_file_system::get_instance()
            .open(&"/devices/null", Mode::WRITE_ONLY.into(), task)
            .await
            .map_err(Error::FailedToOpenStandardFile)?;

        executable::execute(
            shortcut.get_command(),
            shortcut.get_arguments().to_string(),
            Standard::new(
                standard_in,
                standard_out,
                standard_err,
                task,
                virtual_file_system::get_instance(),
            ),
        )
        .await
        .map_err(Error::FailedToExecuteShortcut)?;

        Ok(())
    }

    // This function is intentionally private and is only used within this module.
    async fn refresh_dock(&self) -> Result<()> {
        let dock_child_count = unsafe { lvgl::lv_obj_get_child_count(self.dock) };

        let graphics_manager = graphics::get_instance();

        let window_count = graphics_manager.get_window_count().await?;

        // Remove the icons of windows that do not exist anymore
        for i in 0..dock_child_count {
            let icon = unsafe { lvgl::lv_obj_get_child(self.dock, i as i32) };

            if icon == self.main_button {
                continue;
            }

            let dock_window_identifier = unsafe { lvgl::lv_obj_get_user_data(icon) as usize };

            let mut found = Option::None;

            for j in 1..window_count {
                if let Ok(window_identifier) = graphics_manager.get_window_identifier(j).await
                    && window_identifier == dock_window_identifier
                {
                    found = Some(window_identifier);
                    break;
                }
            }

            if found.is_none() {
                unsafe {
                    lvgl::lv_obj_delete(icon);
                }
            }
        }

        // Add the new icons
        for i in 0..window_count {
            let window_identifier =
                if let Ok(window_identifier) = graphics_manager.get_window_identifier(i).await {
                    window_identifier
                } else {
                    continue;
                };

            // Check if the window is not desk
            if window_identifier == self.window.get_identifier() {
                continue;
            }

            // Find the index of the window in the dock
            let found = (1..dock_child_count).find(|&dock_idx| {
                let dock_window_identifier = unsafe {
                    let icon = lvgl::lv_obj_get_child(self.dock, dock_idx as i32);

                    if icon.is_null() {
                        return false;
                    }

                    lvgl::lv_obj_get_user_data(icon) as usize
                };

                dock_window_identifier == window_identifier
            });

            // If the window is not in the dock, add it
            if found.is_none() {
                // Fetch the window identifier once and reuse it
                let window_identifier = graphics_manager.get_window_identifier(i).await?;
                let (icon_string, icon_color) = graphics_manager.get_window_icon(i).await?;

                unsafe {
                    let icon =
                        create_icon(self.dock, icon_color, &icon_string, Self::DOCK_ICON_SIZE)?;

                    lvgl::lv_obj_set_user_data(icon, window_identifier as *mut c_void);
                }
            }
        }

        Ok(())
    }

    pub async fn event_handler(&mut self) {
        let _lock = graphics::get_instance().lock().await;
        while let Some(event) = self.window.pop_event() {
            match event.get_code() {
                Self::HOME_EVENT => unsafe {
                    lvgl::lv_tileview_set_tile_by_index(self.tile_view, 0, 0, true);
                },
                EventKind::ValueChanged => {
                    if event.get_target() == self.tile_view {
                        unsafe {
                            if lvgl::lv_tileview_get_tile_active(self.tile_view) == self.desk_tile {
                                lvgl::lv_obj_clean(self.drawer_tile);
                            } else if lvgl::lv_obj_get_child_count(self.drawer_tile) == 0 {
                                let _ = self.create_drawer_interface(self.drawer_tile).await;
                            }
                        }
                    }
                }
                EventKind::Clicked => {
                    // If the target is a shortcut, execute the shortcut
                    if let Some(shortcut_name) = self.shortcuts.get(&event.get_target()) {
                        if let Err(error) = self.execute_shortcut(shortcut_name).await {
                            error!("Failed to execute shortcut {shortcut_name}: {error:?}");
                        }
                    }
                    // If the target is a dock icon, move the window to the foreground
                    else if unsafe { lvgl::lv_obj_get_parent(event.get_target()) == self.dock } {
                        // Ignore the main button
                        if event.get_target() == self.main_button {
                            continue;
                        }

                        let window_identifier =
                            unsafe { lvgl::lv_obj_get_user_data(event.get_target()) as usize };

                        graphics::get_instance()
                            .maximize_window(window_identifier)
                            .await
                            .unwrap();
                    }
                }
                EventKind::Pressed => {
                    if event.get_target() == self.main_button
                        || unsafe {
                            lvgl::lv_obj_get_parent(event.get_target()) == self.main_button
                        }
                    {
                        unsafe {
                            lvgl::lv_obj_add_state(self.main_button, lvgl::LV_STATE_PRESSED as u16);
                            for i in 0..4 {
                                let part = lvgl::lv_obj_get_child(self.main_button, i);

                                lvgl::lv_obj_add_state(part, lvgl::LV_STATE_PRESSED as u16);
                            }
                        }
                    }
                }
                EventKind::Released => {
                    if event.get_target() == self.main_button
                        || unsafe {
                            lvgl::lv_obj_get_parent(event.get_target()) == self.main_button
                        }
                    {
                        const STATE: u16 = lvgl::LV_STATE_PRESSED as u16;

                        unsafe {
                            lvgl::lv_obj_add_state(self.main_button, STATE);
                            for i in 0..4 {
                                let part = lvgl::lv_obj_get_child(self.main_button, i);

                                lvgl::lv_obj_remove_state(part, STATE);
                            }
                        }

                        unsafe {
                            lvgl::lv_tileview_set_tile_by_index(self.tile_view, 0, 1, true);
                        }
                    }
                }
                WINDOWS_PARENT_CHILD_CHANGED => {
                    // Ignore consecutive windows parent child changed events
                    if let Some(peeked_event) = self.window.peek_event()
                        && peeked_event.get_code() == WINDOWS_PARENT_CHILD_CHANGED
                    {
                        continue;
                    }

                    self.refresh_dock().await.unwrap();
                }
                _ => {}
            }
        }
    }
}
