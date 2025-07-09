use alloc::{
    borrow::ToOwned,
    ffi::CString,
    string::{String, ToString},
    vec::Vec,
};
use core::{ptr::null_mut, time::Duration};

use file_system::{Path_owned_type, Path_type, Type_type};
use graphics::{
    lvgl,
    palette::{self, Hue_type},
    Event_code_type, Window_type,
};
use virtual_file_system::{get_instance, Directory_type};

use crate::error::{Error_type, Result_type};

pub struct File_manager_type {
    window: Window_type,
    toolbar: *mut lvgl::lv_obj_t,
    up_button: *mut lvgl::lv_obj_t,
    home_button: *mut lvgl::lv_obj_t,
    refresh_button: *mut lvgl::lv_obj_t,
    path_text_area: *mut lvgl::lv_obj_t,
    go_button: *mut lvgl::lv_obj_t,
    file_list: *mut lvgl::lv_obj_t,
    current_path: Path_owned_type,
    files: Vec<File_item_type>,
    running: bool,
}

#[derive(Clone)]
pub struct File_item_type {
    pub name: String,
    pub r#type: Type_type,
    pub size: u64,
}

impl File_manager_type {
    pub async fn new() -> Result_type<Self> {
        let _lock = graphics::get_instance().lock().await;

        let mut window = graphics::get_instance().create_window().await?;

        window.set_icon("Fm", palette::get(Hue_type::Cyan, palette::Tone_type::MAIN));

        let mut manager = Self {
            window,
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
            lvgl::lv_obj_set_layout(
                manager.window.get_object(),
                lvgl::lv_layout_t_LV_LAYOUT_FLEX,
            );
            lvgl::lv_obj_set_flex_flow(
                manager.window.get_object(),
                lvgl::lv_flex_flow_t_LV_FLEX_FLOW_COLUMN,
            );
            lvgl::lv_obj_set_style_pad_all(manager.window.get_object(), 0, lvgl::LV_STATE_DEFAULT);
        }

        manager.create_toolbar().await?;
        manager.create_file_list().await?;
        manager.load_directory().await?;

        Ok(manager)
    }

    pub async fn run(&mut self) {
        while self.running {
            let event = match self.window.pop_event() {
                Some(event) => event,
                None => {
                    task::Manager_type::Sleep(Duration::from_millis(50)).await;
                    continue;
                }
            };

            match event.get_code() {
                Event_code_type::Delete => {
                    if event.get_target() == self.window.get_object() {
                        self.running = false;
                    }
                }
                Event_code_type::Clicked => {
                    let target = event.get_target();

                    // Handle different button clicks
                    if target == self.up_button {
                        self.handle_up_click().await;
                    } else if target == self.home_button {
                        self.handle_home_click().await;
                    } else if target == self.refresh_button {
                        self.handle_refresh_click().await;
                    } else if target == self.go_button {
                        self.handle_go_click().await;
                    } else {
                        // Handle file item clicks
                        self.handle_file_click(target).await;
                    }
                }
                _ => {}
            }
        }
    }

