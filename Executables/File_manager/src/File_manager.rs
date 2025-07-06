use alloc::{
    borrow::ToOwned,
    ffi::CString,
    string::{String, ToString},
    vec::Vec,
};
use core::{ptr::null_mut, time::Duration};

use File_system::{Path_owned_type, Path_type, Type_type};
use Graphics::{
    Event_code_type,
    Palette::{self, Hue_type},
    Window_type, LVGL,
};
use Virtual_file_system::{Directory_type, Get_instance};

use crate::Error::{Error_type, Result_type};

pub struct File_manager_type {
    Window: Window_type,
    Toolbar: *mut LVGL::lv_obj_t,
    Up_button: *mut LVGL::lv_obj_t,
    Home_button: *mut LVGL::lv_obj_t,
    Refresh_button: *mut LVGL::lv_obj_t,
    Path_text_area: *mut LVGL::lv_obj_t,
    Go_button: *mut LVGL::lv_obj_t,
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
}

impl File_manager_type {
    pub async fn New() -> Result_type<Self> {
        let _Lock = Graphics::Get_instance().Lock().await;

        let mut Window = Graphics::Get_instance().Create_window().await?;

        Window.Set_icon("Fm", Palette::Get(Hue_type::Cyan, Palette::Tone_type::Main));

        let mut Manager = Self {
            Window,
            Toolbar: null_mut(),
            Up_button: null_mut(),
            Home_button: null_mut(),
            Refresh_button: null_mut(),
            Path_text_area: null_mut(),
            Go_button: null_mut(),
            File_list: null_mut(),
            Current_path: Path_owned_type::Root(),
            Files: Vec::new(),
            Running: true,
        };

        // Set up window layout for flex
        unsafe {
            LVGL::lv_obj_set_layout(
                Manager.Window.Get_object(),
                LVGL::lv_layout_t_LV_LAYOUT_FLEX,
            );
            LVGL::lv_obj_set_flex_flow(
                Manager.Window.Get_object(),
                LVGL::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN,
            );
            LVGL::lv_obj_set_style_pad_all(Manager.Window.Get_object(), 0, LVGL::LV_STATE_DEFAULT);
        }

        Manager.Create_toolbar().await?;
        Manager.Create_file_list().await?;
        Manager.Load_directory().await?;

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

            match Event.Get_code() {
                Event_code_type::Delete => {
                    if Event.Get_target() == self.Window.Get_object() {
                        self.Running = false;
                    }
                }
                Event_code_type::Clicked => {
                    let Target = Event.Get_target();

                    // Handle different button clicks
                    if Target == self.Up_button {
                        self.Handle_up_click().await;
                    } else if Target == self.Home_button {
                        self.Handle_home_click().await;
                    } else if Target == self.Refresh_button {
                        self.Handle_refresh_click().await;
                    } else if Target == self.Go_button {
                        self.Handle_go_click().await;
                    } else {
                        // Handle file item clicks
                        self.Handle_file_click(Target).await;
                    }
                }
                _ => {}
            }
        }
    }

    async fn Create_toolbar(&mut self) -> Result_type<()> {
        unsafe {
            self.Toolbar = LVGL::lv_obj_create(self.Window.Get_object());
            if self.Toolbar.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            // Toolbar properties - fixed height at top
            LVGL::lv_obj_set_size(self.Toolbar, LVGL::lv_pct(100), LVGL::LV_SIZE_CONTENT);
            LVGL::lv_obj_align(self.Toolbar, LVGL::lv_align_t_LV_ALIGN_TOP_MID, 0, 0);

            LVGL::lv_obj_set_style_border_width(self.Toolbar, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_pad_all(self.Toolbar, 10, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_layout(self.Toolbar, LVGL::lv_layout_t_LV_LAYOUT_FLEX);
            LVGL::lv_obj_set_flex_flow(self.Toolbar, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_ROW);
            LVGL::lv_obj_set_flex_align(
                self.Toolbar,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_START,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
            );

            // Up button
            self.Up_button = LVGL::lv_button_create(self.Toolbar);
            if self.Up_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }
            let Up_label = LVGL::lv_label_create(self.Up_button);
            LVGL::lv_label_set_text(Up_label, LVGL::LV_SYMBOL_UP as *const _ as *const i8);
            LVGL::lv_obj_center(Up_label);

            // Remove event handler - events bubble up to window

            // Home button
            self.Home_button = LVGL::lv_button_create(self.Toolbar);
            if self.Home_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            let Home_label = LVGL::lv_label_create(self.Home_button);
            LVGL::lv_label_set_text(Home_label, LVGL::LV_SYMBOL_HOME as *const _ as *const i8);
            LVGL::lv_obj_center(Home_label);

            // Remove event handler - events bubble up to window

            // Refresh button
            self.Refresh_button = LVGL::lv_button_create(self.Toolbar);
            if self.Refresh_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            let Refresh_label = LVGL::lv_label_create(self.Refresh_button);

            LVGL::lv_label_set_text(
                Refresh_label,
                LVGL::LV_SYMBOL_REFRESH as *const _ as *const i8,
            );
            LVGL::lv_obj_center(Refresh_label);

            // Remove event handler - events bubble up to window

            // Path text area - use flex grow to take remaining space
            self.Path_text_area = LVGL::lv_textarea_create(self.Toolbar);
            if self.Path_text_area.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_flex_grow(self.Path_text_area, 1); // Take remaining space
            LVGL::lv_obj_set_style_pad_left(self.Path_text_area, 10, LVGL::LV_STATE_DEFAULT);

            // Configure text area properties
            LVGL::lv_textarea_set_one_line(self.Path_text_area, true);
            LVGL::lv_textarea_set_cursor_click_pos(self.Path_text_area, true);

            // Go button
            self.Go_button = LVGL::lv_button_create(self.Toolbar);
            if self.Go_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            let Go_label = LVGL::lv_label_create(self.Go_button);
            LVGL::lv_label_set_text(Go_label, LVGL::LV_SYMBOL_RIGHT as *const _ as *const i8);
            LVGL::lv_obj_center(Go_label);

            self.Update_path_label();
        }

        Ok(())
    }

    async fn Create_file_list(&mut self) -> Result_type<()> {
        unsafe {
            self.File_list = LVGL::lv_list_create(self.Window.Get_object());
            if self.File_list.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            // File list properties - use flex grow to fill remaining space
            LVGL::lv_obj_set_width(self.File_list, LVGL::lv_pct(100));
            LVGL::lv_obj_set_flex_grow(self.File_list, 1); // Take remaining vertical space

            // Ensure proper scrolling behavior
            LVGL::lv_obj_set_style_pad_all(self.File_list, 0, LVGL::LV_STATE_DEFAULT);
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
                    let Name = Entry.Get_name();

                    // Skip "." and ".." entries
                    if Name == "." || Name == ".." {
                        continue;
                    }

                    let File_item = File_item_type {
                        Name: Name.clone(),
                        Type: Entry.Get_type(),
                        Size: Entry.Get_size().As_u64(),
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
            let File = &self.Files[Index];

            let Icon_symbol = match File.Type {
                Type_type::Directory => LVGL::LV_SYMBOL_DIRECTORY,
                _ => LVGL::LV_SYMBOL_FILE,
            };

            let Name_cstring = CString::new(File.Name.clone()).unwrap();
            let Button = LVGL::lv_list_add_button(
                self.File_list,
                Icon_symbol.as_ptr() as *const core::ffi::c_void,
                Name_cstring.as_ptr(),
            );

            if Button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            // Store the file index in the button's user data for identification
            LVGL::lv_obj_set_user_data(Button, Index as *mut core::ffi::c_void);
        }

        Ok(())
    }

    fn Clear_file_list(&mut self) {
        unsafe {
            // Clear the list content - LVGL list automatically manages its children
            LVGL::lv_obj_clean(self.File_list);
        }
        self.Files.clear();
    }

    fn Update_path_label(&self) {
        unsafe {
            if !self.Path_text_area.is_null() {
                let Path_text = CString::new(self.Current_path.to_string()).unwrap();
                LVGL::lv_textarea_set_text(self.Path_text_area, Path_text.as_ptr());
            }
        }
    }

    async fn Show_error_message(&self, _Message: &str) {
        // TODO: Implement error message display
        // For now, just print to console if logging is available
    }

    async fn Handle_file_click(&mut self, Target: *mut LVGL::lv_obj_t) {
        unsafe {
            // Check if the clicked object is a file list button
            let Parent = LVGL::lv_obj_get_parent(Target as *const LVGL::lv_obj_t);
            if core::ptr::eq(Parent as *mut LVGL::lv_obj_t, self.File_list) {
                // Get the file index from user data
                let Index = LVGL::lv_obj_get_user_data(Target) as usize;

                if Index < self.Files.len() {
                    let File = &self.Files[Index];

                    if File.Type == Type_type::Directory {
                        // Navigate to directory
                        if let Some(New_path) = self
                            .Current_path
                            .clone()
                            .Join(Path_type::From_str(&File.Name))
                        {
                            self.Current_path = New_path;
                            self.Update_path_label();
                            // Reload directory contents
                            if let Err(error) = self.Load_directory().await {
                                Log::Error!("Failed to load directory: {error:?}");
                            }
                        }
                    } else {
                        // Handle file selection/opening
                        // Could open files with appropriate applications
                    }
                }
            }
        }
    }

    async fn Handle_up_click(&mut self) {
        if let Some(Parent_path) = self.Current_path.Go_parent() {
            self.Current_path = Parent_path.to_owned();
            self.Update_path_label();
            if let Err(error) = self.Load_directory().await {
                Log::Error!("Failed to load parent directory: {error:?}");
            }
        }
    }

    async fn Handle_home_click(&mut self) {
        self.Current_path = Path_owned_type::Root();
        self.Update_path_label();
        if let Err(error) = self.Load_directory().await {
            Log::Error!("Failed to load home directory: {error:?}");
        }
    }

    async fn Handle_refresh_click(&mut self) {
        if let Err(error) = self.Load_directory().await {
            Log::Error!("Failed to refresh directory: {error:?}");
        }
    }

    async fn Handle_go_click(&mut self) {
        unsafe {
            if !self.Path_text_area.is_null() {
                // Get the text from the text area
                let Text_ptr = LVGL::lv_textarea_get_text(self.Path_text_area);
                if !Text_ptr.is_null() {
                    // Convert C string to Rust string
                    let Text_cstr = core::ffi::CStr::from_ptr(Text_ptr);
                    if let Ok(Path_str) = Text_cstr.to_str() {
                        // Try to create a path from the entered string
                        let New_path =
                            Path_owned_type::New(Path_str.to_string()).unwrap_or_else(|| {
                                Log::Error!("Invalid path entered: {Path_str}");
                                self.Current_path.clone()
                            });

                        // Navigate to the new path
                        self.Current_path = New_path;
                        self.Update_path_label();

                        // Try to load the directory
                        if let Err(error) = self.Load_directory().await {
                            Log::Error!("Failed to navigate to path '{Path_str}': {error:?}");
                            // If navigation fails, revert to previous path
                            // For now, just stay on the current path
                        }
                    }
                }
            }
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
