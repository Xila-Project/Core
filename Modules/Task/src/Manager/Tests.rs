// Tests module - contains all Manager tests

use super::*;
use crate::Test;
use core::time::Duration;
use std::collections::BTreeMap;
use Users::{Group_identifier_type, User_identifier_type};

#[Test(crate)]
async fn Test_get_task_name() {
    let Manager = Initialize();

    let Task_name = "Test Task";
    let Task = Manager.Get_current_task_identifier().await;

    let Spawner = Manager.Get_spawner(Task).await.unwrap();

    let _ = Manager
        .Spawn(Task, Task_name, Some(Spawner), async move |Task| {
            assert_eq!(Get_instance().Get_name(Task).await.unwrap(), Task_name);
        })
        .await
        .unwrap()
        .0
        .Join()
        .await;
}

#[Test(crate)]
async fn Test_set_get_owner() {
    let Manager = Initialize();

    let Task = Manager.Get_current_task_identifier().await;

    // Set user and group to root
    Manager
        .Set_user(Task, User_identifier_type::Root)
        .await
        .unwrap();
    Manager
        .Set_group(Task, Group_identifier_type::Root)
        .await
        .unwrap();

    assert_eq!(
        Get_instance().Get_user(Task).await.unwrap(),
        User_identifier_type::Root
    );
    assert_eq!(
        Get_instance().Get_group(Task).await.unwrap(),
        Group_identifier_type::Root
    );
}

#[Test(crate)]
async fn Test_get_current_task_identifier() {
    let Manager = Initialize();

    let Task = Manager.Get_current_task_identifier().await;

    let Spawner = Manager.Get_spawner(Task).await.unwrap();

    Manager
        .Spawn(Task, "Current Task", Some(Spawner), async move |Task| {
            assert_eq!(Get_instance().Get_current_task_identifier().await, Task);
        })
        .await
        .unwrap()
        .0
        .Join()
        .await;
}

#[Test(crate)]
async fn Test_task_owner_inheritance() {
    let Manager = Initialize();

    let Task = Manager.Get_current_task_identifier().await;
    let User_identifier = User_identifier_type::New(123);
    let Group_identifier = Group_identifier_type::New(456);

    Manager.Set_user(Task, User_identifier).await.unwrap();
    Manager.Set_group(Task, Group_identifier).await.unwrap();

    // Get the spawner of the current task
    let Spawner = Manager.Get_spawner(Task).await.unwrap();

    // Spawn first task that verifies inheritance
    Manager
        .Spawn(Task, "Task 1", Some(Spawner), async move |Task_1| {
            assert_eq!(
                Get_instance().Get_user(Task_1).await.unwrap(),
                User_identifier
            );
            assert_eq!(
                Get_instance().Get_group(Task_1).await.unwrap(),
                Group_identifier
            );

            // Get the spawner of Task_1 to inherit to Task_2
            let Task_1_spawner = Get_instance().Get_spawner(Task_1).await.unwrap();

            // Spawn second task as a child of the first task
            let _ = Manager
                .Spawn(
                    Task_1,
                    "Task 2",
                    Some(Task_1_spawner),
                    async move |Task_2| {
                        // Verify that the child task inherits the user and group
                        assert_eq!(
                            Get_instance().Get_user(Task_2).await.unwrap(),
                            User_identifier
                        );
                        assert_eq!(
                            Get_instance().Get_group(Task_2).await.unwrap(),
                            Group_identifier
                        );

                        // This task has no nested calls to Spawn
                    },
                )
                .await
                .unwrap()
                .0
                .Join()
                .await;
        })
        .await
        .unwrap()
        .0
        .Join()
        .await;
}