    async fn create_toolbar(&mut self) -> Result_type<()> {
        unsafe {
            self.toolbar = lvgl::lv_obj_create(self.window.get_object());
            if self.toolbar.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            // Toolbar properties - fixed height at top
            lvgl::lv_obj_set_size(self.toolbar, lvgl::lv_pct(100), lvgl::LV_SIZE_CONTENT);
            lvgl::lv_obj_align(self.toolbar, lvgl::lv_align_t_LV_ALIGN_TOP_MID, 0, 0);

            lvgl::lv_obj_set_style_border_width(self.toolbar, 0, lvgl::LV_STATE_DEFAULT);
            lvgl::lv_obj_set_style_pad_all(self.toolbar, 10, lvgl::LV_STATE_DEFAULT);
            lvgl::lv_obj_set_layout(self.toolbar, lvgl::lv_layout_t_LV_LAYOUT_FLEX);
            lvgl::lv_obj_set_flex_flow(self.toolbar, lvgl::lv_flex_flow_t_LV_FLEX_FLOW_ROW);
            lvgl::lv_obj_set_flex_align(
                self.toolbar,
                lvgl::lv_flex_align_t_LV_FLEX_ALIGN_START,
                lvgl::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                lvgl::lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
            );

            // Up button
            self.up_button = lvgl::lv_button_create(self.toolbar);
            if self.up_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }
            let up_label = lvgl::lv_label_create(self.up_button);
            lvgl::lv_label_set_text(up_label, lvgl::LV_SYMBOL_UP as *const _ as *const i8);
            lvgl::lv_obj_center(up_label);

            // Remove event handler - events bubble up to window

            // Home button
            self.home_button = lvgl::lv_button_create(self.toolbar);
            if self.home_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            let home_label = lvgl::lv_label_create(self.home_button);
            lvgl::lv_label_set_text(home_label, lvgl::LV_SYMBOL_HOME as *const _ as *const i8);
            lvgl::lv_obj_center(home_label);

            // Remove event handler - events bubble up to window

            // Refresh button
            self.refresh_button = lvgl::lv_button_create(self.toolbar);
            if self.refresh_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            let refresh_label = lvgl::lv_label_create(self.refresh_button);

            lvgl::lv_label_set_text(
                refresh_label,
                lvgl::LV_SYMBOL_REFRESH as *const _ as *const i8,
            );
            lvgl::lv_obj_center(refresh_label);

            // Remove event handler - events bubble up to window

            // Path text area - use flex grow to take remaining space
            self.path_text_area = lvgl::lv_textarea_create(self.toolbar);
            if self.path_text_area.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            lvgl::lv_obj_set_flex_grow(self.path_text_area, 1); // Take remaining space
            lvgl::lv_obj_set_style_pad_left(self.path_text_area, 10, lvgl::LV_STATE_DEFAULT);

            // Configure text area properties
            lvgl::lv_textarea_set_one_line(self.path_text_area, true);
            lvgl::lv_textarea_set_cursor_click_pos(self.path_text_area, true);

            // Go button
            self.go_button = lvgl::lv_button_create(self.toolbar);
            if self.go_button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            let go_label = lvgl::lv_label_create(self.go_button);
            lvgl::lv_label_set_text(go_label, lvgl::LV_SYMBOL_RIGHT as *const _ as *const i8);
            lvgl::lv_obj_center(go_label);

            self.update_path_label();
        }

