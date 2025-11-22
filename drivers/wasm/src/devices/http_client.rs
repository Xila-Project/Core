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
        context: &mut Context,
        buffer: &mut [u8],
        _: Size,
    ) -> file_system::Result<usize> {
        todo!()
    }

    fn write(&self, context: &mut Context, buffer: &[u8], _: Size) -> file_system::Result<usize> {
        todo!()
    }

    fn clone_context(&self, context: &Context) -> file_system::Result<Context> {
        Ok(Context::new_empty())
    }
}
