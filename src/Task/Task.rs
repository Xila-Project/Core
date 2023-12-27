use std::thread::Thread;

use super::*;

pub type Task_identifier_type = usize;

pub struct Task_type<'a> {
    Identifier: Task_identifier_type,
    Manager: &'a Manager_type,
}

impl<'a> Task_type<'a> {}