        Ok(())
    }

    async fn create_file_list(&mut self) -> Result_type<()> {
        unsafe {
            self.file_list = lvgl::lv_list_create(self.window.get_object());
            if self.file_list.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            // File list properties - use flex grow to fill remaining space
            lvgl::lv_obj_set_width(self.file_list, lvgl::lv_pct(100));
            lvgl::lv_obj_set_flex_grow(self.file_list, 1); // Take remaining vertical space

            // Ensure proper scrolling behavior
            lvgl::lv_obj_set_style_pad_all(self.file_list, 0, lvgl::LV_STATE_DEFAULT);
        }

        Ok(())
    }

    async fn load_directory(&mut self) -> Result_type<()> {
        // Clear existing files
        self.clear_file_list();

        // Open directory
        let virtual_file_system = get_instance();
        let directory = Directory_type::open(virtual_file_system, &self.current_path).await;

        match directory {
            Ok(directory) => {
                // Read directory entries
                while let Ok(Some(entry)) = directory.read().await {
                    let name = entry.get_name();

                    // Skip "." and ".." entries
                    if name == "." || name == ".." {
                        continue;
                    }

                    let file_item = File_item_type {
                        name: name.clone(),
                        r#type: entry.get_type(),
                        size: entry.get_size().As_u64(),
                    };

                    self.files.push(file_item);
                }

                // Sort files: directories first, then files
                self.files.sort_by(|a, b| match (a.r#type, b.r#type) {
                    (Type_type::Directory, Type_type::Directory) => a.name.cmp(&b.name),
                    (Type_type::Directory, _) => core::cmp::Ordering::Less,
                    (_, Type_type::Directory) => core::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                });

                // Create UI for each file
                for i in 0..self.files.len() {
                    self.create_file_item(i).await?;
                }

                Ok(())
            }
            Err(error) => {
                // Show error message
                self.show_error_message("Failed to open directory").await;
                Err(Error_type::Failed_to_read_directory(error))
            }
        }
    }

    async fn create_file_item(&mut self, index: usize) -> Result_type<()> {
        unsafe {
            let file = &self.files[index];

            let icon_symbol = match file.r#type {
                Type_type::Directory => lvgl::LV_SYMBOL_DIRECTORY,
                _ => lvgl::LV_SYMBOL_FILE,
            };

            let name_cstring = CString::new(file.name.clone()).unwrap();
            let button = lvgl::lv_list_add_button(
                self.file_list,
                icon_symbol.as_ptr() as *const core::ffi::c_void,
                name_cstring.as_ptr(),
            );

            if button.is_null() {
                return Err(Error_type::Failed_to_create_object);
            }

            // Store the file index in the button's user data for identification
            lvgl::lv_obj_set_user_data(button, index as *mut core::ffi::c_void);
        }

        Ok(())
    }

    fn clear_file_list(&mut self) {
        unsafe {
            // Clear the list content - lvgl:: list automatically manages its children
            lvgl::lv_obj_clean(self.file_list);
        }
        self.files.clear();
    }

    fn update_path_label(&self) {
        unsafe {
            if !self.path_text_area.is_null() {
                let path_text = CString::new(self.current_path.to_string()).unwrap();
                lvgl::lv_textarea_set_text(self.path_text_area, path_text.as_ptr());
            }
        }
    }

    async fn show_error_message(&self, _message: &str) {
        // TODO: Implement error message display
        // For now, just print to console if logging is available
    }

    async fn handle_file_click(&mut self, target: *mut lvgl::lv_obj_t) {
        unsafe {
            // Check if the clicked object is a file list button
            let parent = lvgl::lv_obj_get_parent(target as *const lvgl::lv_obj_t);
            if core::ptr::eq(parent as *mut lvgl::lv_obj_t, self.file_list) {
                // Get the file index from user data
                let index = lvgl::lv_obj_get_user_data(target) as usize;

                if index < self.files.len() {
                    let file = &self.files[index];

                    if file.r#type == Type_type::Directory {
                        // Navigate to directory
                        if let Some(new_path) = self
                            .current_path
                            .clone()
                            .Join(Path_type::From_str(&file.name))
                        {
                            self.current_path = new_path;
                            self.update_path_label();
                            // Reload directory contents
                            if let Err(error) = self.load_directory().await {
                                log::Error!("Failed to load directory: {error:?}");
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

    async fn handle_up_click(&mut self) {
        if let Some(parent_path) = self.current_path.Go_parent() {
            self.current_path = parent_path.to_owned();
            self.update_path_label();
            if let Err(error) = self.load_directory().await {
                log::Error!("Failed to load parent directory: {error:?}");
            }
        }
    }

    async fn handle_home_click(&mut self) {
        self.current_path = Path_owned_type::Root();
        self.update_path_label();
        if let Err(error) = self.load_directory().await {
            log::Error!("Failed to load home directory: {error:?}");
        }
    }

    async fn handle_refresh_click(&mut self) {
        if let Err(error) = self.load_directory().await {
            log::Error!("Failed to refresh directory: {error:?}");
        }
    }

    async fn handle_go_click(&mut self) {
        unsafe {
            if !self.path_text_area.is_null() {
                // Get the text from the text area
                let text_ptr: *const i8 = lvgl::lv_textarea_get_text(self.path_text_area);
                if !text_ptr.is_null() {
                    // Convert C string to Rust string
                    let text_cstr = core::ffi::CStr::from_ptr(text_ptr);
                    if let Ok(path_str) = text_cstr.to_str() {
                        // Try to create a path from the entered string
                        let new_path =
                            Path_owned_type::New(path_str.to_string()).unwrap_or_else(|| {
                                log::Error!("Invalid path entered: {path_str}");
                                self.current_path.clone()
                            });

                        // Navigate to the new path
                        self.current_path = new_path;
                        self.update_path_label();

                        // Try to load the directory
                        if let Err(error) = self.load_directory().await {
                            log::Error!("Failed to navigate to path '{path_str}': {error:?}");
                            // If navigation fails, revert to previous path
                            // For now, just stay on the current path
                        }
                    }
                }
            }
        }
    }

    pub async fn refresh(&mut self) -> Result_type<()> {
        self.load_directory().await
    }

    pub async fn navigate_to(&mut self, path: &Path_type) -> Result_type<()> {
        self.current_path = path.to_owned();
        self.update_path_label();
        self.load_directory().await
    }

    pub fn get_current_path(&self) -> &Path_type {
        &self.current_path
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}
