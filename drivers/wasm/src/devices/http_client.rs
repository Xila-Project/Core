use file_system::DeviceTrait;

pub struct HttpClientDevice {
    // Fields for the HTTP client can be added here if needed
}

impl HttpClientDevice {
    pub fn new() -> Self {
        HttpClientDevice {}
    }
}

impl DeviceTrait for HttpClientDevice {
    fn read(&self, buffer: &mut [u8]) -> file_system::Result<file_system::Size> {
        todo!()
    }

    fn write(&self, buffer: &[u8]) -> file_system::Result<file_system::Size> {
        todo!()
    }

    fn get_size(&self) -> file_system::Result<file_system::Size> {
        todo!()
    }

    fn set_position(
        &self,
        position: &file_system::Position,
    ) -> file_system::Result<file_system::Size> {
        todo!()
    }

    fn flush(&self) -> file_system::Result<()> {
        todo!()
    }
}
