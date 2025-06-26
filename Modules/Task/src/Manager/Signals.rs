// Signals module - handles task signal management

use super::*;

impl Manager_type {
    pub async fn Send_signal(
        &self,
        Task_identifier: Task_identifier_type,
        Signal: Signal_type,
    ) -> Result_type<()> {
        Self::Get_task_mutable(&mut *self.0.write().await, Task_identifier)
            .map(|Task| Task.Signals.Send(Signal))
    }

    pub async fn Pop_signal(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Option<Signal_type>> {
        Self::Get_task_mutable(&mut *self.0.write().await, Task_identifier)
            .map(|Task| Task.Signals.Pop())
    }

    pub async fn Peek_signal(
        &self,
        Task_identifier: Task_identifier_type,
    ) -> Result_type<Option<Signal_type>> {
        Self::Get_task(&*self.0.read().await, Task_identifier).map(|Task| Task.Signals.Peek())
    }
}
