use alloc::{ffi::CString, format, sync::Arc};
use core::fmt::Debug;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Environment_variable_type(Arc<CString>, usize);

impl Debug for Environment_variable_type {
    fn fmt(&self, Formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
        let Environment_variable = CString::new(format!("{Name}={Value}")).unwrap();
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

#[cfg(test)]
mod Tests {
    use super::*;

    #[test]
    fn Test_environment_variable_creation() {
        let env_var = Environment_variable_type::New("PATH", "/usr/bin:/bin");

        assert_eq!(env_var.Get_name(), "PATH");
        assert_eq!(env_var.Get_value(), "/usr/bin:/bin");
    }

    #[test]
    fn Test_environment_variable_empty_name() {
        let env_var = Environment_variable_type::New("", "some_value");

        assert_eq!(env_var.Get_name(), "");
        assert_eq!(env_var.Get_value(), "some_value");
    }

    #[test]
    fn Test_environment_variable_empty_value() {
        let env_var = Environment_variable_type::New("EMPTY_VAR", "");

        assert_eq!(env_var.Get_name(), "EMPTY_VAR");
        assert_eq!(env_var.Get_value(), "");
    }

    #[test]
    fn Test_environment_variable_both_empty() {
        let env_var = Environment_variable_type::New("", "");

        assert_eq!(env_var.Get_name(), "");
        assert_eq!(env_var.Get_value(), "");
    }

    #[test]
    fn Test_environment_variable_special_characters() {
        let name = "TEST_VAR";
        let value = "value with spaces and !@#$%^&*()";
        let env_var = Environment_variable_type::New(name, value);

        assert_eq!(env_var.Get_name(), name);
        assert_eq!(env_var.Get_value(), value);
    }

    #[test]
    fn Test_environment_variable_equals_in_value() {
        let name = "CONFIG";
        let value = "key=value=another=part";
        let env_var = Environment_variable_type::New(name, value);

        assert_eq!(env_var.Get_name(), name);
        assert_eq!(env_var.Get_value(), value);
    }

    #[test]
    fn Test_environment_variable_unicode() {
        let name = "UNICODE_VAR";
        let value = "🦀 Rust is awesome! 中文测试";
        let env_var = Environment_variable_type::New(name, value);

        assert_eq!(env_var.Get_name(), name);
        assert_eq!(env_var.Get_value(), value);
    }

    #[test]
    fn Test_environment_variable_clone() {
        let env_var1 = Environment_variable_type::New("HOME", "/home/user");
        let env_var2 = env_var1.clone();

        assert_eq!(env_var1.Get_name(), env_var2.Get_name());
        assert_eq!(env_var1.Get_value(), env_var2.Get_value());
        assert_eq!(env_var1, env_var2);
    }

    #[test]
    fn Test_environment_variable_equality() {
        let env_var1 = Environment_variable_type::New("USER", "alice");
        let env_var2 = Environment_variable_type::New("USER", "alice");
        let env_var3 = Environment_variable_type::New("USER", "bob");
        let env_var4 = Environment_variable_type::New("HOME", "alice");

        assert_eq!(env_var1, env_var2);
        assert_ne!(env_var1, env_var3);
        assert_ne!(env_var1, env_var4);
    }

    #[test]
    fn Test_environment_variable_hash() {
        use std::collections::HashMap;

        let env_var1 = Environment_variable_type::New("PATH", "/usr/bin");
        let env_var2 = Environment_variable_type::New("HOME", "/home/user");
        let env_var3 = Environment_variable_type::New("PATH", "/usr/bin"); // Same as env_var1

        let mut map = HashMap::new();
        map.insert(env_var1.clone(), "first");
        map.insert(env_var2.clone(), "second");

        // Should not create a new entry since it's the same as env_var1
        map.insert(env_var3.clone(), "third");

        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&env_var1), Some(&"third")); // Should be overwritten
        assert_eq!(map.get(&env_var2), Some(&"second"));
        assert_eq!(map.get(&env_var3), Some(&"third"));
    }

    #[test]
    fn Test_environment_variable_debug_format() {
        let env_var = Environment_variable_type::New("DEBUG_VAR", "debug_value");
        let debug_string = format!("{env_var:?}");

        assert!(debug_string.contains("Environment_variable_type"));
        assert!(debug_string.contains("DEBUG_VAR"));
        assert!(debug_string.contains("debug_value"));
    }

    #[test]
    fn Test_environment_variable_get_raw() {
        let name = "RAW_TEST";
        let value = "raw_value";
        let env_var = Environment_variable_type::New(name, value);
        let raw_cstring = env_var.Get_raw();

        assert_eq!(raw_cstring.to_str().unwrap(), "RAW_TEST=raw_value");
    }

    #[test]
    fn Test_environment_variable_long_name_and_value() {
        let long_name = "A".repeat(1000);
        let long_value = "B".repeat(2000);
        let env_var = Environment_variable_type::New(&long_name, &long_value);

        assert_eq!(env_var.Get_name(), long_name);
        assert_eq!(env_var.Get_value(), long_value);
    }

    #[test]
    fn Test_environment_variable_single_character() {
        let env_var = Environment_variable_type::New("A", "B");

        assert_eq!(env_var.Get_name(), "A");
        assert_eq!(env_var.Get_value(), "B");
    }

    #[test]
    fn Test_environment_variable_newlines_and_tabs() {
        let name = "MULTILINE";
        let value = "line1\nline2\tvalue";
        let env_var = Environment_variable_type::New(name, value);

        assert_eq!(env_var.Get_name(), name);
        assert_eq!(env_var.Get_value(), value);
    }

    #[test]
    fn Test_environment_variable_arc_sharing() {
        let env_var1 = Environment_variable_type::New("SHARED", "value");
        let env_var2 = env_var1.clone();

        // Both should point to the same Arc
        assert_eq!(Arc::as_ptr(&env_var1.0), Arc::as_ptr(&env_var2.0));

        // Values should be identical
        assert_eq!(env_var1.Get_name(), env_var2.Get_name());
        assert_eq!(env_var1.Get_value(), env_var2.Get_value());
    }
}
