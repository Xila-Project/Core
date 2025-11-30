use crate::{Shell, error::Result};
use alloc::{vec, vec::Vec};
use getargs::{Arg, Options};
use xila::{
    file_system::{AccessFlags, Path},
    shared::{HttpRequestBuilder, Url},
    virtual_file_system::File,
};

impl Shell {
    pub async fn web_request<'a, I>(&mut self, options: &mut Options<&'a str, I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut headers: Vec<&'a str> = Vec::with_capacity(20);
        let mut method: &'a str = "GET";
        let mut url: Option<&'a str> = None;
        let mut body: Option<&'a str> = None;
        let mut body_file: Option<&'a Path> = None;

        while let Some(argument) = options.next_arg()? {
            match argument {
                Arg::Long("header") | Arg::Short('h') => {
                    let value = options.value()?;
                    headers.push(value);
                }
                Arg::Long("method") | Arg::Short('m') => {
                    method = options.value()?;
                }
                Arg::Long("body") | Arg::Short('b') => {
                    let value = options.value()?;
                    if body.replace(value).is_some() {
                        xila::log::error!("Body already set");
                        return Err(crate::error::Error::InvalidArgument);
                    }
                }
                Arg::Long("body-file") | Arg::Short('f') => {
                    let value = options.value()?;
                    let path = Path::from_str(value);
                    if body_file.replace(path).is_some() {
                        xila::log::error!("Body file already set");
                        return Err(crate::error::Error::InvalidArgument);
                    }
                }
                Arg::Positional(positional) => {
                    if let Some(u) = url.replace(positional) {
                        xila::log::error!("URL already set : {:?}", u);
                        return Err(crate::error::Error::InvalidArgument);
                    }
                }
                e => {
                    xila::log::error!("Unexpected argument: {:?}", e);
                    return Err(crate::error::Error::InvalidArgument);
                }
            }
        }

        let virtual_file_system = xila::virtual_file_system::get_instance();
        let task = self.task;

        let mut buffer: Vec<u8> = vec![0; 4096];

        let url = Url::parse(url.ok_or(crate::error::Error::MissingPositionalArgument("url"))?)
            .ok_or(crate::error::Error::InvalidArgument)?;

        let mut request_builder = HttpRequestBuilder::from_buffer(&mut buffer);

        request_builder
            .add_request(method, url.path, HttpRequestBuilder::HTTP_VERSION_1_1)
            .ok_or(crate::error::Error::InvalidArgument)?
            .add_header("Host", url.host.as_bytes())
            .ok_or(crate::error::Error::InvalidArgument)?
            .add_header("Connection", b"close")
            .ok_or(crate::error::Error::InvalidArgument)?;

        for header in headers {
            let (name, value) = header
                .split_once(':')
                .ok_or(crate::error::Error::InvalidArgument)?;
            let (name, value) = (name.trim(), value.trim());

            request_builder
                .add_header(name, value.as_bytes())
                .ok_or(crate::error::Error::InvalidArgument)?;
        }

        if let Some(body) = body {
            request_builder
                .add_body(body.as_bytes())
                .ok_or(crate::error::Error::InvalidArgument)?;
        } else if let Some(body_file) = body_file {
            let mut buffer: Vec<u8> = Vec::with_capacity(4096);

            File::read_from_path(virtual_file_system, task, body_file, &mut buffer)
                .await
                .map_err(crate::error::Error::FailedToOpenFile)?;

            request_builder
                .add_body(&buffer)
                .ok_or(crate::error::Error::InvalidArgument)?;
        }

        // Open http client device
        let mut file = File::open(
            virtual_file_system,
            task,
            "/devices/https_client",
            AccessFlags::READ_WRITE.into(),
        )
        .await
        .map_err(crate::error::Error::FailedToOpenFile)?;

        // Write request
        file.write(&buffer)
            .await
            .map_err(crate::error::Error::FailedToOpenFile)?;

        // Read header

        buffer.fill(0);

        let bytes_read = file
            .read(&mut buffer)
            .await
            .map_err(crate::error::Error::FailedToOpenFile)?;

        let _ = self.standard.out().write(&buffer[..bytes_read]).await;
        // Read body

        buffer.fill(0);

        let bytes_read = file
            .read(&mut buffer)
            .await
            .map_err(crate::error::Error::FailedToOpenFile)?;

        let bytes_print = bytes_read.min(128);

        let _ = self.standard.out().write_line(&buffer[..bytes_print]).await;

        file.close(virtual_file_system)
            .await
            .map_err(crate::error::Error::FailedToOpenFile)?;

        Ok(())
    }
}

// Test command:
// web_request -m POST -h "Content-Type:application/json" -b "{\"key\":\"value\"}" https://httpbingo.org/post
