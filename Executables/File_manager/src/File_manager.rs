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
    window: Window_type,
    toolbar: *mut LVGL::lv_obj_t,
    up_button: *mut LVGL::lv_obj_t,
    home_button: *mut LVGL::lv_obj_t,
    refresh_button: *mut LVGL::lv_obj_t,
    path_text_area: *mut LVGL::lv_obj_t,
    go_button: *mut LVGL::lv_obj_t,
    file_list: *mut LVGL::lv_obj_t,
    current_path: Path_owned_type,
    files: Vec<File_item_type>,
    running: bool,
}

#[derive(Clone)]
pub struct File_item_type {
    pub name: String,
    pub Type: Type_type,
    pub size: u64,
}

impl File_manager_type {
    pub async fn new() -> Result_type<Self> {
        let _lock = Graphics::Get_instance().Lock().await;

        let mut Window = Graphics::Get_instance().Create_window().await?;

        Window.Set_icon("Fm", Palette::Get(Hue_type::Cyan, Palette::Tone_type::MAIN));

        let mut Manager = Self {
            window: Window,
            toolbar: null_mut(),
            up_button: null_mut(),
            home_button: null_mut(),
            refresh_button: null_mut(),
            path_text_area: null_mut(),
            go_button: null_mut(),
            file_list: null_mut(),
            current_path: Path_owned_type::Root(),
            files: Vec::new(),
            running: true,
        };

        // Set up window layout for flex
        unsafe {
            LVGL::lv_obj_set_layout(
                Manager.window.Get_object(),
                LVGL::lv_layout_t_LV_LAYOUT_FLEX,
            );
            LVGL::lv_obj_set_flex_flow(
                Manager.window.Get_object(),
                LVGL::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN,
            );
            LVGL::lv_obj_set_style_pad_all(Manager.window.Get_object(), 0, LVGL::LV_STATE_DEFAULT);
        }

        Manager.Create_toolbar().await?;
        Manager.Create_file_list().await?;
        Manager.Load_directory().await?;

        Ok(Manager)
    }

    pub async fn Run(&mut self) {
        while self.running {
            let event = match self.window.Pop_event() {
                Some(event) => event,
                None => {
                    Task::Manager_type::Sleep(Duration::from_millis(50)).await;
                    continue;
                }
            };

            match event.Get_code() {
                Event_code_type::Delete => {
                    if event.Get_target() == self.window.Get_object() {
                        self.running = false;
                    }
                }
                Event_code_type::Clicked => {
                    let target = event.Get_target();

                    // Handle different button clicks
                    if target == self.up_button {
                        self.Handle_up_click().await;
                    } else if target == self.home_button {
                        self.Handle_home_click().await;
                    } else if target == self.refresh_button {
                        self.Handle_refresh_click().await;
                    } else if target == self.go_button {
                        self.Handle_go_click().await;
                    } else {
                        // Handle file item clicks
                        self.Handle_file_click(target).await;
                    }
                }
                _ => {}
            }
        }
    }

    async fn Create_toolbar(&mut self) -> Result_type<()> {
        unsafe {
            self.toolbar = LVGL::lv_obj_create(self.window.Get_object());
            if self.toolbar.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            // Toolbar properties - fixed height at top
            LVGL::lv_obj_set_size(self.toolbar, LVGL::lv_pct(100), LVGL::LV_SIZE_CONTENT);
            LVGL::lv_obj_align(self.toolbar, LVGL::lv_align_t_LV_ALIGN_TOP_MID, 0, 0);

            LVGL::lv_obj_set_style_border_width(self.toolbar, 0, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_style_pad_all(self.toolbar, 10, LVGL::LV_STATE_DEFAULT);
            LVGL::lv_obj_set_layout(self.toolbar, LVGL::lv_layout_t_LV_LAYOUT_FLEX);
            LVGL::lv_obj_set_flex_flow(self.toolbar, LVGL::lv_flex_flow_t_LV_FLEX_FLOW_ROW);
            LVGL::lv_obj_set_flex_align(
                self.toolbar,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_START,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                LVGL::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
            );

            // Up button
            self.up_button = LVGL::lv_button_create(self.toolbar);
            if self.up_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }
            let Up_label = LVGL::lv_label_create(self.up_button);
            LVGL::lv_label_set_text(Up_label, LVGL::LV_SYMBOL_UP as *const _ as *const i8);
            LVGL::lv_obj_center(Up_label);

            // Remove event handler - events bubble up to window

            // Home button
            self.home_button = LVGL::lv_button_create(self.toolbar);
            if self.home_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            let Home_label = LVGL::lv_label_create(self.home_button);
            LVGL::lv_label_set_text(Home_label, LVGL::LV_SYMBOL_HOME as *const _ as *const i8);
            LVGL::lv_obj_center(Home_label);

            // Remove event handler - events bubble up to window

            // Refresh button
            self.refresh_button = LVGL::lv_button_create(self.toolbar);
            if self.refresh_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            let Refresh_label = LVGL::lv_label_create(self.refresh_button);

            LVGL::lv_label_set_text(
                Refresh_label,
                LVGL::LV_SYMBOL_REFRESH as *const _ as *const i8,
            );
            LVGL::lv_obj_center(Refresh_label);

            // Remove event handler - events bubble up to window

            // Path text area - use flex grow to take remaining space
            self.path_text_area = LVGL::lv_textarea_create(self.toolbar);
            if self.path_text_area.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            LVGL::lv_obj_set_flex_grow(self.path_text_area, 1); // Take remaining space
            LVGL::lv_obj_set_style_pad_left(self.path_text_area, 10, LVGL::LV_STATE_DEFAULT);

            // Configure text area properties
            LVGL::lv_textarea_set_one_line(self.path_text_area, true);
            LVGL::lv_textarea_set_cursor_click_pos(self.path_text_area, true);

            // Go button
            self.go_button = LVGL::lv_button_create(self.toolbar);
            if self.go_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            let Go_label = LVGL::lv_label_create(self.go_button);
            LVGL::lv_label_set_text(Go_label, LVGL::LV_SYMBOL_RIGHT as *const _ as *const i8);
            LVGL::lv_obj_center(Go_label);

            self.Update_path_label();
        }

        Ok(())
    }

