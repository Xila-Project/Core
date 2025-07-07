#![allow(non_camel_case_types)]

use core::{fmt, num::NonZeroU32};

pub type Result_type<T> = core::result::Result<T, Error_type>;

#[derive(Debug, Clone)]
#[repr(C)]
pub enum Error_type {
    Invalid_task_identifier = 1,
    Invalid_spawner_identifier,
    Thread_not_registered,
    Thread_already_registered,
    Failed_to_create_thread,
    No_thread_for_task,
    Failed_to_spawn_thread,
    Invalid_environment_variable,
    Too_many_tasks,
    Too_many_spawners,
    Already_initialized,
    Already_set,
    Not_initialized,
    No_spawner_available,
}

impl fmt::Display for Error_type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<Error_type> for NonZeroU32 {
    fn from(error: Error_type) -> Self {
        unsafe { NonZeroU32::new_unchecked(error as u32) }
    }
}

#[cfg(test)]
mod Tests {
    use super::*;
    use std::format;
    use std::string::{String, ToString};
    use std::vec;

    #[test]
    fn Test_error_type_display() {
        let error = Error_type::Invalid_task_identifier;
        let display_string = format!("{error}");
        assert_eq!(display_string, "Invalid_task_identifier");

        let error = Error_type::Too_many_tasks;
        let display_string = format!("{error}");
        assert_eq!(display_string, "Too_many_tasks");
    }

    #[test]
    fn Test_error_type_debug() {
        let error = Error_type::Invalid_spawner_identifier;
        let debug_string = format!("{error:?}");
        assert_eq!(debug_string, "Invalid_spawner_identifier");

        let error = Error_type::Thread_not_registered;
        let debug_string = format!("{error:?}");
        assert_eq!(debug_string, "Thread_not_registered");
    }

    #[test]
    fn Test_error_type_clone() {
        let error1 = Error_type::Failed_to_create_thread;
        let error2 = error1.clone();

        assert_eq!(format!("{error1:?}"), format!("{:?}", error2));
    }

    #[test]
    fn Test_all_error_variants() {
        let errors = vec![
            Error_type::Invalid_task_identifier,
            Error_type::Invalid_spawner_identifier,
            Error_type::Thread_not_registered,
            Error_type::Thread_already_registered,
            Error_type::Failed_to_create_thread,
            Error_type::No_thread_for_task,
            Error_type::Failed_to_spawn_thread,
            Error_type::Invalid_environment_variable,
            Error_type::Too_many_tasks,
            Error_type::Too_many_spawners,
            Error_type::Already_initialized,
            Error_type::Already_set,
            Error_type::Not_initialized,
            Error_type::No_spawner_available,
        ];

        // Test that all variants can be created and formatted
        for error in errors {
            let debug_str = format!("{error:?}");
            let display_str = format!("{error}");

            assert!(!debug_str.is_empty());
            assert!(!display_str.is_empty());
            assert_eq!(debug_str, display_str);
        }
    }

    #[test]
    fn Test_error_to_nonzero_u32_conversion() {
        let errors_and_expected_values = vec![
            (Error_type::Invalid_task_identifier, 1u32),
            (Error_type::Invalid_spawner_identifier, 2u32),
            (Error_type::Thread_not_registered, 3u32),
            (Error_type::Thread_already_registered, 4u32),
            (Error_type::Failed_to_create_thread, 5u32),
            (Error_type::No_thread_for_task, 6u32),
            (Error_type::Failed_to_spawn_thread, 7u32),
            (Error_type::Invalid_environment_variable, 8u32),
            (Error_type::Too_many_tasks, 9u32),
            (Error_type::Too_many_spawners, 10u32),
            (Error_type::Already_initialized, 11u32),
            (Error_type::Already_set, 12u32),
            (Error_type::Not_initialized, 13u32),
            (Error_type::No_spawner_available, 14u32),
        ];

        for (error, expected_value) in errors_and_expected_values {
            let non_zero: NonZeroU32 = error.into();
            assert_eq!(non_zero.get(), expected_value);
        }
    }

    #[test]
    fn Test_result_type_ok() {
        let result: Result_type<i32> = Ok(42);
        assert!(result.is_ok());
        if let Ok(value) = result {
            assert_eq!(value, 42);
        }
    }

    #[test]
    fn Test_result_type_err() {
        let result: Result_type<i32> = Err(Error_type::Invalid_task_identifier);
        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(format!("{error:?}"), "Invalid_task_identifier");
        }
    }

    #[test]
    fn Test_error_in_result_chain() {
        fn might_fail(should_fail: bool) -> Result_type<String> {
            if should_fail {
                Err(Error_type::Too_many_tasks)
            } else {
                Ok("Success".to_string())
            }
        }

        let success_result = might_fail(false);
        assert!(success_result.is_ok());
        if let Ok(value) = success_result {
            assert_eq!(value, "Success");
        }

        let failure_result = might_fail(true);
        assert!(failure_result.is_err());

        if let Err(error) = failure_result {
            assert_eq!(format!("{error:?}"), "Too_many_tasks");
        }
    }

    #[test]
    fn Test_error_propagation() {
        fn inner_function() -> Result_type<i32> {
            Err(Error_type::Not_initialized)
        }

        fn outer_function() -> Result_type<String> {
            let _value = inner_function()?;
            Ok("This won't be reached".to_string())
        }

        let result = outer_function();
        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(format!("{error:?}"), "Not_initialized");
        }
    }

    #[test]
    fn Test_error_size() {
        // Ensure the error type has a reasonable size
        assert!(std::mem::size_of::<Error_type>() <= 8);
    }

    #[test]
    fn Test_error_repr_c() {
        // Test that the error can be used in FFI contexts
        // This mainly ensures the #[repr(C)] attribute works as expected
        let error = Error_type::Invalid_task_identifier;
        let error_discriminant = unsafe { std::mem::transmute::<Error_type, u32>(error) };
        assert_eq!(error_discriminant, 1);

        let error2 = Error_type::Invalid_spawner_identifier;
        let error2_discriminant = unsafe { std::mem::transmute::<Error_type, u32>(error2) };
        assert_eq!(error2_discriminant, 2);
    }

    #[test]
    fn Test_specific_error_scenarios() {
        // Test environment variable error specifically
        let env_error = Error_type::Invalid_environment_variable;
        let display_str = format!("{env_error}");
        let non_zero: NonZeroU32 = env_error.into();
        assert_eq!(non_zero.get(), 8u32);
        assert_eq!(display_str, "Invalid_environment_variable");

        // Test task-related errors
        let task_error = Error_type::Invalid_task_identifier;
        assert_eq!(format!("{task_error}"), "Invalid_task_identifier");

        let spawner_error = Error_type::Invalid_spawner_identifier;
        assert_eq!(format!("{spawner_error}"), "Invalid_spawner_identifier");
    }
}
