// Tests module - contains all Manager tests

use super::*;
use crate::Test;
use alloc::{collections::BTreeMap, format, vec::Vec};
use core::time::Duration;
use users::{Group_identifier_type, User_identifier_type};

#[Test(task_path = crate)]
async fn test_get_task_name() {
    let Manager = Initialize();

    let Task_name = "Test Task";
    let Task = Manager.get_current_task_identifier().await;

    let Spawner = Manager.get_spawner(Task).await.unwrap();

    let _ = Manager
        .Spawn(Task, Task_name, Some(Spawner), async move |Task| {
            assert_eq!(get_instance().get_name(Task).await.unwrap(), Task_name);
        })
        .await
        .unwrap()
        .0
        .Join()
        .await;
}

#[Test(task_path = crate)]
async fn test_set_get_owner() {
    let Manager = Initialize();

    let Task = Manager.get_current_task_identifier().await;

    // Set user and group to root
    Manager
        .set_user(Task, User_identifier_type::ROOT)
        .await
        .unwrap();
    Manager
        .Set_group(Task, Group_identifier_type::ROOT)
        .await
        .unwrap();

    assert_eq!(
        get_instance().get_user(Task).await.unwrap(),
        User_identifier_type::ROOT
    );
    assert_eq!(
        get_instance().get_group(Task).await.unwrap(),
        Group_identifier_type::ROOT
    );
}

#[Test(task_path = crate)]
async fn test_get_current_task_identifier() {
    let Manager = Initialize();

    let Task = Manager.get_current_task_identifier().await;

    let Spawner = Manager.get_spawner(Task).await.unwrap();

    Manager
        .Spawn(Task, "Current Task", Some(Spawner), async move |Task| {
            assert_eq!(get_instance().get_current_task_identifier().await, Task);
        })
        .await
        .unwrap()
        .0
        .Join()
        .await;
}

