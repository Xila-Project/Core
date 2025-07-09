// Signals module - handles task signal management

use super::*;

impl Manager_type {
    pub async fn send_signal(
        &self,
        task_identifier: Task_identifier_type,
        signal: Signal_type,
    ) -> Result_type<()> {
        Self::get_task_mutable(&mut *self.0.write().await, task_identifier)
            .map(|task| task.signals.send(signal))
    }

    pub async fn pop_signal(
        &self,
        task_identifier: Task_identifier_type,
    ) -> Result_type<Option<Signal_type>> {
        Self::get_task_mutable(&mut *self.0.write().await, task_identifier)
            .map(|task| task.signals.pop())
    }

    pub async fn peek_signal(
        &self,
        task_identifier: Task_identifier_type,
    ) -> Result_type<Option<Signal_type>> {
        Self::get_task(&*self.0.read().await, task_identifier).map(|task| task.signals.peek())
    }
}
