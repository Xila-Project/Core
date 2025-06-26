use alloc::{
    borrow::ToOwned,
    ffi::CString,
    string::{String, ToString},
    vec::Vec,
};
use core::{ptr::null_mut, time::Duration};

use File_system::{Path_owned_type, Path_type, Type_type};
use Graphics::LVGL;
use Virtual_file_system::{Directory_type, Get_instance};

use crate::Error::{Error_type, Result_type};

pub struct File_manager_type {
    Window: *mut LVGL::lv_obj_t,
    Title_bar: *mut LVGL::lv_obj_t,
    Title_label: *mut LVGL::lv_obj_t,
    Close_button: *mut LVGL::lv_obj_t,
    Toolbar: *mut LVGL::lv_obj_t,
    Up_button: *mut LVGL::lv_obj_t,
    Home_button: *mut LVGL::lv_obj_t,
    Refresh_button: *mut LVGL::lv_obj_t,
    Path_label: *mut LVGL::lv_obj_t,
    File_list: *mut LVGL::lv_obj_t,
    Current_path: Path_owned_type,
    Files: Vec<File_item_type>,
    Running: bool,
}

#[derive(Clone)]
pub struct File_item_type {
    pub Name: String,
    pub Type: Type_type,
    pub Size: u64,
    pub Button: *mut LVGL::lv_obj_t,
    pub Icon: *mut LVGL::lv_obj_t,
    pub Label: *mut LVGL::lv_obj_t,
}

impl Drop for File_manager_type {
    fn drop(&mut self) {
        unsafe {
            if !self.Window.is_null() {
                LVGL::lv_obj_delete_async(self.Window);
            }
        }
    }
}

impl File_manager_type {
    pub async fn New() -> Result_type<Self> {
        let _Lock = Graphics::Get_instance().Lock().await;

        let Screen = unsafe { LVGL::lv_screen_active() };

        let mut Manager = Self {
            Window: null_mut(),
            Title_bar: null_mut(),
            Title_label: null_mut(),
            Close_button: null_mut(),
            Toolbar: null_mut(),
            Up_button: null_mut(),
            Home_button: null_mut(),
            Refresh_button: null_mut(),
            Path_label: null_mut(),
            File_list: null_mut(),
            Current_path: Path_owned_type::Root(),
            Files: Vec::new(),
            Running: true,
        };

        Manager.Create_window(Screen).await?;
        Manager.Create_title_bar().await?;
        Manager.Create_toolbar().await?;
        Manager.Create_file_list().await?;
        Manager.Load_directory().await?;

        Ok(Manager)
    }

    pub async fn Run(&mut self) {
        while self.Running {
            // Handle refresh requests from button clicks
            if self.Should_refresh() {
                let _ = self.Load_directory().await;
            }

            Task::Manager_type::Sleep(Duration::from_millis(50)).await;
        }
    }

    fn Should_refresh(&self) -> bool {
        // This is a placeholder - in a real implementation you'd track refresh requests
        false
    }

