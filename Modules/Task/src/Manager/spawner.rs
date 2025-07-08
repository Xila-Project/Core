// Spawner module - handles spawner registration and selection

use super::*;

use alloc::collections::BTreeMap;
use embassy_executor::Spawner;

impl Manager_type {
    pub fn register_spawner(&'static self, Spawner: Spawner) -> Result_type<usize> {
        let mut inner = embassy_futures::block_on(self.0.write());

        let Identifier = Self::Find_first_available_identifier(
            &inner.spawners,
            (usize::MIN..usize::MAX).step_by(1),
        )
        .ok_or(Error_type::Too_many_spawners)?;

        if inner.spawners.insert(Identifier, Spawner).is_some() {
            unreachable!("Spawner identifier already exists");
        }

        Ok(Identifier)
    }

    pub fn Unregister_spawner(&'static self, Identifier: usize) -> Result_type<()> {
        let mut inner = embassy_futures::block_on(self.0.write());

        inner
            .spawners
            .remove(&Identifier)
            .ok_or(Error_type::No_spawner_available)?;

        Ok(())
    }

    /// Select the best spawner for a new task using load balancing algorithm
    pub(crate) fn Select_best_spawner(Inner: &Inner_type) -> Result_type<usize> {
        if Inner.spawners.is_empty() {
            return Err(Error_type::No_spawner_available);
        }

        let mut Map = BTreeMap::new();

        for Identifier in Inner.spawners.keys() {
            Map.insert(*Identifier, 0); // Initialize all spawners with a load of 0
        }

        for Metadata in Inner.tasks.values() {
            if let Some(load) = Map.get_mut(&Metadata.Spawner_identifier) {
                *load += 1; // Increment the load for the spawner
            }
        }

        // Find the spawner with the lowest load score
        let mut Best_index = 0;
        let mut best_score = usize::MAX;

        for (Identifier, Spawner) in Map.iter() {
            if *Spawner < best_score {
                best_score = *Spawner;
                Best_index = *Identifier;
            }
        }

        Ok(Best_index)
    }

    pub async fn get_spawner(&self, Task: Task_identifier_type) -> Result_type<usize> {
        Self::get_task(&*self.0.read().await, Task)
            .map(|task| task.Spawner_identifier)
            .map_err(|_| Error_type::Invalid_task_identifier)
    }
}
