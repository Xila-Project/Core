use core::ffi::c_void;

use crate::{
    Error::{Error_type, Result_type},
    Icon::Create_icon,
    Shortcut::{Shortcut_type, SHORTCUT_PATH},
};

use alloc::{
    collections::btree_map::BTreeMap,
    ffi::CString,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use Executable::Standard_type;
use File_system::{Mode_type, Type_type};
use Futures::block_on;
use Graphics::{Color_type, Event_code_type, Point_type, Window_type, LVGL};
use Log::Error;
use Virtual_file_system::Directory_type;

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

unsafe extern "C" fn Event_handler(Event: *mut LVGL::lv_event_t) {
    let code = Event_code_type::From_LVGL_code(LVGL::lv_event_get_code(Event));

    if code == Event_code_type::Child_created || code == Event_code_type::Child_deleted {
        let target = LVGL::lv_event_get_target(Event) as *mut LVGL::lv_obj_t;
        let target_parent = LVGL::lv_obj_get_parent(target);

        let Current_target = LVGL::lv_event_get_current_target(Event) as *mut LVGL::lv_obj_t;

        // If the event is not for the current target, ignore it (not the parent window)
        if target_parent != Current_target {
            return;
        }

        let Desk = LVGL::lv_event_get_user_data(Event) as *mut LVGL::lv_obj_t;

        LVGL::lv_obj_send_event(
            Desk,
            WINDOWS_PARENT_CHILD_CHANGED as u32,
            target as *mut c_void,
        );
    }
}

impl Drop for Desk_type {
    fn drop(&mut self) {
        unsafe {
            let _lock = block_on(Graphics::Get_instance().Lock());

            LVGL::lv_obj_delete(self.dock);
        }
    }
}

impl Desk_type {
    const DOCK_ICON_SIZE: Point_type = Point_type::new(32, 32);
    const DRAWER_ICON_SIZE: Point_type = Point_type::new(48, 48);

    pub const HOME_EVENT: Event_code_type = Event_code_type::Custom_1;

    pub fn Get_window_object(&self) -> *mut LVGL::lv_obj_t {
        self.window.Get_object()
    }

    pub fn Is_hidden(&self) -> bool {
        unsafe { LVGL::lv_obj_has_flag(self.dock, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_HIDDEN) }
    }

    pub async fn New(Windows_parent: *mut LVGL::lv_obj_t) -> Result_type<Self> {
        let graphics = Graphics::Get_instance();

        // - Lock the graphics
        let _Lock = graphics.Lock().await; // Lock the graphics

        // - Create a window
        let mut Window = graphics.Create_window().await?;

        Window.Set_icon("De", Color_type::BLACK);

        unsafe {
            LVGL::lv_obj_set_style_pad_all(Window.Get_object(), 0, LVGL::LV_STATE_DEFAULT);

            LVGL::lv_obj_add_event_cb(
                Windows_parent,
                Some(Event_handler),
                Event_code_type::All as u32,
                Window.Get_object() as *mut core::ffi::c_void,
            );
        }

        // - Create the logo
        unsafe {
            // Create the logo in the background of the window
            let Logo = Create_logo(Window.Get_object(), 4, Color_type::BLACK)?;

            LVGL::lv_obj_set_align(Logo, LVGL::lv_align_t_LV_ALIGN_CENTER);
            LVGL::lv_obj_add_flag(Logo, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_OVERFLOW_VISIBLE);

            // Set shadow color according to BG color
            for i in 0..4 {
                let part = LVGL::lv_obj_get_child(Logo, i);

                LVGL::lv_obj_set_style_bg_opa(part, LVGL::LV_OPA_0 as u8, LVGL::LV_STATE_DEFAULT);

                LVGL::lv_obj_set_style_border_width(part, 2, LVGL::LV_STATE_DEFAULT);
            }
        }

        // - Create a tile view
        let Tile_view = unsafe {
            let tile_view = LVGL::lv_tileview_create(Window.Get_object());

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
        let Desk_tile = unsafe {
            let desk = LVGL::lv_tileview_add_tile(Tile_view, 0, 0, LVGL::lv_dir_t_LV_DIR_BOTTOM);

            if desk.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_style_pad_all(desk, 20, LVGL::LV_STATE_DEFAULT);

            desk
        };

        // - Create the drawer tile
        let Drawer_tile = unsafe {
            let drawer = LVGL::lv_tileview_add_tile(Tile_view, 0, 1, LVGL::lv_dir_t_LV_DIR_TOP);

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
        let Dock = unsafe {
            let dock = LVGL::lv_obj_create(Desk_tile);

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
        let Main_button = unsafe { Create_logo(Dock, 1, Color_type::WHITE)? };

        let Shortcuts = BTreeMap::new();

        let Desk = Self {
            window: Window,
            tile_view: Tile_view,
            desk_tile: Desk_tile,
            drawer_tile: Drawer_tile,
            dock: Dock,
            main_button: Main_button,
            shortcuts: Shortcuts,
        };

        Ok(Desk)
    }

    unsafe fn Create_drawer_shortcut(
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

            let Icon = Create_icon(container, icon_color, icon_string, Self::DRAWER_ICON_SIZE)?;

            let Label = LVGL::lv_label_create(container);

            let Name = CString::new(name).map_err(Error_type::Null_character_in_string)?;

            LVGL::lv_label_set_text(Label, Name.as_ptr());

            Icon
        };

        self.shortcuts.insert(icon, entry_name.to_string());

        Ok(())
    }

    async unsafe fn Create_drawer_interface(
        &mut self,
        drawer: *mut LVGL::lv_obj_t,
    ) -> Result_type<()> {
        let task = Task::Get_instance().Get_current_task_identifier().await;

        let Virtual_file_system = Virtual_file_system::Get_instance();

        let _ = Virtual_file_system
            .Create_directory(&SHORTCUT_PATH, task)
            .await;

        let mut Buffer: Vec<u8> = vec![];

        let Shortcuts_directory = Directory_type::Open(Virtual_file_system, SHORTCUT_PATH)
            .await
            .map_err(Error_type::Failed_to_read_shortcut_directory)?;

        for Shortcut_entry in Shortcuts_directory {
            if Shortcut_entry.Get_type() != Type_type::File {
                continue;
            }

            if !Shortcut_entry.Get_name().ends_with(".json") {
                continue;
            }

            match Shortcut_type::Read(Shortcut_entry.Get_name(), &mut Buffer).await {
                Ok(shortcut) => {
                    self.Create_drawer_shortcut(
                        Shortcut_entry.Get_name(),
                        shortcut.Get_name(),
                        shortcut.Get_icon_color(),
                        shortcut.Get_icon_string(),
                        drawer,
                    )?;
                }
                Err(_) => {
                    Error!("Failed to read shortcut {}", Shortcut_entry.Get_name());
                    continue;
                }
            }
        }

        Ok(())
    }

    async fn Execute_shortcut(&self, Shortcut_name: &str) -> Result_type<()> {
        let task = Task::Get_instance().Get_current_task_identifier().await;

        let mut Buffer = vec![];

        let Shortcut = Shortcut_type::Read(Shortcut_name, &mut Buffer).await?;

        let Standard_in = Virtual_file_system::Get_instance()
            .Open(&"/Devices/Null", Mode_type::READ_ONLY.into(), task)
            .await
            .map_err(Error_type::Failed_to_open_standard_file)?;

        let Standard_out = Virtual_file_system::Get_instance()
            .Open(&"/Devices/Null", Mode_type::WRITE_ONLY.into(), task)
            .await
            .map_err(Error_type::Failed_to_open_standard_file)?;

        let Standard_err = Virtual_file_system::Get_instance()
            .Open(&"/Devices/Null", Mode_type::WRITE_ONLY.into(), task)
            .await
            .map_err(Error_type::Failed_to_open_standard_file)?;

        Executable::Execute(
            Shortcut.Get_command(),
            Shortcut.Get_arguments().to_string(),
            Standard_type::New(
                Standard_in,
                Standard_out,
                Standard_err,
                task,
                Virtual_file_system::Get_instance(),
            ),
        )
        .await
        .map_err(Error_type::Failed_to_execute_shortcut)?;

        Ok(())
    }

    // This function is intentionally private and is only used within this module.
    async fn Refresh_dock(&self) -> Result_type<()> {
        let dock_child_count = unsafe { LVGL::lv_obj_get_child_count(self.dock) };

        let Graphics_manager = Graphics::Get_instance();

        let Window_count = Graphics_manager.Get_window_count().await?;

        // Remove the icons of windows that do not exist anymore
        for i in 0..dock_child_count {
            let icon = unsafe { LVGL::lv_obj_get_child(self.dock, i as i32) };

            if icon == self.main_button {
                continue;
            }

            let Dock_window_identifier = unsafe { LVGL::lv_obj_get_user_data(icon) as usize };

            let mut Found = Option::None;

            for j in 1..Window_count {
                if let Ok(window_identifier) = Graphics_manager.Get_window_identifier(j).await {
                    if window_identifier == Dock_window_identifier {
                        Found = Some(window_identifier);
                        break;
                    }
                }
            }

            if Found.is_none() {
                unsafe {
                    LVGL::lv_obj_delete(icon);
                }
            }
        }

        // Add the new icons
        for i in 0..Window_count {
            let window_identifier =
                if let Ok(window_identifier) = Graphics_manager.Get_window_identifier(i).await {
                    window_identifier
                } else {
                    continue;
                };

            // Check if the window is not desk
            if window_identifier == self.window.Get_identifier() {
                continue;
            }

            // Find the index of the window in the dock
            let Found = (1..dock_child_count).find(|&dock_idx| {
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
            if Found.is_none() {
                // Fetch the window identifier once and reuse it
                let Window_identifier = Graphics_manager.Get_window_identifier(i).await?;
                let (icon_string, icon_color) = Graphics_manager.Get_window_icon(i).await?;

                unsafe {
                    let icon =
                        Create_icon(self.dock, icon_color, &icon_string, Self::DOCK_ICON_SIZE)?;

                    LVGL::lv_obj_set_user_data(icon, Window_identifier as *mut c_void);
                }
            }
        }

        Ok(())
    }

    pub async fn Event_handler(&mut self) {
        let _lock = Graphics::Get_instance().Lock().await;
        while let Some(event) = self.window.Pop_event() {
            match event.Get_code() {
                Self::HOME_EVENT => unsafe {
                    LVGL::lv_tileview_set_tile_by_index(self.tile_view, 0, 0, true);
                },
                Event_code_type::Value_changed => {
                    if event.Get_target() == self.tile_view {
                        unsafe {
                            if LVGL::lv_tileview_get_tile_active(self.tile_view) == self.desk_tile {
                                LVGL::lv_obj_clean(self.drawer_tile);
                            } else if LVGL::lv_obj_get_child_count(self.drawer_tile) == 0 {
                                let _ = self.Create_drawer_interface(self.drawer_tile).await;
                            }
                        }
                    }
                }
                Event_code_type::Clicked => {
                    // If the target is a shortcut, execute the shortcut
                    if let Some(Shortcut_name) = self.shortcuts.get(&event.Get_target()) {
                        if let Err(error) = self.Execute_shortcut(Shortcut_name).await {
                            Error!("Failed to execute shortcut {Shortcut_name}: {error:?}");
                        }
                    }
                    // If the target is a dock icon, move the window to the foreground
                    else if unsafe { LVGL::lv_obj_get_parent(event.Get_target()) == self.dock } {
                        // Ignore the main button
                        if event.Get_target() == self.main_button {
                            continue;
                        }

                        let Window_identifier =
                            unsafe { LVGL::lv_obj_get_user_data(event.Get_target()) as usize };

                        Graphics::Get_instance()
                            .Maximize_window(Window_identifier)
                            .await
                            .unwrap();
                    }
                }
                Event_code_type::Pressed => {
                    if event.Get_target() == self.main_button
                        || unsafe {
                            LVGL::lv_obj_get_parent(event.Get_target()) == self.main_button
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
                    if event.Get_target() == self.main_button
                        || unsafe {
                            LVGL::lv_obj_get_parent(event.Get_target()) == self.main_button
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
                    if let Some(Peeked_event) = self.window.Peek_event() {
                        if Peeked_event.Get_code() == WINDOWS_PARENT_CHILD_CHANGED {
                            continue;
                        }
                    }

                    self.Refresh_dock().await.unwrap();
                }
                _ => {}
            }
        }
    }
}

unsafe fn Create_logo(
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

    New_part(logo, LVGL::lv_align_t_LV_ALIGN_TOP_RIGHT, factor, color)?;
    New_part(logo, LVGL::lv_align_t_LV_ALIGN_BOTTOM_RIGHT, factor, color)?;
    New_part(logo, LVGL::lv_align_t_LV_ALIGN_BOTTOM_LEFT, factor, color)?;
    New_part(logo, LVGL::lv_align_t_LV_ALIGN_TOP_LEFT, factor, color)?;

    Ok(logo)
}

fn New_part(
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
