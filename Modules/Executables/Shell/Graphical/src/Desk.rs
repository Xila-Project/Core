use crate::{
    Error::{Error_type, Result_type},
    Icon::Create_icon,
};

use Graphics::{Color_type, Event_code_type, Point_type, Window_type, LVGL};

pub struct Desk_type {
    Window: Window_type,
    Tile_view: *mut LVGL::lv_obj_t,
    Drawer_tile: *mut LVGL::lv_obj_t,
    Desk_tile: *mut LVGL::lv_obj_t,
    Dock: *mut LVGL::lv_obj_t,
    Main_button: *mut LVGL::lv_obj_t,
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

    pub fn New() -> Result_type<Self> {
        // - Lock the graphics
        let _Lock = Graphics::Get_instance().Lock()?; // Lock the graphics

        // - Create a window
        let Window = Graphics::Get_instance().Create_window()?;

        unsafe {
            LVGL::lv_obj_set_style_pad_all(Window.Get_object(), 0, LVGL::LV_STATE_DEFAULT);
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

        // Create some fake icons
        for i in 0..5 {
            unsafe {
                Create_icon(Dock, &format!("Icon {}", i), Self::Dock_icon_size)?;
            }
        }

        Ok(Self {
            Window,
            Desk_tile,
            Drawer_tile,
            Tile_view,
            Dock,
            Main_button,
        })
    }

    unsafe fn Create_drawer_interface(Drawer: *mut LVGL::lv_obj_t) -> Result_type<()> {
        for i in 0..67 {
            unsafe {
                let Container = LVGL::lv_obj_create(Drawer);

                LVGL::lv_obj_set_size(Container, 12 * 8, 11 * 8);
                LVGL::lv_obj_set_style_bg_opa(
                    Container,
                    LVGL::LV_OPA_0 as u8,
                    LVGL::LV_STATE_DEFAULT,
                );
                LVGL::lv_obj_set_style_border_width(Container, 0, LVGL::LV_STATE_DEFAULT);
                LVGL::lv_obj_set_flex_flow(Container, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN);
                LVGL::lv_obj_set_style_pad_all(Container, 0, LVGL::LV_STATE_DEFAULT);
                LVGL::lv_obj_set_flex_align(
                    Container,
                    LVGL::lv_flex_align_t_LV_FLEX_ALIGN_SPACE_EVENLY,
                    LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                    LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                );

                Create_icon(Container, &format!("Icon {}", i), Self::Drawer_icon_size)?;

                let Label = LVGL::lv_label_create(Container);

                LVGL::lv_label_set_text(Label, format!("Label {}\0", i).as_ptr() as *const i8)
            }
        }

        Ok(())
    }

    pub fn Event_handler(&mut self) {
        while let Some(Event) = self.Window.Pop_event() {
            match Event.Get_code() {
                Self::Home_event => unsafe {
                    let _Lock = Graphics::Get_instance().Lock().unwrap();

                    LVGL::lv_tileview_set_tile_by_index(
                        self.Tile_view,
                        0,
                        0,
                        LVGL::lv_anim_enable_t_LV_ANIM_ON,
                    );
                },
                Event_code_type::Value_changed => {
                    if Event.Get_target() == self.Tile_view {
                        let _Lock = Graphics::Get_instance().Lock().unwrap();
                        unsafe {
                            if LVGL::lv_tileview_get_tile_active(self.Tile_view) == self.Desk_tile {
                                LVGL::lv_obj_clean(self.Drawer_tile);
                            } else if LVGL::lv_obj_get_child_count(self.Drawer_tile) == 0 {
                                let _ = Self::Create_drawer_interface(self.Drawer_tile);
                            }
                        }
                    }
                }
                Event_code_type::Pressed => {
                    let _Lock = Graphics::Get_instance().Lock().unwrap();
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