#[Test(crate)]
async fn Test_environment_variables() {
    let Manager = Initialize();

    let Task_identifier = Manager.Get_current_task_identifier().await;
    let Name = "Key";
    let Value = "Value";

    Manager
        .Set_environment_variable(Task_identifier, Name, Value)
        .await
        .unwrap();
    assert_eq!(
        Manager
            .Get_environment_variable(Task_identifier, Name)
            .await
            .unwrap()
            .Get_value(),
        Value
    );
    Manager
        .Remove_environment_variable(Task_identifier, Name)
        .await
        .unwrap();
    assert!(Manager
        .Get_environment_variable(Task_identifier, Name)
        .await
        .is_err());
}

#[Test(crate)]
async fn Test_environment_variable_inheritance() {
    let Manager = Initialize();

    let Task = Manager.Get_current_task_identifier().await;

    Get_instance()
        .Set_environment_variable(Task, "Key", "Value")
        .await
        .unwrap();

    // Get the spawner of the current task
    let Spawner = Manager.Get_spawner(Task).await.unwrap();

    // Then spawn the grandchild task with the returned task ID
    Manager
        .Spawn(Task, "Grand child Task", Some(Spawner), async move |Task| {
            assert_eq!(
                Get_instance()
                    .Get_environment_variable(Task, "Key")
                    .await
                    .unwrap()
                    .Get_value(),
                "Value"
            );
        })
        .await
        .unwrap()
        .0
        .Join()
        .await;
}

#[Test(crate)]
async fn Test_join_handle() {
    let Manager = Initialize();

    let Task = Manager.Get_current_task_identifier().await;

    let Spawner = Manager.Get_spawner(Task).await.unwrap();
    let Join_handle = Manager
        .Spawn(Task, "Task with join handle", Some(Spawner), async |_| 42)
        .await
        .unwrap();
    assert_eq!(Join_handle.0.Join().await, 42);
}

#[Test(crate)]
async fn Test_set_user() {
    let Manager = Initialize();

    let Task = Manager.Get_current_task_identifier().await;

    let User = User_identifier_type::New(123); // Assuming User_identifier_type is i32 for example

    Manager.Set_user(Task, User).await.unwrap();

    assert_eq!(Manager.Get_user(Task).await.unwrap(), User);
}

#[Test(crate)]
async fn Test_set_group() {
    let Manager = Initialize();

    let Task = Manager.Get_current_task_identifier().await;

    let Group = Group_identifier_type::New(456); // Assuming Group_identifier_type is i32 for example

    Manager.Set_group(Task, Group).await.unwrap();

    assert_eq!(Manager.Get_group(Task).await.unwrap(), Group);
}

#[Test(crate)]
async fn Test_signal() {
    let Manager = Initialize();

    let Task = Manager.Get_current_task_identifier().await;

    let Spawner = Manager.Get_spawner(Task).await.unwrap();

    let (Child_handle, Child_identifier) = Manager
        .Spawn(Task, "Task with signal", Some(Spawner), async |Task| {
            Manager_type::Sleep(Duration::from_millis(10)).await; // Allow the parent task to set signals

            assert_eq!(
                Get_instance().Peek_signal(Task).await.unwrap(),
                Some(Signal_type::Hangup)
            );

            assert_eq!(
                Get_instance().Pop_signal(Task).await.unwrap(),
                Some(Signal_type::Hangup)
            );

            assert_eq!(
                Get_instance().Peek_signal(Task).await.unwrap(),
                Some(Signal_type::Kill)
            );

            assert_eq!(
                Get_instance().Pop_signal(Task).await.unwrap(),
                Some(Signal_type::Kill)
            );
        })
        .await
        .unwrap();

    Get_instance()
        .Send_signal(Child_identifier, Signal_type::Kill)
        .await
        .unwrap();

    Get_instance()
        .Send_signal(Child_identifier, Signal_type::Hangup)
        .await
        .unwrap();

    // Wait for the task to finish
    Child_handle.Join().await;
}

