// Utilities module - helper functions and common operations

use super::*;

use alloc::collections::BTreeMap;
use core::fmt::Debug;

impl Manager_type {
    /// Find the first available identifier for any identifier type with Into_inner() method
    pub(crate) fn Find_first_available_identifier<Raw_identifier_type, Identifier_type, V>(
        map: &BTreeMap<Identifier_type, V>,
        mut range: impl Iterator<Item = Raw_identifier_type>,
    ) -> Option<Identifier_type>
    where
        Identifier_type: PartialEq<Raw_identifier_type> + From<Raw_identifier_type> + Debug,
        Raw_identifier_type: Debug,
    {
        for Key in map.keys() {
            match range.next() {
                Some(test_key) => {
                    if *Key != test_key {
                        return Some(test_key.into());
                    }
                }

                None => return None, // No more identifiers to check
            }
        }

        range.next().map(|Key| Key.into())
    }

    pub(crate) fn Get_task(
        inner: &Inner_type,
        task_identifier: Task_identifier_type,
    ) -> Result_type<&Metadata_type> {
        inner
            .tasks
            .get(&task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)
    }

    pub(crate) fn Get_task_mutable(
        inner: &mut Inner_type,
        task_identifier: Task_identifier_type,
    ) -> Result_type<&mut Metadata_type> {
        inner
            .tasks
            .get_mut(&task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)
    }
}
