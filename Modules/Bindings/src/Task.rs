use Binding_tool::Bind_function_native;
use Task::{Error_type, Result_type, Task_identifier_type, Task_type};
use Virtual_machine::{Function_descriptors, Registrable_trait, Runtime_type};

pub struct Task_bindings {}

impl Registrable_trait for Task_bindings {
    fn Get_functions(&self) -> &[Virtual_machine::Function_descriptor_type] {
        &Task_bindings_functions
    }
}

impl Default for Task_bindings {
    fn default() -> Self {
        Self::New()
    }
}

impl Task_bindings {
    pub fn New() -> Self {
        Self {}
    }
}

fn Get_task_manager() -> &'static Task::Manager_type {
    Task::Get_instance().expect("Task manager not initialized")
}

const Task_bindings_functions: [Virtual_machine::Function_descriptor_type; 5] = Function_descriptors!(
    New_task_binding,
    Get_environment_variable_binding,
    Set_environment_variable_binding,
    Remove_environment_variable_binding,
    Sleep_binding
);

#[Bind_function_native(Prefix = "Task")]
fn Sleep(Duration: u64) {
    Task_type::Sleep(std::time::Duration::from_millis(Duration));
}

#[Bind_function_native(Prefix = "Task")]
fn New_task(Name: &str, Stack_size: u32, Function: u32) -> Result_type<()> {
    let New_environment = Environment
        .Create_environment(Stack_size as usize)
        .map_err(|_| Error_type::Failed_to_spawn_thread)
        .unwrap();

    Get_task_manager().New_task(None, None, Name, Some(Stack_size as usize), move || {
        Runtime_type::Initialize_thread_environment().unwrap();

        let _ = New_environment.Call_indirect_function(Function, &vec![]);

        Runtime_type::Deinitialize_thread_environment();
    })?;

    Ok(())
}

#[Bind_function_native(Prefix = "Task")]
fn Get_environment_variable(
    Task_identifier: Task_identifier_type,
    Name: &str,
    Value: &mut [u8],
) -> Result_type<()> {
    Value.copy_from_slice(
        Get_task_manager()
            .Get_environment_variable(Task_identifier, Name)?
            .as_bytes(),
    );

    Ok(())
}

#[Bind_function_native(Prefix = "Task")]
fn Set_environment_variable(
    Task_identifier: Task_identifier_type,
    Name: &str,
    Value: &str,
) -> Result_type<()> {
    Get_task_manager().Set_environment_variable(Task_identifier, Name, Value)?;

    Ok(())
}

#[Bind_function_native(Prefix = "Task")]
fn Remove_environment_variable(
    Task_identifier: Task_identifier_type,
    Name: &str,
) -> Result_type<()> {
    Get_task_manager().Remove_environment_variable(Task_identifier, Name)?;

    Ok(())
}