    async fn Create_file_list(&mut self) -> Result_type<()> {
        unsafe {
            self.file_list = LVGL::lv_list_create(self.window.Get_object());
            if self.file_list.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            // File list properties - use flex grow to fill remaining space
            LVGL::lv_obj_set_width(self.file_list, LVGL::lv_pct(100));
            LVGL::lv_obj_set_flex_grow(self.file_list, 1); // Take remaining vertical space

            // Ensure proper scrolling behavior
            LVGL::lv_obj_set_style_pad_all(self.file_list, 0, LVGL::LV_STATE_DEFAULT);
        }

        Ok(())
    }

    async fn Load_directory(&mut self) -> Result_type<()> {
        // Clear existing files
        self.Clear_file_list();

        // Open directory
        let Virtual_file_system = Get_instance();
        let directory = Directory_type::Open(Virtual_file_system, &self.current_path).await;

        match directory {
            Ok(directory) => {
                // Read directory entries
                while let Ok(Some(Entry)) = directory.Read().await {
                    let name = Entry.Get_name();

                    // Skip "." and ".." entries
                    if name == "." || name == ".." {
                        continue;
                    }

                    let File_item = File_item_type {
                        name: name.clone(),
                        Type: Entry.Get_type(),
                        size: Entry.Get_size().As_u64(),
                    };

                    self.files.push(File_item);
                }

                // Sort files: directories first, then files
                self.files.sort_by(|a, b| match (a.Type, b.Type) {
                    (Type_type::Directory, Type_type::Directory) => a.name.cmp(&b.name),
                    (Type_type::Directory, _) => core::cmp::Ordering::Less,
                    (_, Type_type::Directory) => core::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                });

                // Create UI for each file
                for i in 0..self.files.len() {
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
            let file = &self.files[Index];

            let Icon_symbol = match file.Type {
                Type_type::Directory => LVGL::LV_SYMBOL_DIRECTORY,
                _ => LVGL::LV_SYMBOL_FILE,
            };

            let Name_cstring = CString::new(file.name.clone()).unwrap();
            let button = LVGL::lv_list_add_button(
                self.file_list,
                Icon_symbol.as_ptr() as *const core::ffi::c_void,
                Name_cstring.as_ptr(),
            );

            if button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            // Store the file index in the button's user data for identification
            LVGL::lv_obj_set_user_data(button, Index as *mut core::ffi::c_void);
        }

        Ok(())
    }

    fn Clear_file_list(&mut self) {
        unsafe {
            // Clear the list content - LVGL list automatically manages its children
            LVGL::lv_obj_clean(self.file_list);
        }
        self.files.clear();
    }

    fn Update_path_label(&self) {
        unsafe {
            if !self.path_text_area.is_null() {
                let path_text = CString::new(self.current_path.to_string()).unwrap();
                LVGL::lv_textarea_set_text(self.path_text_area, path_text.as_ptr());
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
            if core::ptr::eq(Parent as *mut LVGL::lv_obj_t, self.file_list) {
                // Get the file index from user data
                let Index = LVGL::lv_obj_get_user_data(Target) as usize;

                if Index < self.files.len() {
                    let file = &self.files[Index];

                    if file.Type == Type_type::Directory {
                        // Navigate to directory
                        if let Some(New_path) = self
                            .current_path
                            .clone()
                            .Join(Path_type::From_str(&file.name))
                        {
                            self.current_path = New_path;
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
        if let Some(parent_path) = self.current_path.Go_parent() {
            self.current_path = parent_path.to_owned();
            self.Update_path_label();
            if let Err(error) = self.Load_directory().await {
                Log::Error!("Failed to load parent directory: {error:?}");
            }
        }
    }

    async fn Handle_home_click(&mut self) {
        self.current_path = Path_owned_type::Root();
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
            if !self.path_text_area.is_null() {
                // Get the text from the text area
                let Text_ptr = LVGL::lv_textarea_get_text(self.path_text_area);
                if !Text_ptr.is_null() {
                    // Convert C string to Rust string
                    let Text_cstr = core::ffi::CStr::from_ptr(Text_ptr);
                    if let Ok(path_str) = Text_cstr.to_str() {
                        // Try to create a path from the entered string
                        let New_path =
                            Path_owned_type::New(path_str.to_string()).unwrap_or_else(|| {
                                Log::Error!("Invalid path entered: {path_str}");
                                self.current_path.clone()
                            });

                        // Navigate to the new path
                        self.current_path = New_path;
                        self.Update_path_label();

                        // Try to load the directory
                        if let Err(error) = self.Load_directory().await {
                            Log::Error!("Failed to navigate to path '{path_str}': {error:?}");
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
        self.current_path = Path.to_owned();
        self.Update_path_label();
        self.Load_directory().await
    }

    pub fn Get_current_path(&self) -> &Path_type {
        &self.current_path
    }

    pub fn Is_running(&self) -> bool {
        self.running
    }
}
