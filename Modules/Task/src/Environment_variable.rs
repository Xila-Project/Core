use std::{ffi::CString, fmt::Debug, sync::Arc};

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Environment_variable_type(Arc<CString>, usize);

impl Debug for Environment_variable_type {
    fn fmt(&self, Formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Formatter
            .debug_struct("Environment_variable_type")
            .field("Name", &self.Get_name())
            .field("Value", &self.Get_value())
            .finish()
    }
}

impl Environment_variable_type {
    /// Create a new environment variable.
    pub fn New(Name: &str, Value: &str) -> Self {
        let Environment_variable = CString::new(format!("{}={}", Name, Value)).unwrap();
        Self(Arc::new(Environment_variable), Name.len())
    }

    /// Create a new environment variable from a raw string.
    ///
    /// # Example
    ///
    /// ```
    /// use Task::Environment_variable_type;
    ///
    /// let Environment_variable = Environment_variable_type::New("Name", "Value");
    ///
    /// assert_eq!(Environment_variable.Get_name(), "Name");
    /// ```
    pub fn Get_name(&self) -> &str {
        self.0.to_str().unwrap()[..self.1].trim_end_matches('\0')
    }

    /// Get the value of the environment variable.
    ///
    /// # Example
    ///
    /// ```
    /// use Task::Environment_variable_type;
    ///
    /// let Environment_variable = Environment_variable_type::New("Name", "Value");
    ///
    /// assert_eq!(Environment_variable.Get_value(), "Value");
    /// ```
    pub fn Get_value(&self) -> &str {
        self.0.to_str().unwrap()[self.1 + 1..].trim_end_matches('\0')
    }

    /// Get the inner raw CString.
    pub fn Get_raw(&self) -> &CString {
        &self.0
    }
}