#[test]
fn test_find_first_available_identifier_empty_map() {
    let map: BTreeMap<u32, ()> = BTreeMap::new();
    let range = 0u32..10u32;

    let result = Manager_type::Find_first_available_identifier(&map, range);
    assert_eq!(result, Some(0u32));
}

#[test]
fn test_find_first_available_identifier_no_gaps() {
    let mut map: BTreeMap<u32, ()> = BTreeMap::new();
    map.insert(0, ());
    map.insert(1, ());
    map.insert(2, ());

    let range = 0u32..10u32;
    let result = Manager_type::Find_first_available_identifier(&map, range);
    assert_eq!(result, Some(3u32));
}

#[test]
fn test_find_first_available_identifier_with_gap() {
    let mut map: BTreeMap<u32, ()> = BTreeMap::new();
    map.insert(0, ());
    map.insert(2, ());
    map.insert(3, ());

    let range = 0u32..10u32;
    let result = Manager_type::Find_first_available_identifier(&map, range);
    assert_eq!(result, Some(1u32));
}

#[test]
fn test_find_first_available_identifier_range_exhausted() {
    let mut map: BTreeMap<u32, ()> = BTreeMap::new();
    map.insert(0, ());
    map.insert(1, ());

    let range = 0u32..2u32;
    let result = Manager_type::Find_first_available_identifier(&map, range);
    assert_eq!(result, None);
}

#[test]
fn test_find_first_available_identifier_single_element() {
    let mut map: BTreeMap<u32, ()> = BTreeMap::new();
    map.insert(1, ());

    let range = 0u32..5u32;
    let result = Manager_type::Find_first_available_identifier(&map, range);
    assert_eq!(result, Some(0u32));
}

#[test]
fn test_find_first_available_identifier_large_gap() {
    let mut map: BTreeMap<u32, ()> = BTreeMap::new();
    map.insert(0, ());
    map.insert(100, ());

    let range = 0u32..200u32;
    let result = Manager_type::Find_first_available_identifier(&map, range);
    assert_eq!(result, Some(1u32));
}

#[test]
fn test_find_first_available_identifier_step_range() {
    let mut map: BTreeMap<usize, ()> = BTreeMap::new();
    map.insert(0, ());
    map.insert(2, ());

    let range = (0..10).step_by(2);
    let result = Manager_type::Find_first_available_identifier(&map, range);
    assert_eq!(result, Some(4));
}

#[test]
fn test_find_first_available_identifier_unordered_keys() {
    let mut map: BTreeMap<u32, ()> = BTreeMap::new();
    map.insert(5, ());
    map.insert(1, ());
    map.insert(3, ());

    let range = 0u32..10u32;
    let result = Manager_type::Find_first_available_identifier(&map, range);
    // BTreeMap orders keys, so iteration will be 1, 3, 5
    // Range starts at 0, so first mismatch is at position 0
    assert_eq!(result, Some(0u32));
}

#[test]
fn test_find_first_available_identifier_exact_match_sequence() {
    let mut map: BTreeMap<u32, ()> = BTreeMap::new();
    for i in 5..10 {
        map.insert(i, ());
    }

    let range = 5u32..15u32;
    let result = Manager_type::Find_first_available_identifier(&map, range);
    assert_eq!(result, Some(10u32));
}

#[test]
fn test_find_first_available_identifier_empty_range() {
    let mut map: BTreeMap<u32, ()> = BTreeMap::new();
    map.insert(0, ());

    let range = 5u32..5u32; // Empty range
    let result = Manager_type::Find_first_available_identifier(&map, range);
    assert_eq!(result, None);
}

#[Test(crate)]
async fn Test_spawn() {
    let Manager = Initialize();

    let Task_name = "Child Task";
    let Task = Manager.Get_current_task_identifier().await;

    let Spawner = Manager.Get_spawner(Task).await.unwrap();

    let _ = Manager
        .Spawn(Task, Task_name, Some(Spawner), async |_| {})
        .await
        .unwrap()
        .0
        .Join()
        .await;
}
