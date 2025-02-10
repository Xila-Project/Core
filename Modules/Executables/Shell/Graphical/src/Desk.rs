use std::{collections::BTreeMap, ffi::CString, os::raw::c_void};

use crate::{
    Error::{Error_type, Result_type},
    Icon::Create_icon,
    Shortcut::{Shortcut_path, Shortcut_type},
};

use Executable::Standard_type;
use File_system::{Mode_type, Type_type};
use Graphics::{Color_type, Event_code_type, Point_type, Window_type, LVGL};
use Virtual_file_system::Directory_type;

pub const Windows_parent_child_changed: Graphics::Event_code_type =
    Graphics::Event_code_type::Custom_2;

pub struct Desk_type {
    Window: Window_type,
    Tile_view: *mut LVGL::lv_obj_t,
    Drawer_tile: *mut LVGL::lv_obj_t,
    Desk_tile: *mut LVGL::lv_obj_t,
    Dock: *mut LVGL::lv_obj_t,
    Main_button: *mut LVGL::lv_obj_t,
    Shortcuts: BTreeMap<*mut LVGL::lv_obj_t, String>,
}

unsafe extern "C" fn Event_handler(Event: *mut LVGL::lv_event_t) {
    let Code = Event_code_type::From_LVGL_code(LVGL::lv_event_get_code(Event));

    if Code == Event_code_type::Child_created || Code == Event_code_type::Child_deleted {
        let Target = LVGL::lv_event_get_target(Event) as *mut LVGL::lv_obj_t;
        let Target_parent = LVGL::lv_obj_get_parent(Target);

        let Current_target = LVGL::lv_event_get_current_target(Event) as *mut LVGL::lv_obj_t;

        // If the event is not for the current target, ignore it (not the parent window)
        if Target_parent != Current_target {
            return;
        }

        let Desk = LVGL::lv_event_get_user_data(Event) as *mut LVGL::lv_obj_t;

        LVGL::lv_obj_send_event(
            Desk,
            Windows_parent_child_changed as u32,
            Target as *mut c_void,
        );
    }
}

impl Drop for Desk_type {
    fn drop(&mut self) {
        unsafe {
            if let Ok(_Lock) = Graphics::Get_instance().Lock() {
                LVGL::lv_obj_delete(self.Dock);
            }
        }
    }
}

impl Desk_type {
    const Dock_icon_size: Point_type = Point_type::New(32, 32);
    const Drawer_icon_size: Point_type = Point_type::New(48, 48);

    pub const Home_event: Event_code_type = Event_code_type::Custom_1;

    pub fn Get_window_object(&self) -> *mut LVGL::lv_obj_t {
        self.Window.Get_object()
    }

    pub fn Is_hidden(&self) -> bool {
        unsafe { LVGL::lv_obj_has_flag(self.Dock, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_HIDDEN) }
    }

    pub fn New(Windows_parent: *mut LVGL::lv_obj_t) -> Result_type<Self> {
        // - Lock the graphics
        let _Lock = Graphics::Get_instance().Lock()?; // Lock the graphics

        // - Create a window
        let mut Window = Graphics::Get_instance().Create_window()?;

        Window.Set_icon("De", Color_type::Black);

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
            let Logo = Create_logo(Window.Get_object(), 4, Color_type::Black)?;

            LVGL::lv_obj_set_align(Logo, LVGL::lv_align_t_LV_ALIGN_CENTER);
            LVGL::lv_obj_add_flag(Logo, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_OVERFLOW_VISIBLE);

            // Set shadow color according to BG color
            for i in 0..4 {
                let Part = LVGL::lv_obj_get_child(Logo, i);

                LVGL::lv_obj_set_style_bg_opa(Part, LVGL::LV_OPA_0 as u8, LVGL::LV_STATE_DEFAULT);

                LVGL::lv_obj_set_style_border_width(Part, 2, LVGL::LV_STATE_DEFAULT);
            }
        }

