// Utilities module - helper functions and common operations

use super::*;

use alloc::collections::BTreeMap;
use core::fmt::Debug;

impl Manager_type {
    /// Find the first available identifier for any identifier type with Into_inner() method
    pub(crate) fn Find_first_available_identifier<Raw_identifier_type, Identifier_type, V>(
        Map: &BTreeMap<Identifier_type, V>,
        mut Range: impl Iterator<Item = Raw_identifier_type>,
    ) -> Option<Identifier_type>
    where
        Identifier_type: PartialEq<Raw_identifier_type> + From<Raw_identifier_type> + Debug,
        Raw_identifier_type: Debug,
    {
        for Key in Map.keys() {
            match Range.next() {
                Some(Test_key) => {
                    if *Key != Test_key {
                        return Some(Test_key.into());
                    }
                }

                None => return None, // No more identifiers to check
            }
        }

        Range.next().map(|Key| Key.into())
    }

    pub(crate) fn Get_task(
        Inner: &Inner_type,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<&Metadata_type> {
        Inner
            .Tasks
            .get(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)
    }

    pub(crate) fn Get_task_mutable(
        Inner: &mut Inner_type,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<&mut Metadata_type> {
        Inner
            .Tasks
            .get_mut(&Task_identifier)
            .ok_or(Error_type::Invalid_task_identifier)
    }
}