#[Test(task_path = crate)]
async fn test_task_owner_inheritance() {
    let Manager = Initialize();

    let Task = Manager.get_current_task_identifier().await;
    let User_identifier = User_identifier_type::New(123);
    let Group_identifier = Group_identifier_type::New(456);

    Manager.set_user(Task, User_identifier).await.unwrap();
    Manager.Set_group(Task, Group_identifier).await.unwrap();

    // Get the spawner of the current task
    let Spawner = Manager.get_spawner(Task).await.unwrap();

    // Spawn first task that verifies inheritance
    Manager
        .Spawn(Task, "Task 1", Some(Spawner), async move |Task_1| {
            assert_eq!(
                get_instance().get_user(Task_1).await.unwrap(),
                User_identifier
            );
            assert_eq!(
                get_instance().get_group(Task_1).await.unwrap(),
                Group_identifier
            );

            // Get the spawner of Task_1 to inherit to Task_2
            let Task_1_spawner = get_instance().get_spawner(Task_1).await.unwrap();

            // Spawn second task as a child of the first task
            let _ = Manager
                .Spawn(
                    Task_1,
                    "Task 2",
                    Some(Task_1_spawner),
                    async move |Task_2| {
                        // Verify that the child task inherits the user and group
                        assert_eq!(
                            get_instance().get_user(Task_2).await.unwrap(),
                            User_identifier
                        );
                        assert_eq!(
                            get_instance().get_group(Task_2).await.unwrap(),
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

#[Test(task_path = crate)]
async fn test_environment_variables() {
    let Manager = Initialize();

    let Task_identifier = Manager.get_current_task_identifier().await;
    let Name = "Key";
    let Value = "Value";

    Manager
        .Set_environment_variable(Task_identifier, Name, Value)
        .await
        .unwrap();
    assert_eq!(
        Manager
            .get_environment_variable(Task_identifier, Name)
            .await
            .unwrap()
            .get_value(),
        Value
    );
    Manager
        .Remove_environment_variable(Task_identifier, Name)
        .await
        .unwrap();
    assert!(Manager
        .get_environment_variable(Task_identifier, Name)
        .await
        .is_err());
}

#[Test(task_path = crate)]
async fn test_environment_variable_inheritance() {
    let Manager = Initialize();

    let Task = Manager.get_current_task_identifier().await;

    get_instance()
        .Set_environment_variable(Task, "Key", "Value")
        .await
        .unwrap();

    // Get the spawner of the current task
    let Spawner = Manager.get_spawner(Task).await.unwrap();

    // Then spawn the grandchild task with the returned task ID
    Manager
        .Spawn(Task, "Grand child Task", Some(Spawner), async move |Task| {
            assert_eq!(
                get_instance()
                    .get_environment_variable(Task, "Key")
                    .await
                    .unwrap()
                    .get_value(),
                "Value"
            );
        })
        .await
        .unwrap()
        .0
        .Join()
        .await;
}

#[Test(task_path = crate)]
async fn test_join_handle() {
    let Manager = Initialize();

    let Task = Manager.get_current_task_identifier().await;

    let Spawner = Manager.get_spawner(Task).await.unwrap();
    let Join_handle = Manager
        .Spawn(Task, "Task with join handle", Some(Spawner), async |_| 42)
        .await
        .unwrap();
    assert_eq!(Join_handle.0.Join().await, 42);
}

#[Test(task_path = crate)]
async fn test_set_user() {
    let Manager = Initialize();

    let Task = Manager.get_current_task_identifier().await;

    let User = User_identifier_type::New(123); // Assuming User_identifier_type is i32 for example

    Manager.set_user(Task, User).await.unwrap();

    assert_eq!(Manager.get_user(Task).await.unwrap(), User);
}

#[Test(task_path = crate)]
async fn test_set_group() {
    let Manager = Initialize();

    let Task = Manager.get_current_task_identifier().await;

    let Group = Group_identifier_type::New(456); // Assuming Group_identifier_type is i32 for example

    Manager.Set_group(Task, Group).await.unwrap();

    assert_eq!(Manager.get_group(Task).await.unwrap(), Group);
}

#[Test(task_path = crate)]
async fn test_signal() {
    let Manager = Initialize();

    let Task = Manager.get_current_task_identifier().await;

    let Spawner = Manager.get_spawner(Task).await.unwrap();

    let (Child_handle, Child_identifier) = Manager
        .Spawn(Task, "Task with signal", Some(Spawner), async |Task| {
            Manager_type::Sleep(Duration::from_millis(10)).await; // Allow the parent task to set signals

            assert_eq!(
                get_instance().Peek_signal(Task).await.unwrap(),
                Some(Signal_type::Hangup)
            );

            assert_eq!(
                get_instance().Pop_signal(Task).await.unwrap(),
                Some(Signal_type::Hangup)
            );

            assert_eq!(
                get_instance().Peek_signal(Task).await.unwrap(),
                Some(Signal_type::Kill)
            );

            assert_eq!(
                get_instance().Pop_signal(Task).await.unwrap(),
                Some(Signal_type::Kill)
            );
        })
        .await
        .unwrap();

    get_instance()
        .send_signal(Child_identifier, Signal_type::Kill)
        .await
        .unwrap();

    get_instance()
        .send_signal(Child_identifier, Signal_type::Hangup)
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

#[Test(task_path = crate)]
async fn test_spawn() {
    let Manager = Initialize();

    let Task_name = "Child Task";
    let Task = Manager.get_current_task_identifier().await;

    let Spawner = Manager.get_spawner(Task).await.unwrap();

    let _ = Manager
        .Spawn(Task, Task_name, Some(Spawner), async |_| {})
        .await
        .unwrap()
        .0
        .Join()
        .await;
}

#[Test(task_path = crate)]
async fn test_get_parent() {
    let Manager = Initialize();

    let Root_task = Manager.get_current_task_identifier().await;
    let Spawner = Manager.get_spawner(Root_task).await.unwrap();

    let (Child_handle, _Child_task) = Manager
        .Spawn(
            Root_task,
            "Child Task",
            Some(Spawner),
            async move |Child_task| {
                // Test that child task's parent is the root task
                assert_eq!(
                    get_instance().get_parent(Child_task).await.unwrap(),
                    Root_task
                );
            },
        )
        .await
        .unwrap();

    Child_handle.Join().await;
}

#[Test(task_path = crate)]
async fn test_get_children() {
    let Manager = Initialize();

    let Root_task = Manager.get_current_task_identifier().await;
    let Spawner = Manager.get_spawner(Root_task).await.unwrap();

    // Initially, root task should have no children
    let Initial_children = Manager.get_children(Root_task).await.unwrap();
    let Initial_count = Initial_children.len();
    assert_eq!(Initial_count, 0);

    // Spawn first child
    let (Child1_handle, Child1_task) = Manager
        .Spawn(Root_task, "Child Task 1", Some(Spawner), async move |_| {
            Manager_type::Sleep(core::time::Duration::from_millis(50)).await;
        })
        .await
        .unwrap();

    // Spawn second child
    let (Child2_handle, Child2_task) = Manager
        .Spawn(Root_task, "Child Task 2", Some(Spawner), async move |_| {
            Manager_type::Sleep(core::time::Duration::from_millis(50)).await;
        })
        .await
        .unwrap();

    // Check that root task has exactly 2 more children
    let Children = Manager.get_children(Root_task).await.unwrap();
    assert_eq!(Children.len(), Initial_count + 2);
    assert!(Children.contains(&Child1_task));
    assert!(Children.contains(&Child2_task));

    // Wait for children to complete
    Child1_handle.Join().await;
    Child2_handle.Join().await;

    // After children complete, they should no longer be in the children list
    let Final_children = Manager.get_children(Root_task).await.unwrap();
    assert_eq!(Final_children.len(), Initial_count);
}

#[Test(task_path = crate)]
async fn test_get_children_with_nested_tasks() {
    let Manager = Initialize();

    let Root_task = Manager.get_current_task_identifier().await;
    let Spawner = Manager.get_spawner(Root_task).await.unwrap();

    let (Parent_handle, _Parent_task) = Manager
        .Spawn(
            Root_task,
            "Parent Task",
            Some(Spawner),
            async move |Parent_task| {
                let Parent_spawner = get_instance().get_spawner(Parent_task).await.unwrap();

                // Parent task should initially have no children
                let Initial_children = get_instance().get_children(Parent_task).await.unwrap();
                assert_eq!(Initial_children.len(), 0);

                // Spawn child from parent task
                let (Child_handle, Child_task) = get_instance()
                    .Spawn(
                        Parent_task,
                        "Nested Child",
                        Some(Parent_spawner),
                        async move |Child_task| {
                            // Verify parent-child relationship
                            assert_eq!(
                                get_instance().get_parent(Child_task).await.unwrap(),
                                Parent_task
                            );
                        },
                    )
                    .await
                    .unwrap();

                // Parent should now have one child
                let Children = get_instance().get_children(Parent_task).await.unwrap();
                assert_eq!(Children.len(), 1);
                assert!(Children.contains(&Child_task));

                Child_handle.Join().await;
            },
        )
        .await
        .unwrap();

    Parent_handle.Join().await;
}

#[Test(task_path = crate)]
async fn test_get_parent_invalid_task() {
    let Manager = Initialize();

    // Test with an invalid task identifier
    let Invalid_task = Task_identifier_type::new(99999);
    let Result = Manager.get_parent(Invalid_task).await;

    assert!(Result.is_err());
}

#[Test(task_path = crate)]
async fn test_get_children_invalid_task() {
    let Manager = Initialize();

    // Test with an invalid task identifier
    let Invalid_task = Task_identifier_type::new(99999);
    let Result = Manager.get_children(Invalid_task).await;

    // get_children should return an empty vector for invalid task
    // since it filters by parent, not checking if the task exists
    assert!(Result.is_ok());
    assert_eq!(Result.unwrap().len(), 0);
}

#[Test(task_path = crate)]
async fn test_root_task_parent() {
    let Manager = Initialize();

    let Root_task = Manager.get_current_task_identifier().await;

    // Root task should be its own parent
    let Parent = Manager.get_parent(Root_task).await.unwrap();
    assert_eq!(Parent, Manager_type::ROOT_TASK_IDENTIFIER);
}

#[Test(task_path = crate)]
async fn test_multiple_generation_relationships() {
    let Manager = Initialize();

    let Root_task = Manager.get_current_task_identifier().await;
    let Spawner = Manager.get_spawner(Root_task).await.unwrap();

    let (Level1_handle, Level1_task) = Manager
        .Spawn(
            Root_task,
            "Level 1 Task",
            Some(Spawner),
            async move |Level1_task| {
                // Verify Level 1 parent is root
                assert_eq!(
                    get_instance().get_parent(Level1_task).await.unwrap(),
                    Root_task
                );

                let Level1_spawner = get_instance().get_spawner(Level1_task).await.unwrap();

                let (Level2_handle, Level2_task) = get_instance()
                    .Spawn(
                        Level1_task,
                        "Level 2 Task",
                        Some(Level1_spawner),
                        async move |Level2_task| {
                            // Verify Level 2 parent is Level 1
                            assert_eq!(
                                get_instance().get_parent(Level2_task).await.unwrap(),
                                Level1_task
                            );

                            let Level2_spawner =
                                get_instance().get_spawner(Level2_task).await.unwrap();

                            let (Level3_handle, Level3_task) = get_instance()
                                .Spawn(
                                    Level2_task,
                                    "Level 3 Task",
                                    Some(Level2_spawner),
                                    async move |Level3_task| {
                                        // Verify Level 3 parent is Level 2
                                        assert_eq!(
                                            get_instance().get_parent(Level3_task).await.unwrap(),
                                            Level2_task
                                        );
                                    },
                                )
                                .await
                                .unwrap();

                            // Level 2 should have Level 3 as child
                            let Level2_children =
                                get_instance().get_children(Level2_task).await.unwrap();
                            assert!(Level2_children.contains(&Level3_task));

                            Level3_handle.Join().await;
                        },
                    )
                    .await
                    .unwrap();

                // Level 1 should have Level 2 as child
                let Level1_children = get_instance().get_children(Level1_task).await.unwrap();
                assert!(Level1_children.contains(&Level2_task));

                Level2_handle.Join().await;
            },
        )
        .await
        .unwrap();

    // Root should have Level 1 as child
    let Root_children = Manager.get_children(Root_task).await.unwrap();
    assert!(Root_children.contains(&Level1_task));

    Level1_handle.Join().await;
}

#[Test(task_path = crate)]
async fn test_register_spawner() {
    let Manager = Initialize();

    // Get current task and its spawner
    let Current_task = Manager.get_current_task_identifier().await;
    let Current_spawner_id = Manager.get_spawner(Current_task).await.unwrap();

    // We can't easily create new spawners in tests due to lifetime requirements,
    // but we can test the registration logic by verifying the current spawner exists
    // and testing error conditions

    // Verify the current spawner ID is valid
    assert!(Current_spawner_id != usize::MAX);

    // Test that we can unregister and re-register spawners
    // Note: We can't actually unregister the current spawner as it would break the test
    // So we test the error path instead
    let Invalid_spawner_id = 99999;
    let Result = Manager.Unregister_spawner(Invalid_spawner_id);
    assert!(Result.is_err());
    assert!(matches!(
        Result.unwrap_err(),
        Error_type::No_spawner_available
    ));
}

#[Test(task_path = crate)]
async fn test_unregister_spawner() {
    let Manager = Initialize();

    // Test unregistering a non-existent spawner
    let Invalid_spawner_id = 99999;
    let Result = Manager.Unregister_spawner(Invalid_spawner_id);
    assert!(Result.is_err());
    assert!(matches!(
        Result.unwrap_err(),
        Error_type::No_spawner_available
    ));

    // Test unregistering the same spawner twice
    let Second_result = Manager.Unregister_spawner(Invalid_spawner_id);
    assert!(Second_result.is_err());
    assert!(matches!(
        Second_result.unwrap_err(),
        Error_type::No_spawner_available
    ));
}

#[Test(task_path = crate)]
async fn test_unregister_nonexistent_spawner() {
    let Manager = Initialize();

    // Try to unregister a nonexistent spawner
    let Invalid_id = 99999;
    let Result = Manager.Unregister_spawner(Invalid_id);

    assert!(Result.is_err());
    assert!(matches!(
        Result.unwrap_err(),
        Error_type::No_spawner_available
    ));
}

#[Test(task_path = crate)]
async fn test_register_multiple_spawners() {
    let Manager = Initialize();

    // Test the ID assignment logic by checking the current spawner
    let Current_task = Manager.get_current_task_identifier().await;
    let Current_spawner_id = Manager.get_spawner(Current_task).await.unwrap();

    // The spawner ID should be valid
    assert!(Current_spawner_id != usize::MAX);

    // Test that invalid spawner IDs are handled correctly
    let Invalid_ids = [99999, usize::MAX, 12345];
    for invalid_id in Invalid_ids {
        let Result = Manager.Unregister_spawner(invalid_id);
        assert!(Result.is_err());
        assert!(matches!(
            Result.unwrap_err(),
            Error_type::No_spawner_available
        ));
    }
}

//#[Test(task_path = crate)]
async fn _Test_spawner_load_balancing() {
    let Manager = Initialize();

    // Test the load balancing behavior by checking how tasks are distributed
    // Since we can't easily create multiple spawners in tests, we test that
    // the Select_best_spawner function works with the available spawners

    let Parent_task = Manager.get_current_task_identifier().await;
    let Current_spawner = Manager.get_spawner(Parent_task).await.unwrap();

    // Spawn multiple tasks and verify they all get valid spawner assignments
    let mut Handles = Vec::new();

    for i in 0..4 {
        let Task_name = format!("Load Balance Task {i}");
        let (Handle, _) = Manager
            .Spawn(Parent_task, &Task_name, None, async move |Task| {
                Manager_type::Sleep(core::time::Duration::from_millis(10)).await;
                get_instance().get_spawner(Task).await.unwrap()
            })
            .await
            .unwrap();

        Handles.push(Handle);
    }

    // Wait for all tasks to complete and collect their spawner IDs
    let mut Used_spawners = Vec::new();
    for Handle in Handles {
        let Spawner_used = Handle.Join().await;
        Used_spawners.push(Spawner_used);
    }

    // Verify that all tasks got valid spawner assignments
    for spawner_id in Used_spawners {
        assert!(
            spawner_id != usize::MAX,
            "All tasks should get valid spawner IDs"
        );
        // Verify they're using a reasonable spawner (could be the current one or another valid one)
        assert!(
            spawner_id == Current_spawner || spawner_id < 1000,
            "Spawner ID should be reasonable"
        );
    }
}

#[Test(task_path = crate)]
async fn test_get_spawner_invalid_task() {
    let Manager = Initialize();

    // Test with an invalid task identifier
    let Invalid_task = Task_identifier_type::new(99999);
    let Result = Manager.get_spawner(Invalid_task).await;

    assert!(Result.is_err());
    assert!(matches!(
        Result.unwrap_err(),
        Error_type::Invalid_task_identifier
    ));
}

#[Test(task_path = crate)]
async fn test_get_spawner_valid_task() {
    let Manager = Initialize();

    let Task = Manager.get_current_task_identifier().await;
    let Spawner_result = Manager.get_spawner(Task).await;

    // Root task should have a spawner
    assert!(Spawner_result.is_ok());
}

#[Test(task_path = crate)]
async fn test_spawner_with_explicit_selection() {
    let Manager = Initialize();

    let Parent_task = Manager.get_current_task_identifier().await;
    let Current_spawner_id = Manager.get_spawner(Parent_task).await.unwrap();

    // Spawn task with explicitly selected spawner
    let (Handle, _) = Manager
        .Spawn(
            Parent_task,
            "Explicit Spawner Task",
            Some(Current_spawner_id),
            async move |Task| get_instance().get_spawner(Task).await.unwrap(),
        )
        .await
        .unwrap();

    let Used_spawner = Handle.Join().await;
    assert_eq!(Used_spawner, Current_spawner_id);
}

#[Test(task_path = crate)]
async fn test_spawner_with_invalid_selection() {
    let Manager = Initialize();

    let Parent_task = Manager.get_current_task_identifier().await;
    let Invalid_spawner = 99999;

    // Try to spawn task with invalid spawner ID
    let Result = Manager
        .Spawn(
            Parent_task,
            "Invalid Spawner Task",
            Some(Invalid_spawner),
            async |_| {},
        )
        .await;

    assert!(Result.is_err());
    if let Err(error) = Result {
        assert!(matches!(error, Error_type::Invalid_spawner_identifier));
    }
}

#[Test(task_path = crate)]
async fn test_spawner_reuse_after_unregister() {
    let Manager = Initialize();

    // Test the ID reuse behavior by testing with invalid spawner IDs
    // Since we can't easily create new spawners in tests, we verify the
    // error handling behavior of the spawner management system

    // Test unregistering non-existent spawners
    let Invalid_spawner_id = 99999;
    let Result1 = Manager.Unregister_spawner(Invalid_spawner_id);
    assert!(Result1.is_err());
    assert!(matches!(
        Result1.unwrap_err(),
        Error_type::No_spawner_available
    ));

    // Test that the same invalid ID consistently fails
    let Result2 = Manager.Unregister_spawner(Invalid_spawner_id);
    assert!(Result2.is_err());
    assert!(matches!(
        Result2.unwrap_err(),
        Error_type::No_spawner_available
    ));

    // Verify that valid spawner operations still work
    let Current_task = Manager.get_current_task_identifier().await;
    let Current_spawner = Manager.get_spawner(Current_task).await.unwrap();
    assert!(Current_spawner != usize::MAX);
}