    async fn Create_window(&mut self, Parent: *mut LVGL::lv_obj_t) -> Result_type<()> {
        unsafe {
            self.Window = LVGL::lv_obj_create(Parent);
            if self.Window.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            // Set window properties
            LVGL::lv_obj_set_size(self.Window, LVGL::lv_pct(90), LVGL::lv_pct(90));
            LVGL::lv_obj_center(self.Window);
            LVGL::lv_obj_set_style_radius(self.Window, 10, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_shadow_width(self.Window, 20, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_shadow_opa(
                self.Window,
                LVGL::LV_OPA_30 as u8,
                LVGL::LV_STATE_DEFAULT,
            );

            // Set background color
            LVGL::lv_obj_set_style_bg_color(
                self.Window,
                LVGL::lv_color_hex(0x2D2D2D),
                LVGL::LV_STATE_DEFAULT,
            );

            // Remove default padding
            LVGL::lv_obj_set_style_pad_all(self.Window, 0, LVGL::LV_STATE_DEFAULT);

            // Set layout to flex column
            LVGL::lv_obj_set_layout(self.Window, LVGL::lv_layout_t_LV_LAYOUT_FLEX);
            LVGL::lv_obj_set_flex_flow(self.Window, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN);
        }

        Ok(())
    }

    async fn Create_title_bar(&mut self) -> Result_type<()> {
        unsafe {
            self.Title_bar = LVGL::lv_obj_create(self.Window);
            if self.Title_bar.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            // Title bar properties
            LVGL::lv_obj_set_size(self.Title_bar, LVGL::lv_pct(100), 40);
            LVGL::lv_obj_set_style_bg_color(
                self.Title_bar,
                LVGL::lv_color_hex(0x404040),
                LVGL::LV_STATE_DEFAULT,
            );
            LVGL::lv_obj_set_style_border_width(self.Title_bar, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_radius(self.Title_bar, 10, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_radius(
                self.Title_bar,
                0,
                LVGL::LV_STATE_DEFAULT | LVGL::LV_PART_MAIN,
            );
            LVGL::lv_obj_set_style_pad_all(self.Title_bar, 10, LVGL::LV_STATE_DEFAULT);

            // Title label
            self.Title_label = LVGL::lv_label_create(self.Title_bar);
            if self.Title_label.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            let Title_text = CString::new("File Manager").unwrap();
            LVGL::lv_label_set_text(self.Title_label, Title_text.as_ptr());
            LVGL::lv_obj_set_style_text_color(
                self.Title_label,
                LVGL::lv_color_white(),
                LVGL::LV_STATE_DEFAULT,
            );
            LVGL::lv_obj_align(self.Title_label, LVGL::lv_align_t_LV_ALIGN_LEFT_MID, 0, 0);

            // Close button
            self.Close_button = LVGL::lv_button_create(self.Title_bar);
            if self.Close_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_size(self.Close_button, 30, 30);
            LVGL::lv_obj_align(self.Close_button, LVGL::lv_align_t_LV_ALIGN_RIGHT_MID, 0, 0);
            LVGL::lv_obj_set_style_bg_color(
                self.Close_button,
                LVGL::lv_color_hex(0xFF4444),
                LVGL::LV_STATE_DEFAULT,
            );

            let Close_label = LVGL::lv_label_create(self.Close_button);
            let Close_text = CString::new("×").unwrap();
            LVGL::lv_label_set_text(Close_label, Close_text.as_ptr());
            LVGL::lv_obj_set_style_text_color(
                Close_label,
                LVGL::lv_color_white(),
                LVGL::LV_STATE_DEFAULT,
            );
            LVGL::lv_obj_center(Close_label);

            // Add event handler for close button
            LVGL::lv_obj_add_event_cb(
                self.Close_button,
                Some(Self::Close_event_handler),
                LVGL::lv_event_code_t_LV_EVENT_CLICKED,
                self as *mut Self as *mut core::ffi::c_void,
            );
        }

        Ok(())
    }

    async fn Create_toolbar(&mut self) -> Result_type<()> {
        unsafe {
            self.Toolbar = LVGL::lv_obj_create(self.Window);
            if self.Toolbar.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            // Toolbar properties
            LVGL::lv_obj_set_size(self.Toolbar, LVGL::lv_pct(100), 50);
            LVGL::lv_obj_set_style_bg_color(
                self.Toolbar,
                LVGL::lv_color_hex(0x353535),
                LVGL::LV_STATE_DEFAULT,
            );
            LVGL::lv_obj_set_style_border_width(self.Toolbar, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_pad_all(self.Toolbar, 10, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_layout(self.Toolbar, LVGL::lv_layout_t_LV_LAYOUT_FLEX);
            LVGL::lv_obj_set_flex_flow(self.Toolbar, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_ROW);

            // Up button
            self.Up_button = LVGL::lv_button_create(self.Toolbar);
            if self.Up_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_size(self.Up_button, 40, 30);
            let Up_label = LVGL::lv_label_create(self.Up_button);
            let Up_text = CString::new("↑").unwrap();
            LVGL::lv_label_set_text(Up_label, Up_text.as_ptr());
            LVGL::lv_obj_center(Up_label);

            LVGL::lv_obj_add_event_cb(
                self.Up_button,
                Some(Self::Up_event_handler),
                LVGL::lv_event_code_t_LV_EVENT_CLICKED,
                self as *mut Self as *mut core::ffi::c_void,
            );

            // Home button
            self.Home_button = LVGL::lv_button_create(self.Toolbar);
            if self.Home_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_size(self.Home_button, 40, 30);
            let Home_label = LVGL::lv_label_create(self.Home_button);
            let Home_text = CString::new("🏠").unwrap();
            LVGL::lv_label_set_text(Home_label, Home_text.as_ptr());
            LVGL::lv_obj_center(Home_label);

            LVGL::lv_obj_add_event_cb(
                self.Home_button,
                Some(Self::Home_event_handler),
                LVGL::lv_event_code_t_LV_EVENT_CLICKED,
                self as *mut Self as *mut core::ffi::c_void,
            );

            // Refresh button
            self.Refresh_button = LVGL::lv_button_create(self.Toolbar);
            if self.Refresh_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_size(self.Refresh_button, 40, 30);
            let Refresh_label = LVGL::lv_label_create(self.Refresh_button);
            let Refresh_text = CString::new("⟳").unwrap();
            LVGL::lv_label_set_text(Refresh_label, Refresh_text.as_ptr());
            LVGL::lv_obj_center(Refresh_label);

            LVGL::lv_obj_add_event_cb(
                self.Refresh_button,
                Some(Self::Refresh_event_handler),
                LVGL::lv_event_code_t_LV_EVENT_CLICKED,
                self as *mut Self as *mut core::ffi::c_void,
            );

            // Path label
            self.Path_label = LVGL::lv_label_create(self.Toolbar);
            if self.Path_label.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_style_text_color(
                self.Path_label,
                LVGL::lv_color_white(),
                LVGL::LV_STATE_DEFAULT,
            );
            LVGL::lv_obj_set_flex_grow(self.Path_label, 1);
            LVGL::lv_obj_set_style_pad_left(self.Path_label, 10, LVGL::LV_STATE_DEFAULT);

            self.Update_path_label();
        }

        Ok(())
    }

    async fn Create_file_list(&mut self) -> Result_type<()> {
        unsafe {
            self.File_list = LVGL::lv_obj_create(self.Window);
            if self.File_list.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            // File list properties
            LVGL::lv_obj_set_size(self.File_list, LVGL::lv_pct(100), LVGL::lv_pct(100));
            LVGL::lv_obj_set_style_bg_color(
                self.File_list,
                LVGL::lv_color_hex(0x2D2D2D),
                LVGL::LV_STATE_DEFAULT,
            );
            LVGL::lv_obj_set_style_border_width(self.File_list, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_pad_all(self.File_list, 10, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_layout(self.File_list, LVGL::lv_layout_t_LV_LAYOUT_FLEX);
            LVGL::lv_obj_set_flex_flow(self.File_list, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN);
            LVGL::lv_obj_set_flex_grow(self.File_list, 1);

            // Make it scrollable
            LVGL::lv_obj_set_scrollbar_mode(
                self.File_list,
                LVGL::lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_AUTO,
            );
        }

        Ok(())
    }

    async fn Load_directory(&mut self) -> Result_type<()> {
        // Clear existing files
        self.Clear_file_list();

        // Open directory
        let Virtual_file_system = Get_instance();
        let Directory = Directory_type::Open(Virtual_file_system, &self.Current_path).await;

        match Directory {
            Ok(Directory) => {
                // Read directory entries
                while let Ok(Some(Entry)) = Directory.Read().await {
                    let File_item = File_item_type {
                        Name: Entry.Get_name().clone(),
                        Type: Entry.Get_type(),
                        Size: Entry.Get_size().As_u64(),
                        Button: null_mut(),
                        Icon: null_mut(),
                        Label: null_mut(),
                    };

                    self.Files.push(File_item);
                }

                // Sort files: directories first, then files
                self.Files.sort_by(|a, b| match (a.Type, b.Type) {
                    (Type_type::Directory, Type_type::Directory) => a.Name.cmp(&b.Name),
                    (Type_type::Directory, _) => core::cmp::Ordering::Less,
                    (_, Type_type::Directory) => core::cmp::Ordering::Greater,
                    _ => a.Name.cmp(&b.Name),
                });

                // Create UI for each file
                for i in 0..self.Files.len() {
                    self.Create_file_item(i).await?;
                }

                Ok(())
            }
            Err(Error) => {
                // Show error message
                self.Show_error_message("Failed to open directory").await;
                Err(Error_type::Failed_to_read_directory(Error))
            }
        }
    }

    async fn Create_file_item(&mut self, Index: usize) -> Result_type<()> {
        unsafe {
            let File = &mut self.Files[Index];

            // Create button for file item
            File.Button = LVGL::lv_button_create(self.File_list);
            if File.Button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_size(File.Button, LVGL::lv_pct(100), 40);
            LVGL::lv_obj_set_style_bg_color(
                File.Button,
                LVGL::lv_color_hex(0x404040),
                LVGL::LV_STATE_DEFAULT,
            );
            LVGL::lv_obj_set_style_bg_color(
                File.Button,
                LVGL::lv_color_hex(0x505050),
                LVGL::LV_STATE_PRESSED,
            );
            LVGL::lv_obj_set_style_border_width(File.Button, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_radius(File.Button, 5, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_pad_all(File.Button, 10, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_layout(File.Button, LVGL::lv_layout_t_LV_LAYOUT_FLEX);
            LVGL::lv_obj_set_flex_flow(File.Button, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_ROW);

            // Create icon
            File.Icon = LVGL::lv_label_create(File.Button);
            if File.Icon.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            let Icon_text = match File.Type {
                Type_type::Directory => CString::new("📁").unwrap(),
                Type_type::File => CString::new("📄").unwrap(),
                _ => CString::new("❓").unwrap(),
            };
            LVGL::lv_label_set_text(File.Icon, Icon_text.as_ptr());
            LVGL::lv_obj_set_style_text_color(
                File.Icon,
                LVGL::lv_color_white(),
                LVGL::LV_STATE_DEFAULT,
            );

            // Create label
            File.Label = LVGL::lv_label_create(File.Button);
            if File.Label.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            let Label_text = CString::new(File.Name.clone()).unwrap();
            LVGL::lv_label_set_text(File.Label, Label_text.as_ptr());
            LVGL::lv_obj_set_style_text_color(
                File.Label,
                LVGL::lv_color_white(),
                LVGL::LV_STATE_DEFAULT,
            );
            LVGL::lv_obj_set_style_pad_left(File.Label, 10, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_flex_grow(File.Label, 1);

            // Add event handler
            LVGL::lv_obj_add_event_cb(
                File.Button,
                Some(Self::File_item_event_handler),
                LVGL::lv_event_code_t_LV_EVENT_CLICKED,
                Index as *mut core::ffi::c_void,
            );

            // Store manager reference in user data
            LVGL::lv_obj_set_user_data(File.Button, self as *mut Self as *mut core::ffi::c_void);
        }

        Ok(())
    }

    fn Clear_file_list(&mut self) {
        unsafe {
            for File in &mut self.Files {
                if !File.Button.is_null() {
                    LVGL::lv_obj_delete_async(File.Button);
                }
            }
        }
        self.Files.clear();
    }

    fn Update_path_label(&self) {
        unsafe {
            if !self.Path_label.is_null() {
                let Path_text = CString::new(self.Current_path.to_string()).unwrap();
                LVGL::lv_label_set_text(self.Path_label, Path_text.as_ptr());
            }
        }
    }

    async fn Show_error_message(&self, _Message: &str) {
        // TODO: Implement error message display
        // For now, just print to console if logging is available
    }

    // Event handlers
    unsafe extern "C" fn Close_event_handler(Event: *mut LVGL::lv_event_t) {
        let Manager = LVGL::lv_event_get_user_data(Event) as *mut File_manager_type;
        if !Manager.is_null() {
            (*Manager).Running = false;
        }
    }

    unsafe extern "C" fn Up_event_handler(Event: *mut LVGL::lv_event_t) {
        let Manager = LVGL::lv_event_get_user_data(Event) as *mut File_manager_type;
        if !Manager.is_null() {
            let Manager_ref = &mut *Manager;
            if let Some(Parent_path) = Manager_ref.Current_path.Go_parent() {
                Manager_ref.Current_path = Parent_path.to_owned();
                Manager_ref.Update_path_label();
                // We can't call async functions from C callback, so we'll need to handle this differently
                // For now, just update the label
            }
        }
    }

    unsafe extern "C" fn Home_event_handler(Event: *mut LVGL::lv_event_t) {
        let Manager = LVGL::lv_event_get_user_data(Event) as *mut File_manager_type;
        if !Manager.is_null() {
            let Manager_ref = &mut *Manager;
            Manager_ref.Current_path = Path_owned_type::Root();
            Manager_ref.Update_path_label();
        }
    }

    unsafe extern "C" fn Refresh_event_handler(Event: *mut LVGL::lv_event_t) {
        let Manager = LVGL::lv_event_get_user_data(Event) as *mut File_manager_type;
        if !Manager.is_null() {
            // Refresh would need to be handled asynchronously
        }
    }

    unsafe extern "C" fn File_item_event_handler(Event: *mut LVGL::lv_event_t) {
        let Index = LVGL::lv_event_get_user_data(Event) as usize;
        let Target = LVGL::lv_event_get_target(Event);
        let Manager = LVGL::lv_obj_get_user_data(Target as *mut _) as *mut File_manager_type;

        if !Manager.is_null() {
            let Manager_ref = &mut *Manager;
            if Index < Manager_ref.Files.len() {
                let File = &Manager_ref.Files[Index];

                if File.Type == Type_type::Directory {
                    // Navigate to directory
                    if let Some(New_path) = Manager_ref
                        .Current_path
                        .clone()
                        .Join(Path_type::From_str(&File.Name))
                    {
                        Manager_ref.Current_path = New_path;
                        Manager_ref.Update_path_label();
                        // Again, we can't call async functions from C callback
                    }
                } else {
                    // Handle file selection/opening
                    // Could open files with appropriate applications
                }
            }
        }
    }

    pub fn Show(&self) {
        unsafe {
            if !self.Window.is_null() {
                LVGL::lv_obj_remove_flag(self.Window, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_HIDDEN);
            }
        }
    }

    pub fn Hide(&self) {
        unsafe {
            if !self.Window.is_null() {
                LVGL::lv_obj_add_flag(self.Window, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_HIDDEN);
            }
        }
    }

    pub fn Is_visible(&self) -> bool {
        unsafe {
            !self.Window.is_null()
                && !LVGL::lv_obj_has_flag(self.Window, LVGL::lv_obj_flag_t_LV_OBJ_FLAG_HIDDEN)
        }
    }

    pub async fn Refresh(&mut self) -> Result_type<()> {
        self.Load_directory().await
    }

    pub async fn Navigate_to(&mut self, Path: &Path_type) -> Result_type<()> {
        self.Current_path = Path.to_owned();
        self.Update_path_label();
        self.Load_directory().await
    }

    pub fn Get_current_path(&self) -> &Path_type {
        &self.Current_path
    }

    pub fn Is_running(&self) -> bool {
        self.Running
    }
}
