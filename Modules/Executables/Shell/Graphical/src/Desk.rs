use crate::{
    Error::{Error_type, Result_type},
    Icon::Create_icon,
    Shell_type,
};

use Graphics::{Point_type, Window_type, LVGL};

pub struct Desk_type {
    Window: Window_type,
    Tile_view: *mut LVGL::lv_obj_t,
    Drawer_tile: *mut LVGL::lv_obj_t,
    Desk_tile: *mut LVGL::lv_obj_t,
    Dock: *mut LVGL::lv_obj_t,
    Main_button: *mut LVGL::lv_obj_t,
    Parts: [*mut LVGL::lv_obj_t; 4],
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

    pub fn Get_window_object(&self) -> *mut LVGL::lv_obj_t {
        self.Window.Get_object()
    }

    pub fn Is_hidden(&self) -> bool {
        unsafe { LVGL::lv_obj_has_flag(self.Dock, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_HIDDEN) }
    }

    pub fn New() -> Result_type<Self> {
        let _Lock = Graphics::Get_instance().Lock()?; // Lock the graphics

        // - Create a window
        let Window = Graphics::Get_instance().Create_window()?;

        unsafe {
            LVGL::lv_obj_set_style_pad_all(Window.Get_object(), 0, LVGL::LV_STATE_DEFAULT);
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
            let Desk = LVGL::lv_tileview_add_tile(Tile_view, 0, 0, LVGL::lv_dir_t_LV_DIR_NONE);

            if Desk.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_style_pad_all(Desk, 20, LVGL::LV_STATE_DEFAULT);

            Desk
        };

        // - Create the drawer tile
        let Drawer_tile = unsafe {
            let Drawer = LVGL::lv_tileview_add_tile(Tile_view, 1, 0, LVGL::lv_dir_t_LV_DIR_LEFT);

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

            LVGL::lv_obj_set_style_bg_color(Dock, LVGL::lv_color_black(), LVGL::LV_STATE_DEFAULT);

            LVGL::lv_obj_set_align(Dock, LVGL::lv_align_t_LV_ALIGN_BOTTOM_MID);
            LVGL::lv_obj_set_size(Dock, LVGL::LV_SIZE_CONTENT, LVGL::LV_SIZE_CONTENT);
            LVGL::lv_obj_set_style_border_width(Dock, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_flex_flow(Dock, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_ROW);
            LVGL::lv_obj_set_style_bg_opa(Dock, LVGL::LV_OPA_80 as u8, LVGL::LV_STATE_DEFAULT);

            LVGL::lv_obj_set_style_pad_all(Dock, 12, LVGL::LV_STATE_DEFAULT);

            Dock
        };

        // - Create the main button
        let Main_button = unsafe {
            let Main_button = LVGL::lv_obj_create(Dock);

            if Main_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_size(Main_button, 32, 32);
            LVGL::lv_obj_set_style_bg_opa(
                Main_button,
                LVGL::LV_OPA_0 as u8,
                LVGL::LV_STATE_DEFAULT,
            );
            LVGL::lv_obj_set_style_pad_all(Main_button, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_radius(Main_button, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_border_width(Main_button, 0, LVGL::LV_STATE_DEFAULT);

            Main_button
        };

        let Red_part = New_part(Main_button, LVGL::lv_align_t_LV_ALIGN_TOP_LEFT);
        let Blue_part = New_part(Main_button, LVGL::lv_align_t_LV_ALIGN_BOTTOM_LEFT);
        let Green_part = New_part(Main_button, LVGL::lv_align_t_LV_ALIGN_BOTTOM_RIGHT);
        let Yellow_part = New_part(Main_button, LVGL::lv_align_t_LV_ALIGN_TOP_RIGHT);

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
            Parts: [Red_part, Blue_part, Green_part, Yellow_part],
        })
    }

    unsafe fn Create_drawer_interface(Drawer: *mut LVGL::lv_obj_t) -> Result_type<()> {
        let _Lock = Graphics::Get_instance().Lock()?; // Lock the graphics

        for i in 0..67 {
            unsafe {
                let Container = LVGL::lv_obj_create(Drawer);

                LVGL::lv_obj_set_size(Container, 12 * 8, 11 * 8);
                LVGL::lv_obj_set_style_border_width(Container, 0, LVGL::LV_STATE_DEFAULT);
                LVGL::lv_obj_set_flex_flow(Container, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN);
                LVGL::lv_obj_set_style_pad_all(Container, 0, LVGL::LV_STATE_DEFAULT);
                LVGL::lv_obj_set_flex_align(
                    Container,
                    LVGL::lv_flex_align_t_LV_FLEX_ALIGN_SPACE_EVENLY,
                    LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                    LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                );

                let Icon = Create_icon(Container, &format!("Icon {}", i), Self::Drawer_icon_size)?;

                let Label = LVGL::lv_label_create(Container);

                LVGL::lv_label_set_text(Label, format!("Label {}\0", i).as_ptr() as *const i8)
            }
        }

        Ok(())
    }

    pub fn Event_handler(&mut self) {
        while let Some(Event) = self.Window.Pop_event() {
            match Event.Code {
                LVGL::lv_event_code_t_LV_EVENT_VALUE_CHANGED => {
                    if Event.Target == self.Tile_view {
                        unsafe {
                            if LVGL::lv_tileview_get_tile_active(self.Tile_view) == self.Desk_tile {
                                let _Lock = Graphics::Get_instance().Lock().unwrap();

                                LVGL::lv_obj_clean(self.Drawer_tile);
                            } else {
                                let _ = Self::Create_drawer_interface(self.Drawer_tile);
                            }
                        }
                        println!("Tile view value changed");
                    }
                }
                LVGL::lv_event_code_t_LV_EVENT_PRESSED => {
                    if Event.Target == self.Main_button
                        || self.Parts.iter().any(|&Part| Part == Event.Target)
                    {
                        const State: u16 = LVGL::LV_STATE_PRESSED as u16;

                        self.Parts.iter().for_each(|&Part| unsafe {
                            LVGL::lv_obj_add_state(Part, State);
                        });
                    }
                }
                LVGL::lv_event_code_t_LV_EVENT_RELEASED => {
                    if Event.Target == self.Main_button
                        || self.Parts.iter().any(|&Part| Part == Event.Target)
                    {
                        const State: u16 = LVGL::LV_STATE_PRESSED as u16;

                        self.Parts.iter().for_each(|&Part| unsafe {
                            LVGL::lv_obj_remove_state(Part, State);
                        });

                        unsafe {
                            LVGL::lv_tileview_set_tile_by_index(
                                self.Tile_view,
                                1,
                                0,
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

fn New_part(Parent: *mut LVGL::lv_obj_t, Alignment: LVGL::lv_align_t) -> *mut LVGL::lv_obj_t {
    let Color = match Alignment {
        LVGL::lv_align_t_LV_ALIGN_TOP_LEFT => LVGL::lv_palette_t_LV_PALETTE_RED,
        LVGL::lv_align_t_LV_ALIGN_BOTTOM_LEFT => LVGL::lv_palette_t_LV_PALETTE_BLUE,
        LVGL::lv_align_t_LV_ALIGN_BOTTOM_RIGHT => LVGL::lv_palette_t_LV_PALETTE_GREEN,
        LVGL::lv_align_t_LV_ALIGN_TOP_RIGHT => LVGL::lv_palette_t_LV_PALETTE_YELLOW,
        _ => LVGL::lv_palette_t_LV_PALETTE_GREY,
    };

    let Size = (10_i32, 21_i32);

    unsafe {
        let Part = LVGL::lv_obj_create(Parent);

        if Part.is_null() {
            return Part;
        }

        let Color = LVGL::lv_palette_main(Color);

        LVGL::lv_obj_set_style_bg_color(Part, Color, LVGL::LV_STATE_DEFAULT);
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

        Part
    }
}
