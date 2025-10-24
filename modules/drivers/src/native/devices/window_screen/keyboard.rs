use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, RawMutex},
    channel::Receiver,
};
use file_system::{DeviceTrait, Size};
use graphics::{InputData, Key, State};

pub struct KeyboardDevice<M, const N: usize>(Receiver<'static, M, (Key, State), N>)
where
    M: RawMutex + 'static;

impl<M, const N: usize> KeyboardDevice<M, N>
where
    M: RawMutex + 'static,
{
    pub fn new(receiver: Receiver<'static, M, (Key, State), N>) -> Self {
        Self(receiver)
    }
}

impl<const N: usize> DeviceTrait for KeyboardDevice<CriticalSectionRawMutex, N> {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<Size> {
        // - Cast
        let data: &mut InputData = buffer
            .try_into()
            .map_err(|_| file_system::Error::InvalidParameter)?;

        if let Ok((key, state)) = self.0.try_receive() {
            data.set_key(key);
            data.set_state(state);
        }

        data.set_continue(!self.0.is_empty());

        Ok(size_of::<InputData>().into())
    }

    fn write(&self, _: &[u8]) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn get_size(&self) -> file_system::Result<Size> {
        Ok(size_of::<InputData>().into())
    }

    fn set_position(&self, _: &file_system::Position) -> file_system::Result<Size> {
        Err(file_system::Error::UnsupportedOperation)
    }

    fn flush(&self) -> file_system::Result<()> {
        Ok(())
    }
}