        // - Create a tile view
        let Tile_view = unsafe {
            let Tile_view = LVGL::lv_tileview_create(Window.Get_object());

            if Tile_view.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_style_bg_opa(Tile_view, LVGL::LV_OPA_0 as u8, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_scrollbar_mode(
                Tile_view,
                LVGL::lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_OFF,
            );

            Tile_view
        };

        // - Create the desk tile
        let Desk_tile = unsafe {
            let Desk = LVGL::lv_tileview_add_tile(Tile_view, 0, 0, LVGL::lv_dir_t_LV_DIR_BOTTOM);

            if Desk.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_style_pad_all(Desk, 20, LVGL::LV_STATE_DEFAULT);

            Desk
        };

        // - Create the drawer tile
        let Drawer_tile = unsafe {
            let Drawer = LVGL::lv_tileview_add_tile(Tile_view, 0, 1, LVGL::lv_dir_t_LV_DIR_TOP);

            if Drawer.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_style_pad_top(Drawer, 40, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_pad_bottom(Drawer, 40, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_pad_left(Drawer, 40, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_flex_flow(Drawer, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_ROW_WRAP);

            Drawer
        };

        // - Create a dock
        let Dock = unsafe {
            let Dock = LVGL::lv_obj_create(Desk_tile);

            if Dock.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_style_bg_color(Dock, Color_type::Black.into(), LVGL::LV_STATE_DEFAULT);

            LVGL::lv_obj_set_align(Dock, LVGL::lv_align_t_LV_ALIGN_BOTTOM_MID);
            LVGL::lv_obj_set_size(Dock, LVGL::LV_SIZE_CONTENT, LVGL::LV_SIZE_CONTENT);
            LVGL::lv_obj_set_style_border_width(Dock, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_flex_flow(Dock, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_ROW);
            LVGL::lv_obj_set_style_bg_opa(Dock, LVGL::LV_OPA_50 as u8, LVGL::LV_STATE_DEFAULT);

            LVGL::lv_obj_set_style_pad_all(Dock, 12, LVGL::LV_STATE_DEFAULT);

            Dock
        };

        // - Create the main button
        let Main_button = unsafe { Create_logo(Dock, 1, Color_type::White)? };

        let Shortcuts = BTreeMap::new();

        let Desk = Self {
            Window,
            Tile_view,
            Desk_tile,
            Drawer_tile,
            Dock,
            Main_button,
            Shortcuts,
        };

        Ok(Desk)
    }

    unsafe fn Create_drawer_shortcut(
        &mut self,
        Entry_name: &str,
        Name: &str,
        Icon_color: Color_type,
        Icon_string: &str,
        Drawer: *mut LVGL::lv_obj_t,
    ) -> Result_type<()> {
        let Icon = unsafe {
            let Container = LVGL::lv_obj_create(Drawer);

            LVGL::lv_obj_set_size(Container, 12 * 8, 11 * 8);
            LVGL::lv_obj_set_style_bg_opa(Container, LVGL::LV_OPA_0 as u8, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_border_width(Container, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_flex_flow(Container, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN);
            LVGL::lv_obj_set_style_pad_all(Container, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_flex_align(
                Container,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_SPACE_EVENLY,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
            );

            let Icon = Create_icon(Container, Icon_color, Icon_string, Self::Drawer_icon_size)?;

            let Label = LVGL::lv_label_create(Container);

            let Name = CString::new(Name).map_err(Error_type::Null_character_in_string)?;

            LVGL::lv_label_set_text(Label, Name.as_ptr());

            Icon
        };

        self.Shortcuts.insert(Icon, Entry_name.to_string());

        Ok(())
    }

    unsafe fn Create_drawer_interface(&mut self, Drawer: *mut LVGL::lv_obj_t) -> Result_type<()> {
        let Task = Task::Get_instance()
            .Get_current_task_identifier()
            .map_err(Error_type::Failed_to_get_current_task_identifier)?;

        let Virtual_file_system = Virtual_file_system::Get_instance();

        let _ = Virtual_file_system.Create_directory(&Shortcut_path, Task);

        let mut Buffer: Vec<u8> = vec![];

        let Shortcuts_directory = Directory_type::Open(Virtual_file_system, Shortcut_path)
            .map_err(Error_type::Failed_to_read_shortcut_directory)?;

        for Shortcut_entry in Shortcuts_directory {
            if Shortcut_entry.Get_type() != Type_type::File {
                continue;
            }

            if !Shortcut_entry.Get_name().ends_with(".json") {
                continue;
            }

            match Shortcut_type::Read(Shortcut_entry.Get_name(), &mut Buffer) {
                Ok(Shortcut) => {
                    self.Create_drawer_shortcut(
                        Shortcut_entry.Get_name(),
                        Shortcut.Get_name(),
                        Shortcut.Get_icon_color(),
                        Shortcut.Get_icon_string(),
                        Drawer,
                    )?;
                }
                Err(_) => {
                    // ? : Log error ?
                    continue;
                }
            }
        }

        Ok(())
    }

    fn Execute_shortcut(&self, Shortcut_name: &str) -> Result_type<()> {
        let Task = Task::Get_instance()
            .Get_current_task_identifier()
            .map_err(Error_type::Failed_to_get_current_task_identifier)?;

        let mut Buffer = vec![];

        let Shortcut = Shortcut_type::Read(Shortcut_name, &mut Buffer)?;

        let Standard_in = Virtual_file_system::Get_instance()
            .Open(&"/Devices/Null", Mode_type::Read_only.into(), Task)
            .map_err(Error_type::Failed_to_open_standard_file)?;

        let Standard_out = Virtual_file_system::Get_instance()
            .Open(&"/Devices/Null", Mode_type::Write_only.into(), Task)
            .map_err(Error_type::Failed_to_open_standard_file)?;

        let Standard_err = Virtual_file_system::Get_instance()
            .Open(&"/Devices/Null", Mode_type::Write_only.into(), Task)
            .map_err(Error_type::Failed_to_open_standard_file)?;

        Executable::Execute(
            Shortcut.Get_command(),
            Shortcut.Get_arguments().to_string(),
            Standard_type::New(
                Standard_in,
                Standard_out,
                Standard_err,
                Task,
                Virtual_file_system::Get_instance(),
            ),
        )
        .map_err(Error_type::Failed_to_execute_shortcut)?;

        Ok(())
    }

    fn Refresh_dock(&self) -> Result_type<()> {
        let Dock_child_count = unsafe { LVGL::lv_obj_get_child_count(self.Dock) };

        let Graphics_manager = Graphics::Get_instance();

        let Window_count = Graphics_manager.Get_window_count()?;

        // Remove the icons of the windows that are not in the dock
        for i in 1..Dock_child_count {
            let Icon = unsafe { LVGL::lv_obj_get_child(self.Dock, i as i32) };

            if Icon == self.Main_button {
                continue;
            }

            let Dock_window_identifier = unsafe { LVGL::lv_obj_get_user_data(Icon) as usize };

            let Found = (1..Window_count).find(|&i| {
                if let Ok(Window_identifier) = Graphics_manager.Get_window_identifier(i) {
                    Window_identifier == Dock_window_identifier
                } else {
                    false
                }
            });

            if Found.is_none() {
                unsafe {
                    LVGL::lv_obj_delete(Icon);
                }
            }
        }

        // Add the new icons
        for i in 1..Window_count {
            let Window_identifier =
                if let Ok(Window_identifier) = Graphics_manager.Get_window_identifier(i) {
                    Window_identifier
                } else {
                    continue;
                };

            // Find the index of the window in the dock
            let Found = (1..Dock_child_count).find(|&i| {
                let Dock_window_identifier = unsafe {
                    let Icon = LVGL::lv_obj_get_child(self.Dock, i as i32);

                    LVGL::lv_obj_get_user_data(Icon) as usize
                };

                Dock_window_identifier == Window_identifier
            });

            // If the window is not in the dock, add it
            if Found.is_none() {
                let (Icon_string, Icon_color) = Graphics_manager.Get_window_icon(i)?;

                let Window_identifier = Graphics_manager.Get_window_identifier(i)?;

                unsafe {
                    let Icon =
                        Create_icon(self.Dock, Icon_color, &Icon_string, Self::Dock_icon_size)?;

                    LVGL::lv_obj_set_user_data(Icon, Window_identifier as *mut c_void);
                }
            }
        }

        Ok(())
    }

    pub fn Event_handler(&mut self) {
        let _Lock = Graphics::Get_instance().Lock().unwrap();
        while let Some(Event) = self.Window.Pop_event() {
            match Event.Get_code() {
                Self::Home_event => unsafe {
                    LVGL::lv_tileview_set_tile_by_index(
                        self.Tile_view,
                        0,
                        0,
                        LVGL::lv_anim_enable_t_LV_ANIM_ON,
                    );
                },
                Event_code_type::Value_changed => {
                    if Event.Get_target() == self.Tile_view {
                        unsafe {
                            if LVGL::lv_tileview_get_tile_active(self.Tile_view) == self.Desk_tile {
                                LVGL::lv_obj_clean(self.Drawer_tile);
                            } else if LVGL::lv_obj_get_child_count(self.Drawer_tile) == 0 {
                                let _ = self.Create_drawer_interface(self.Drawer_tile);
                            }
                        }
                    }
                }
                Event_code_type::Clicked => {
                    // If the target is a shortcut, execute the shortcut
                    if let Some(Shortcut_name) = self.Shortcuts.get(&Event.Get_target()) {
                        if let Err(Error) = self.Execute_shortcut(Shortcut_name) {
                            // ? : Log error ?
                            todo!("Failed to execute shortcut {}", Error.to_string());
                        }
                    }
                    // If the target is a dock icon, move the window to the foreground
                    else if unsafe { LVGL::lv_obj_get_parent(Event.Get_target()) == self.Dock } {
                        // Ignore the main button
                        if Event.Get_target() == self.Main_button {
                            continue;
                        }

                        let Window_identifier =
                            unsafe { LVGL::lv_obj_get_user_data(Event.Get_target()) as usize };

                        Graphics::Get_instance()
                            .Maximize_window(Window_identifier)
                            .unwrap();
                    }
                }
                Event_code_type::Pressed => {
                    if Event.Get_target() == self.Main_button
                        || unsafe {
                            LVGL::lv_obj_get_parent(Event.Get_target()) == self.Main_button
                        }
                    {
                        unsafe {
                            LVGL::lv_obj_add_state(self.Main_button, LVGL::LV_STATE_PRESSED as u16);
                            for i in 0..4 {
                                let Part = LVGL::lv_obj_get_child(self.Main_button, i);

                                LVGL::lv_obj_add_state(Part, LVGL::LV_STATE_PRESSED as u16);
                            }
                        }
                    }
                }
                Event_code_type::Released => {
                    if Event.Get_target() == self.Main_button
                        || unsafe {
                            LVGL::lv_obj_get_parent(Event.Get_target()) == self.Main_button
                        }
                    {
                        const State: u16 = LVGL::LV_STATE_PRESSED as u16;

                        unsafe {
                            LVGL::lv_obj_add_state(self.Main_button, State);
                            for i in 0..4 {
                                let Part = LVGL::lv_obj_get_child(self.Main_button, i);

                                LVGL::lv_obj_remove_state(Part, State);
                            }
                        }

                        unsafe {
                            LVGL::lv_tileview_set_tile_by_index(
                                self.Tile_view,
                                0,
                                1,
                                LVGL::lv_anim_enable_t_LV_ANIM_ON,
                            );
                        }
                    }
                }
                Windows_parent_child_changed => {
                    // Ignore consecutive windows parent child changed events
                    if let Some(Peeked_event) = self.Window.Peek_event() {
                        if Peeked_event.Get_code() == Windows_parent_child_changed {
                            continue;
                        }
                    }

                    self.Refresh_dock().unwrap();
                }
                _ => {}
            }
        }
    }
}

unsafe fn Create_logo(
    Parent: *mut LVGL::lv_obj_t,
    Factor: u8,
    Color: Color_type,
) -> Result_type<*mut LVGL::lv_obj_t> {
    let Logo = LVGL::lv_button_create(Parent);

    if Logo.is_null() {
        return Err(Error_type::Failed_to_create_object);
    }

    LVGL::lv_obj_set_size(Logo, 32 * Factor as i32, 32 * Factor as i32);
    LVGL::lv_obj_set_style_bg_opa(Logo, LVGL::LV_OPA_0 as u8, LVGL::LV_STATE_DEFAULT);
    LVGL::lv_obj_set_style_pad_all(Logo, 0, LVGL::LV_STATE_DEFAULT);
    LVGL::lv_obj_set_style_radius(Logo, 0, LVGL::LV_STATE_DEFAULT);
    LVGL::lv_obj_set_style_border_width(Logo, 0, LVGL::LV_STATE_DEFAULT);

    New_part(Logo, LVGL::lv_align_t_LV_ALIGN_TOP_RIGHT, Factor, Color)?;
    New_part(Logo, LVGL::lv_align_t_LV_ALIGN_BOTTOM_RIGHT, Factor, Color)?;
    New_part(Logo, LVGL::lv_align_t_LV_ALIGN_BOTTOM_LEFT, Factor, Color)?;
    New_part(Logo, LVGL::lv_align_t_LV_ALIGN_TOP_LEFT, Factor, Color)?;

    Ok(Logo)
}

fn New_part(
    Parent: *mut LVGL::lv_obj_t,
    Alignment: LVGL::lv_align_t,
    Factor: u8,
    Color: Color_type,
) -> Result_type<*mut LVGL::lv_obj_t> {
    //let Color = match Alignment {
    //    LVGL::lv_align_t_LV_ALIGN_TOP_RIGHT => LVGL::lv_palette_t_LV_PALETTE_YELLOW,
    //    LVGL::lv_align_t_LV_ALIGN_BOTTOM_LEFT => LVGL::lv_palette_t_LV_PALETTE_BLUE,
    //    LVGL::lv_align_t_LV_ALIGN_BOTTOM_RIGHT => LVGL::lv_palette_t_LV_PALETTE_GREEN,
    //    LVGL::lv_align_t_LV_ALIGN_TOP_LEFT => LVGL::lv_palette_t_LV_PALETTE_RED,
    //    _ => LVGL::lv_palette_t_LV_PALETTE_GREY,
    //};

    let Size = (10_i32 * Factor as i32, 21_i32 * Factor as i32);

    unsafe {
        let Part = LVGL::lv_button_create(Parent);

        if Part.is_null() {
            return Err(Error_type::Failed_to_create_object);
        }

        LVGL::lv_obj_set_style_bg_color(Part, Color.Into_LVGL_color(), LVGL::LV_STATE_DEFAULT);
        LVGL::lv_obj_set_style_bg_color(Part, LVGL::lv_color_white(), LVGL::LV_STATE_PRESSED);

        LVGL::lv_obj_set_align(Part, Alignment);

        match Alignment {
            LVGL::lv_align_t_LV_ALIGN_TOP_LEFT | LVGL::lv_align_t_LV_ALIGN_BOTTOM_RIGHT => {
                LVGL::lv_obj_set_size(Part, Size.0, Size.1);
            }
            LVGL::lv_align_t_LV_ALIGN_BOTTOM_LEFT | LVGL::lv_align_t_LV_ALIGN_TOP_RIGHT => {
                LVGL::lv_obj_set_size(Part, Size.1, Size.0);
            }
            _ => {}
        }

        LVGL::lv_obj_set_style_pad_all(Part, 0, LVGL::LV_STATE_DEFAULT);
        LVGL::lv_obj_set_style_radius(Part, 0, LVGL::LV_STATE_DEFAULT);
        LVGL::lv_obj_set_style_border_width(Part, 0, LVGL::LV_STATE_DEFAULT);
        LVGL::lv_obj_add_flag(Part, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_EVENT_BUBBLE);

        Ok(Part)
    }
}
