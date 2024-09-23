use crate::Duration_type;

pub trait Driver_trait: Send + Sync {
    fn Get_instant_since_startup(&self) -> Duration_type;

    fn Get_current_time(&self) -> Duration_type;
}
