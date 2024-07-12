use crate::{Position_type, Result_type};

pub trait Device_trait: Send + Sync {
    fn Read(&self, Buffer: &mut [u8]) -> Result_type<usize>;

    fn Write(&self, Buffer: &[u8]) -> Result_type<usize>;

    fn Get_size(&self) -> Result_type<usize>;

    fn Set_position(&self, Position: &Position_type) -> Result_type<usize>;

    fn Flush(&self) -> Result_type<()>;
}
