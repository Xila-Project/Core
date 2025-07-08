use core::ffi::c_void;

use crate::{
    Error::{Error_type, Result_type},
    Icon::create_icon,
    Shortcut::{Shortcut_type, SHORTCUT_PATH},
};

use alloc::{
    collections::btree_map::BTreeMap,
    ffi::CString,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use executable::Standard_type;
use file_system::{Mode_type, Type_type};
use futures::block_on;
use graphics::{Color_type, Event_code_type, Point_type, Window_type, LVGL};
use log::Error;
use virtual_file_system::Directory_type;

pub const WINDOWS_PARENT_CHILD_CHANGED: Graphics::Event_code_type =
    Graphics::Event_code_type::Custom_2;

pub struct Desk_type {
    window: Window_type,
    tile_view: *mut LVGL::lv_obj_t,
    drawer_tile: *mut LVGL::lv_obj_t,
    desk_tile: *mut LVGL::lv_obj_t,
    dock: *mut LVGL::lv_obj_t,
    main_button: *mut LVGL::lv_obj_t,
    shortcuts: BTreeMap<*mut LVGL::lv_obj_t, String>,
}

unsafe extern "C" fn event_handler(event: *mut LVGL::lv_event_t) {
    let code = Event_code_type::From_LVGL_code(LVGL::lv_event_get_code(event));

    if code == Event_code_type::Child_created || code == Event_code_type::Child_deleted {
        let target = LVGL::lv_event_get_target(event) as *mut LVGL::lv_obj_t;
        let target_parent = LVGL::lv_obj_get_parent(target);

        let current_target = LVGL::lv_event_get_current_target(event) as *mut LVGL::lv_obj_t;

        // If the event is not for the current target, ignore it (not the parent window)
        if target_parent != current_target {
            return;
        }

        let desk = LVGL::lv_event_get_user_data(event) as *mut LVGL::lv_obj_t;

        LVGL::lv_obj_send_event(
            desk,
            WINDOWS_PARENT_CHILD_CHANGED as u32,
            target as *mut c_void,
        );
    }
}

impl Drop for Desk_type {
    fn drop(&mut self) {
        unsafe {
            let _lock = block_on(Graphics::get_instance().lock());

            LVGL::lv_obj_delete(self.dock);
        }
    }
}

impl Desk_type {
    const DOCK_ICON_SIZE: Point_type = Point_type::new(32, 32);
    const DRAWER_ICON_SIZE: Point_type = Point_type::new(48, 48);

    pub const HOME_EVENT: Event_code_type = Event_code_type::Custom_1;

    pub fn get_window_object(&self) -> *mut LVGL::lv_obj_t {
        self.window.get_object()
    }

    pub fn is_hidden(&self) -> bool {
        unsafe { LVGL::lv_obj_has_flag(self.dock, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_HIDDEN) }
    }

    pub async fn new(windows_parent: *mut LVGL::lv_obj_t) -> Result_type<Self> {
        let graphics = Graphics::get_instance();

        // - Lock the graphics
        let _lock = graphics.lock().await; // Lock the graphics

        // - Create a window
        let mut window = graphics.create_window().await?;

        window.set_icon("De", Color_type::BLACK);

        unsafe {
            LVGL::lv_obj_set_style_pad_all(window.get_object(), 0, LVGL::LV_STATE_DEFAULT);

            LVGL::lv_obj_add_event_cb(
                windows_parent,
                Some(event_handler),
                Event_code_type::All as u32,
                window.get_object() as *mut core::ffi::c_void,
            );
        }

        // - Create the logo
        unsafe {
            // Create the logo in the background of the window
            let logo = create_logo(window.get_object(), 4, Color_type::BLACK)?;

            LVGL::lv_obj_set_align(logo, LVGL::lv_align_t_LV_ALIGN_CENTER);
            LVGL::lv_obj_add_flag(logo, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_OVERFLOW_VISIBLE);

            // Set shadow color according to BG color
            for i in 0..4 {
                let part = LVGL::lv_obj_get_child(logo, i);

                LVGL::lv_obj_set_style_bg_opa(part, LVGL::LV_OPA_0 as u8, LVGL::LV_STATE_DEFAULT);

                LVGL::lv_obj_set_style_border_width(part, 2, LVGL::LV_STATE_DEFAULT);
            }
        }

        // - Create a tile view
        let tile_view = unsafe {
            let tile_view = LVGL::lv_tileview_create(window.get_object());

            if tile_view.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_style_bg_opa(tile_view, LVGL::LV_OPA_0 as u8, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_scrollbar_mode(
                tile_view,
                LVGL::lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_OFF,
            );

            tile_view
        };

        // - Create the desk tile
        let desk_tile = unsafe {
            let desk = LVGL::lv_tileview_add_tile(tile_view, 0, 0, LVGL::lv_dir_t_LV_DIR_BOTTOM);

            if desk.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_style_pad_all(desk, 20, LVGL::LV_STATE_DEFAULT);

            desk
        };

        // - Create the drawer tile
        let drawer_tile = unsafe {
            let drawer = LVGL::lv_tileview_add_tile(tile_view, 0, 1, LVGL::lv_dir_t_LV_DIR_TOP);

            if drawer.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_style_pad_top(drawer, 40, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_pad_bottom(drawer, 40, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_pad_left(drawer, 40, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_flex_flow(drawer, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_ROW_WRAP);

            drawer
        };

        // - Create a dock
        let dock = unsafe {
            let dock = LVGL::lv_obj_create(desk_tile);

            if dock.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_style_bg_color(dock, Color_type::BLACK.into(), LVGL::LV_STATE_DEFAULT);

            LVGL::lv_obj_set_align(dock, LVGL::lv_align_t_LV_ALIGN_BOTTOM_MID);
            LVGL::lv_obj_set_size(dock, LVGL::LV_SIZE_CONTENT, LVGL::LV_SIZE_CONTENT);
            LVGL::lv_obj_set_style_border_width(dock, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_flex_flow(dock, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_ROW);
            LVGL::lv_obj_set_style_bg_opa(dock, LVGL::LV_OPA_50 as u8, LVGL::LV_STATE_DEFAULT);

            LVGL::lv_obj_set_style_pad_all(dock, 12, LVGL::LV_STATE_DEFAULT);

            dock
        };

        // - Create the main button
        let main_button = unsafe { create_logo(dock, 1, Color_type::WHITE)? };

        let shortcuts = BTreeMap::new();

        let desk: Desk_type = Self {
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
        icon_color: Color_type,
        icon_string: &str,
        drawer: *mut LVGL::lv_obj_t,
    ) -> Result_type<()> {
        let icon = unsafe {
            let container = LVGL::lv_obj_create(drawer);

            LVGL::lv_obj_set_size(container, 12 * 8, 11 * 8);
            LVGL::lv_obj_set_style_bg_opa(container, LVGL::LV_OPA_0 as u8, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_border_width(container, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_flex_flow(container, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN);
            LVGL::lv_obj_set_style_pad_all(container, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_flex_align(
                container,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_SPACE_EVENLY,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
            );

            let icon = create_icon(container, icon_color, icon_string, Self::DRAWER_ICON_SIZE)?;

            let label = LVGL::lv_label_create(container);

            let name = CString::new(name).map_err(Error_type::Null_character_in_string)?;

            LVGL::lv_label_set_text(label, name.as_ptr());

            icon
        };

        self.shortcuts.insert(icon, entry_name.to_string());

        Ok(())
    }

    async unsafe fn create_drawer_interface(
        &mut self,
        drawer: *mut LVGL::lv_obj_t,
    ) -> Result_type<()> {
        let task = Task::get_instance().get_current_task_identifier().await;

        let virtual_file_system = Virtual_file_system::get_instance();

        let _ = virtual_file_system
            .create_directory(&SHORTCUT_PATH, task)
            .await;

        let mut buffer: Vec<u8> = vec![];

        let shortcuts_directory = Directory_type::open(virtual_file_system, SHORTCUT_PATH)
            .await
            .map_err(Error_type::Failed_to_read_shortcut_directory)?;

        for shortcut_entry in shortcuts_directory {
            if shortcut_entry.get_type() != Type_type::File {
                continue;
            }

            if !shortcut_entry.get_name().ends_with(".json") {
                continue;
            }

            match Shortcut_type::read(shortcut_entry.get_name(), &mut buffer).await {
                Ok(shortcut) => {
                    self.create_drawer_shortcut(
                        shortcut_entry.get_name(),
                        shortcut.get_name(),
                        shortcut.get_icon_color(),
                        shortcut.get_icon_string(),
                        drawer,
                    )?;
                }
                Err(_) => {
                    Error!("Failed to read shortcut {}", shortcut_entry.get_name());
                    continue;
                }
            }
        }

        Ok(())
    }

    async fn execute_shortcut(&self, shortcut_name: &str) -> Result_type<()> {
        let task = Task::get_instance().get_current_task_identifier().await;

        let mut buffer = vec![];

        let shortcut = Shortcut_type::read(shortcut_name, &mut buffer).await?;

        let standard_in = Virtual_file_system::get_instance()
            .open(&"/Devices/Null", Mode_type::READ_ONLY.into(), task)
            .await
            .map_err(Error_type::Failed_to_open_standard_file)?;

        let standard_out = Virtual_file_system::get_instance()
            .open(&"/Devices/Null", Mode_type::WRITE_ONLY.into(), task)
            .await
            .map_err(Error_type::Failed_to_open_standard_file)?;

        let standard_err = Virtual_file_system::get_instance()
            .open(&"/Devices/Null", Mode_type::WRITE_ONLY.into(), task)
            .await
            .map_err(Error_type::Failed_to_open_standard_file)?;

        Executable::execute(
            shortcut.get_command(),
            shortcut.get_arguments().to_string(),
            Standard_type::new(
                standard_in,
                standard_out,
                standard_err,
                task,
                Virtual_file_system::get_instance(),
            ),
        )
        .await
        .map_err(Error_type::Failed_to_execute_shortcut)?;

        Ok(())
    }

    // This function is intentionally private and is only used within this module.
    async fn refresh_dock(&self) -> Result_type<()> {
        let dock_child_count = unsafe { LVGL::lv_obj_get_child_count(self.dock) };

        let graphics_manager = Graphics::get_instance();

        let window_count = graphics_manager.get_window_count().await?;

        // Remove the icons of windows that do not exist anymore
        for i in 0..dock_child_count {
            let icon = unsafe { LVGL::lv_obj_get_child(self.dock, i as i32) };

            if icon == self.main_button {
                continue;
            }

            let dock_window_identifier = unsafe { LVGL::lv_obj_get_user_data(icon) as usize };

            let mut found = Option::None;

            for j in 1..window_count {
                if let Ok(window_identifier) = graphics_manager.get_window_identifier(j).await {
                    if window_identifier == dock_window_identifier {
                        found = Some(window_identifier);
                        break;
                    }
                }
            }

            if found.is_none() {
                unsafe {
                    LVGL::lv_obj_delete(icon);
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
                    let icon = LVGL::lv_obj_get_child(self.dock, dock_idx as i32);

                    if icon.is_null() {
                        return false;
                    }

                    LVGL::lv_obj_get_user_data(icon) as usize
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

                    LVGL::lv_obj_set_user_data(icon, window_identifier as *mut c_void);
                }
            }
        }

        Ok(())
    }

    pub async fn event_handler(&mut self) {
        let _lock = Graphics::get_instance().lock().await;
        while let Some(event) = self.window.pop_event() {
            match event.get_code() {
                Self::HOME_EVENT => unsafe {
                    LVGL::lv_tileview_set_tile_by_index(self.tile_view, 0, 0, true);
                },
                Event_code_type::Value_changed => {
                    if event.get_target() == self.tile_view {
                        unsafe {
                            if LVGL::lv_tileview_get_tile_active(self.tile_view) == self.desk_tile {
                                LVGL::lv_obj_clean(self.drawer_tile);
                            } else if LVGL::lv_obj_get_child_count(self.drawer_tile) == 0 {
                                let _ = self.create_drawer_interface(self.drawer_tile).await;
                            }
                        }
                    }
                }
                Event_code_type::Clicked => {
                    // If the target is a shortcut, execute the shortcut
                    if let Some(shortcut_name) = self.shortcuts.get(&event.get_target()) {
                        if let Err(error) = self.execute_shortcut(shortcut_name).await {
                            Error!("Failed to execute shortcut {shortcut_name}: {error:?}");
                        }
                    }
                    // If the target is a dock icon, move the window to the foreground
                    else if unsafe { LVGL::lv_obj_get_parent(event.get_target()) == self.dock } {
                        // Ignore the main button
                        if event.get_target() == self.main_button {
                            continue;
                        }

                        let window_identifier =
                            unsafe { LVGL::lv_obj_get_user_data(event.get_target()) as usize };

                        Graphics::get_instance()
                            .maximize_window(window_identifier)
                            .await
                            .unwrap();
                    }
                }
                Event_code_type::Pressed => {
                    if event.get_target() == self.main_button
                        || unsafe {
                            LVGL::lv_obj_get_parent(event.get_target()) == self.main_button
                        }
                    {
                        unsafe {
                            LVGL::lv_obj_add_state(self.main_button, LVGL::LV_STATE_PRESSED as u16);
                            for i in 0..4 {
                                let part = LVGL::lv_obj_get_child(self.main_button, i);

                                LVGL::lv_obj_add_state(part, LVGL::LV_STATE_PRESSED as u16);
                            }
                        }
                    }
                }
                Event_code_type::Released => {
                    if event.get_target() == self.main_button
                        || unsafe {
                            LVGL::lv_obj_get_parent(event.get_target()) == self.main_button
                        }
                    {
                        const STATE: u16 = LVGL::LV_STATE_PRESSED as u16;

                        unsafe {
                            LVGL::lv_obj_add_state(self.main_button, STATE);
                            for i in 0..4 {
                                let part = LVGL::lv_obj_get_child(self.main_button, i);

                                LVGL::lv_obj_remove_state(part, STATE);
                            }
                        }

                        unsafe {
                            LVGL::lv_tileview_set_tile_by_index(self.tile_view, 0, 1, true);
                        }
                    }
                }
                WINDOWS_PARENT_CHILD_CHANGED => {
                    // Ignore consecutive windows parent child changed events
                    if let Some(peeked_event) = self.window.peek_event() {
                        if peeked_event.get_code() == WINDOWS_PARENT_CHILD_CHANGED {
                            continue;
                        }
                    }

                    self.refresh_dock().await.unwrap();
                }
                _ => {}
            }
        }
    }
}

unsafe fn create_logo(
    parent: *mut LVGL::lv_obj_t,
    factor: u8,
    color: Color_type,
) -> Result_type<*mut LVGL::lv_obj_t> {
    let logo = LVGL::lv_button_create(parent);

    if logo.is_null() {
        return Err(Error_type::Failed_to_create_object);
    }

    LVGL::lv_obj_set_size(logo, 32 * factor as i32, 32 * factor as i32);
    LVGL::lv_obj_set_style_bg_opa(logo, LVGL::LV_OPA_0 as u8, LVGL::LV_STATE_DEFAULT);
    LVGL::lv_obj_set_style_pad_all(logo, 0, LVGL::LV_STATE_DEFAULT);
    LVGL::lv_obj_set_style_radius(logo, 0, LVGL::LV_STATE_DEFAULT);
    LVGL::lv_obj_set_style_border_width(logo, 0, LVGL::LV_STATE_DEFAULT);

    new_part(logo, LVGL::lv_align_t_LV_ALIGN_TOP_RIGHT, factor, color)?;
    new_part(logo, LVGL::lv_align_t_LV_ALIGN_BOTTOM_RIGHT, factor, color)?;
    new_part(logo, LVGL::lv_align_t_LV_ALIGN_BOTTOM_LEFT, factor, color)?;
    new_part(logo, LVGL::lv_align_t_LV_ALIGN_TOP_LEFT, factor, color)?;

    Ok(logo)
}

fn new_part(
    parent: *mut LVGL::lv_obj_t,
    alignment: LVGL::lv_align_t,
    factor: u8,
    color: Color_type,
) -> Result_type<*mut LVGL::lv_obj_t> {
    let size = (10_i32 * factor as i32, 21_i32 * factor as i32);

    unsafe {
        let part = LVGL::lv_button_create(parent);

        if part.is_null() {
            return Err(Error_type::Failed_to_create_object);
        }

        LVGL::lv_obj_set_style_bg_color(part, color.Into_LVGL_color(), LVGL::LV_STATE_DEFAULT);
        LVGL::lv_obj_set_style_bg_color(part, LVGL::lv_color_white(), LVGL::LV_STATE_PRESSED);

        LVGL::lv_obj_set_align(part, alignment);

        match alignment {
            LVGL::lv_align_t_LV_ALIGN_TOP_LEFT | LVGL::lv_align_t_LV_ALIGN_BOTTOM_RIGHT => {
                LVGL::lv_obj_set_size(part, size.0, size.1);
            }
            LVGL::lv_align_t_LV_ALIGN_BOTTOM_LEFT | LVGL::lv_align_t_LV_ALIGN_TOP_RIGHT => {
                LVGL::lv_obj_set_size(part, size.1, size.0);
            }
            _ => {}
        }

        LVGL::lv_obj_set_style_pad_all(part, 0, LVGL::LV_STATE_DEFAULT);
        LVGL::lv_obj_set_style_radius(part, 0, LVGL::LV_STATE_DEFAULT);
        LVGL::lv_obj_set_style_border_width(part, 0, LVGL::LV_STATE_DEFAULT);
        LVGL::lv_obj_add_flag(part, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_EVENT_BUBBLE);

        Ok(part)
    }
}
