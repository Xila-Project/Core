use file_system::{BaseOperations, Context, Size};

pub struct HttpClientDevice {
    // Fields for the HTTP client can be added here if needed
}

impl HttpClientDevice {
    pub fn new() -> Self {
        HttpClientDevice {}
    }
}

impl BaseOperations for HttpClientDevice {
    fn read(
        &self,
        _context: &mut Context,
        _buffer: &mut [u8],
        _: Size,
    ) -> file_system::Result<usize> {
        todo!()
    }

    fn write(&self, _context: &mut Context, _buffer: &[u8], _: Size) -> file_system::Result<usize> {
        todo!()
    }

    fn clone_context(&self, _context: &Context) -> file_system::Result<Context> {
        Ok(Context::new_empty())
    }
}
