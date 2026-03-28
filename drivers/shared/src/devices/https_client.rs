use alloc::boxed::Box;
use core::mem;

use embedded_io as embedded_io_v06;
use embedded_io_v06::Write as _;
use embedded_tls::blocking::{Aes128GcmSha256, NoVerify, TlsConfig, TlsConnection, TlsContext};
use file_system::{BaseOperations, CharacterDevice, Context, Error, MountOperations, Result, Size};
use network::{DnsQueryKind, Duration as NetworkDuration, Port, TcpSocket};
use rand_core::{CryptoRng, RngCore};
use shared::{HttpRequestParser, HttpResponseBuilder, HttpResponseParser};

const TLS_RECORD_BUFFER_SIZE: usize = 4096;
const RESPONSE_SCAN_BUFFER_SIZE: usize = 3072;
const RESPONSE_SERIALIZED_HEADER_SIZE: usize = 2048;
const RESPONSE_BODY_PREFIX_SIZE: usize = RESPONSE_SCAN_BUFFER_SIZE;
const TLS_READ_CHUNK_SIZE: usize = 512;
const DEFAULT_HTTPS_PORT: u16 = 443;
const IO_TIMEOUT_SECONDS: u64 = 15;

struct SystemRng;

impl CryptoRng for SystemRng {}

impl RngCore for SystemRng {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        getrandom::fill(&mut bytes).expect("SystemRng failed to gather u32 entropy");
        u32::from_le_bytes(bytes)
    }

    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        getrandom::fill(&mut bytes).expect("SystemRng failed to gather u64 entropy");
        u64::from_le_bytes(bytes)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        getrandom::fill(dest).expect("SystemRng failed to gather entropy");
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> core::result::Result<(), rand_core::Error> {
        getrandom::fill(dest).map_err(|_| rand_core::Error::from(core::num::NonZeroU32::MIN))
    }
}

#[derive(Debug, Copy, Clone)]
enum IoError {
    FileSystem(Error),
}

impl embedded_io_v06::Error for IoError {
    fn kind(&self) -> embedded_io_v06::ErrorKind {
        match self {
            IoError::FileSystem(Error::NotFound) => embedded_io_v06::ErrorKind::NotFound,
            IoError::FileSystem(Error::PermissionDenied) => {
                embedded_io_v06::ErrorKind::PermissionDenied
            }
            _ => embedded_io_v06::ErrorKind::Other,
        }
    }
}

struct TcpSocketAdapter {
    socket: TcpSocket,
}

impl TcpSocketAdapter {
    fn new(mut socket: TcpSocket) -> Self {
        task::block_on(socket.set_timeout(Some(NetworkDuration::from_seconds(IO_TIMEOUT_SECONDS))));
        Self { socket }
    }
}

impl embedded_io_v06::ErrorType for TcpSocketAdapter {
    type Error = IoError;
}

impl embedded_io_v06::Read for TcpSocketAdapter {
    fn read(&mut self, buffer: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        task::block_on(self.socket.read(buffer))
            .map_err(map_network_error)
            .map_err(IoError::FileSystem)
    }
}

impl embedded_io_v06::Write for TcpSocketAdapter {
    fn write(&mut self, buffer: &[u8]) -> core::result::Result<usize, Self::Error> {
        task::block_on(self.socket.write(buffer))
            .map_err(map_network_error)
            .map_err(IoError::FileSystem)
    }

    fn flush(&mut self) -> core::result::Result<(), Self::Error> {
        task::block_on(self.socket.flush())
            .map_err(map_network_error)
            .map_err(IoError::FileSystem)
    }
}

struct Session {
    tls_ptr: *mut TlsConnection<'static, TcpSocketAdapter, Aes128GcmSha256>,
    read_record_ptr: *mut [u8; TLS_RECORD_BUFFER_SIZE],
    write_record_ptr: *mut [u8; TLS_RECORD_BUFFER_SIZE],
    closed: bool,
}

unsafe impl Send for Session {}
unsafe impl Sync for Session {}

impl Session {
    fn tls_mut(&mut self) -> &mut TlsConnection<'static, TcpSocketAdapter, Aes128GcmSha256> {
        unsafe { &mut *self.tls_ptr }
    }

    fn close(&mut self) {
        if self.closed {
            return;
        }

        let tls = unsafe { *Box::from_raw(self.tls_ptr) };
        match tls.close() {
            Ok(_) => {}
            Err((mut adapter, _)) => {
                let _ = task::block_on(adapter.socket.close());
            }
        }

        unsafe {
            let _ = Box::from_raw(self.read_record_ptr);
            let _ = Box::from_raw(self.write_record_ptr);
        }

        self.closed = true;
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.close();
    }
}

enum State {
    Idle,
    HeadersReady(Session),
    BodyStreaming(Session),
    Failed(Error),
}

struct HttpsClientContext {
    state: State,
    response_headers: [u8; RESPONSE_SERIALIZED_HEADER_SIZE],
    response_headers_len: usize,
    response_headers_cursor: usize,
    response_body_prefix: [u8; RESPONSE_BODY_PREFIX_SIZE],
    response_body_prefix_len: usize,
    response_body_prefix_cursor: usize,
}

impl HttpsClientContext {
    fn new() -> Self {
        Self {
            state: State::Idle,
            response_headers: [0; RESPONSE_SERIALIZED_HEADER_SIZE],
            response_headers_len: 0,
            response_headers_cursor: 0,
            response_body_prefix: [0; RESPONSE_BODY_PREFIX_SIZE],
            response_body_prefix_len: 0,
            response_body_prefix_cursor: 0,
        }
    }

    fn reset_buffers(&mut self) {
        self.response_headers_len = 0;
        self.response_headers_cursor = 0;
        self.response_body_prefix_len = 0;
        self.response_body_prefix_cursor = 0;
    }

    fn set_failed(&mut self, error: Error) {
        self.reset_buffers();
        self.state = State::Failed(error);
    }
}

unsafe impl Send for HttpsClientContext {}
unsafe impl Sync for HttpsClientContext {}

fn map_network_error(error: network::Error) -> Error {
    match error {
        network::Error::NotFound => Error::NotFound,
        network::Error::PermissionDenied => Error::PermissionDenied,
        network::Error::ConnectionRefused
        | network::Error::ConnectionReset
        | network::Error::ConnectionAborted
        | network::Error::TimedOut => Error::InputOutput,
        network::Error::InvalidInput
        | network::Error::InvalidData
        | network::Error::InvalidPort
        | network::Error::InvalidEndpoint => Error::InvalidParameter,
        network::Error::NoRoute
        | network::Error::HostUnreachable
        | network::Error::NetworkUnreachable => Error::NotFound,
        network::Error::ResourceBusy | network::Error::Pending => Error::RessourceBusy,
        _ => Error::Other,
    }
}

fn map_tls_error(_error: embedded_tls::TlsError) -> Error {
    Error::InputOutput
}

fn split_host_port(host_value: &str) -> (&str, u16) {
    if let Some(stripped) = host_value.strip_prefix('[') {
        if let Some(end) = stripped.find(']') {
            let host = &stripped[..end];
            let remainder = &stripped[end + 1..];
            if let Some(port_string) = remainder.strip_prefix(':') {
                if let Ok(port) = port_string.parse::<u16>() {
                    return (host, port);
                }
            }
            return (host, DEFAULT_HTTPS_PORT);
        }
    }

    if let Some((host, port_string)) = host_value.rsplit_once(':') {
        if !host.contains(':') {
            if let Ok(port) = port_string.parse::<u16>() {
                return (host, port);
            }
        }
    }

    (host_value, DEFAULT_HTTPS_PORT)
}

fn compute_request_length(buffer: &[u8], parser: HttpRequestParser<'_>) -> Result<usize> {
    let trimmed_tail = match buffer.iter().rposition(|byte| *byte != 0) {
        Some(position) => position + 1,
        None => return Err(Error::InvalidParameter),
    };

    let headers_end = buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4);

    let Some(headers_end) = headers_end else {
        return Ok(trimmed_tail);
    };

    let content_length = parser
        .get_headers()
        .find(|(name, _)| *name == HttpRequestParser::CONTENT_LENGTH_HEADER)
        .and_then(|(_, value)| value.parse::<usize>().ok());

    let body_length = match content_length {
        Some(length) => length,
        None => trimmed_tail.saturating_sub(headers_end),
    };

    let total_length = headers_end
        .checked_add(body_length)
        .ok_or(Error::InvalidParameter)?;

    if total_length == 0 || total_length > buffer.len() {
        return Err(Error::InvalidParameter);
    }

    Ok(total_length)
}

fn build_serialized_response_headers(raw_headers: &[u8], output: &mut [u8]) -> Result<usize> {
    let parser = HttpResponseParser::from_buffer(raw_headers);
    let status_code = parser.get_status_code().ok_or(Error::InvalidParameter)?;

    let mut builder = HttpResponseBuilder::from_buffer(output);
    builder
        .add_status_code(status_code)
        .ok_or(Error::InternalError)?;

    for (name, value) in parser.get_headers() {
        builder
            .add_header(name, value.as_bytes())
            .ok_or(Error::FileTooLarge)?;
    }

    builder.add_line(b"").ok_or(Error::FileTooLarge)?;
    Ok(builder.get_position())
}

fn read_response_headers_and_prefix(
    tls: &mut TlsConnection<'static, TcpSocketAdapter, Aes128GcmSha256>,
    raw_headers: &mut [u8; RESPONSE_SCAN_BUFFER_SIZE],
    body_prefix: &mut [u8; RESPONSE_BODY_PREFIX_SIZE],
) -> Result<(usize, usize)> {
    let mut filled = 0usize;
    let mut chunk = [0u8; TLS_READ_CHUNK_SIZE];

    loop {
        let bytes_read = tls.read(&mut chunk).map_err(map_tls_error)?;

        if bytes_read == 0 {
            return Err(Error::InputOutput);
        }

        let destination = raw_headers
            .get_mut(filled..filled + bytes_read)
            .ok_or(Error::FileTooLarge)?;
        destination.copy_from_slice(&chunk[..bytes_read]);
        filled += bytes_read;

        if let Some(headers_end) = raw_headers[..filled]
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|position| position + 4)
        {
            let body_length = filled.saturating_sub(headers_end);
            let copy_length = body_length.min(body_prefix.len());
            if copy_length > 0 {
                body_prefix[..copy_length]
                    .copy_from_slice(&raw_headers[headers_end..headers_end + copy_length]);
            }

            return Ok((headers_end, copy_length));
        }
    }
}

fn create_tls_session(host: &str, port: u16) -> Result<Session> {
    log::information!("https_client: create session host='{}' port={}", host, port);
    let manager = network::get_instance();

    let address = task::block_on(async {
        log::information!("https_client: resolving host='{}'", host);
        let dns_socket = manager
            .new_dns_socket(None)
            .await
            .map_err(map_network_error)?;
        let resolved = dns_socket
            .resolve(host, DnsQueryKind::A | DnsQueryKind::Aaaa)
            .await
            .map_err(map_network_error)?;
        dns_socket.close().await.map_err(map_network_error)?;

        let address = resolved.into_iter().next().ok_or(Error::NotFound)?;
        log::information!("https_client: dns resolved host='{}'", host);
        Ok::<network::IpAddress, Error>(address)
    })?;

    let socket = task::block_on(async {
        log::information!("https_client: creating tcp socket");
        let mut socket = manager
            .new_tcp_socket(4096, 4096, None)
            .await
            .map_err(map_network_error)?;
        log::information!(
            "https_client: connecting tcp socket to {}:{}",
            address,
            port
        );
        socket
            .connect(address, Port::from_inner(port))
            .await
            .map_err(map_network_error)?;
        log::information!("https_client: tcp connected");
        Ok::<TcpSocket, Error>(socket)
    })?;

    let read_record_ptr = Box::into_raw(Box::new([0u8; TLS_RECORD_BUFFER_SIZE]));
    let write_record_ptr = Box::into_raw(Box::new([0u8; TLS_RECORD_BUFFER_SIZE]));

    let read_record = unsafe { &mut *read_record_ptr };
    let write_record = unsafe { &mut *write_record_ptr };

    let mut tls = TlsConnection::new(TcpSocketAdapter::new(socket), read_record, write_record);

    let configuration = TlsConfig::new().with_server_name(host);
    let mut random = SystemRng;
    let context = TlsContext::new(&configuration, &mut random);

    log::information!("https_client: starting tls handshake");
    tls.open::<SystemRng, NoVerify>(context)
        .map_err(map_tls_error)?;
    log::information!("https_client: tls handshake done");

    let tls_ptr = Box::into_raw(Box::new(tls));

    Ok(Session {
        tls_ptr,
        read_record_ptr,
        write_record_ptr,
        closed: false,
    })
}

fn run_request(context: &mut HttpsClientContext, request: &[u8]) -> Result<()> {
    log::information!(
        "https_client: run_request begin (buffer_len={})",
        request.len()
    );
    let parser = HttpRequestParser::from_buffer(request);
    let _ = parser.get_request().ok_or(Error::InvalidParameter)?;

    let host_header = parser
        .get_headers()
        .find(|(name, _)| *name == HttpRequestParser::HOST_HEADER)
        .map(|(_, value)| value)
        .ok_or(Error::InvalidParameter)?;

    let (host, port) = split_host_port(host_header);
    log::information!("https_client: parsed host='{}' port={}", host, port);

    let mut session = create_tls_session(host, port)?;

    let request_length = compute_request_length(request, parser)?;
    let payload = &request[..request_length];
    log::information!("https_client: request length computed = {}", request_length);

    log::information!("https_client: tls write_all begin");
    session
        .tls_mut()
        .write_all(payload)
        .map_err(map_tls_error)?;

    let has_header_terminator = payload.windows(4).any(|window| window == b"\r\n\r\n");
    if !has_header_terminator {
        let suffix = if payload.ends_with(b"\r\n") {
            b"\r\n".as_slice()
        } else {
            b"\r\n\r\n".as_slice()
        };

        log::warning!(
            "https_client: request missing header terminator, appending {} bytes",
            suffix.len()
        );

        session.tls_mut().write_all(suffix).map_err(map_tls_error)?;
    }

    log::information!("https_client: tls write_all done");

    log::information!("https_client: tls flush begin");
    session.tls_mut().flush().map_err(map_tls_error)?;
    log::information!("https_client: tls flush done");

    let mut raw_headers = [0u8; RESPONSE_SCAN_BUFFER_SIZE];
    log::information!("https_client: waiting response headers");
    let (raw_headers_end, prefix_length) = read_response_headers_and_prefix(
        session.tls_mut(),
        &mut raw_headers,
        &mut context.response_body_prefix,
    )?;
    log::information!(
        "https_client: response headers received (headers_end={}, body_prefix={})",
        raw_headers_end,
        prefix_length
    );

    let serialized_headers_len = build_serialized_response_headers(
        &raw_headers[..raw_headers_end],
        &mut context.response_headers,
    )?;

    context.response_headers_len = serialized_headers_len;
    context.response_headers_cursor = 0;
    context.response_body_prefix_len = prefix_length;
    context.response_body_prefix_cursor = 0;
    context.state = State::HeadersReady(session);
    log::information!(
        "https_client: request complete, headers ready len={}",
        serialized_headers_len
    );

    Ok(())
}

pub struct HttpsClientDevice;

impl BaseOperations for HttpsClientDevice {
    fn open(&self, context: &mut Context) -> Result<()> {
        context.set_private_data(Box::new(HttpsClientContext::new()));
        Ok(())
    }

    fn close(&self, context: &mut Context) -> Result<()> {
        if let Some(client_context) = context.take_private_data_of_type::<HttpsClientContext>() {
            match client_context.state {
                State::HeadersReady(_session) | State::BodyStreaming(_session) => {}
                _ => {}
            }
        }

        Ok(())
    }

    fn read(&self, context: &mut Context, buffer: &mut [u8], _: Size) -> Result<usize> {
        let context = context
            .get_private_data_mutable_of_type::<HttpsClientContext>()
            .ok_or(Error::InvalidParameter)?;

        let state = mem::replace(&mut context.state, State::Idle);

        match state {
            State::Idle => {
                context.state = State::Idle;
                Ok(0)
            }
            State::Failed(error) => {
                context.state = State::Idle;
                Err(error)
            }
            State::HeadersReady(session) => {
                let remaining = context
                    .response_headers_len
                    .saturating_sub(context.response_headers_cursor);

                let bytes_to_copy = remaining.min(buffer.len());

                buffer[..bytes_to_copy].copy_from_slice(
                    &context.response_headers[context.response_headers_cursor
                        ..context.response_headers_cursor + bytes_to_copy],
                );

                context.response_headers_cursor += bytes_to_copy;

                if context.response_headers_cursor >= context.response_headers_len {
                    context.state = State::BodyStreaming(session);
                } else {
                    context.state = State::HeadersReady(session);
                }

                Ok(bytes_to_copy)
            }
            State::BodyStreaming(mut session) => {
                if context.response_body_prefix_cursor < context.response_body_prefix_len {
                    let remaining = context
                        .response_body_prefix_len
                        .saturating_sub(context.response_body_prefix_cursor);
                    let bytes_to_copy = remaining.min(buffer.len());

                    buffer[..bytes_to_copy].copy_from_slice(
                        &context.response_body_prefix[context.response_body_prefix_cursor
                            ..context.response_body_prefix_cursor + bytes_to_copy],
                    );
                    context.response_body_prefix_cursor += bytes_to_copy;

                    context.state = State::BodyStreaming(session);
                    return Ok(bytes_to_copy);
                }

                match session.tls_mut().read(buffer).map_err(map_tls_error) {
                    Ok(0) => {
                        context.state = State::Idle;
                        context.reset_buffers();
                        Ok(0)
                    }
                    Ok(bytes_read) => {
                        context.state = State::BodyStreaming(session);
                        Ok(bytes_read)
                    }
                    Err(error) => {
                        context.set_failed(error);
                        Err(error)
                    }
                }
            }
        }
    }

    fn write(&self, context: &mut Context, buffer: &[u8], _: Size) -> Result<usize> {
        log::information!("https_client: write called size={}", buffer.len());
        let context = context
            .get_private_data_mutable_of_type::<HttpsClientContext>()
            .ok_or(Error::InvalidParameter)?;

        let state = mem::replace(&mut context.state, State::Idle);
        match state {
            State::Idle | State::Failed(_) => {}
            State::HeadersReady(session) => {
                context.state = State::HeadersReady(session);
                log::warning!("https_client: write rejected (headers still pending read)");
                return Err(Error::RessourceBusy);
            }
            State::BodyStreaming(session) => {
                context.state = State::BodyStreaming(session);
                log::warning!("https_client: write rejected (body streaming)");
                return Err(Error::RessourceBusy);
            }
        }

        context.reset_buffers();

        if let Err(error) = run_request(context, buffer) {
            context.set_failed(error);
            log::error!("https_client: run_request failed: {:?}", error);
            return Err(error);
        }

        log::information!("https_client: write completed successfully");
        Ok(buffer.len())
    }

    fn clone_context(&self, _context: &Context) -> Result<Context> {
        Ok(Context::new(Some(HttpsClientContext::new())))
    }
}

impl MountOperations for HttpsClientDevice {}

impl CharacterDevice for HttpsClientDevice {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_host_port_parses_default_port() {
        let (host, port) = split_host_port("example.com");
        assert_eq!(host, "example.com");
        assert_eq!(port, 443);
    }

    #[test]
    fn split_host_port_parses_explicit_port() {
        let (host, port) = split_host_port("example.com:8443");
        assert_eq!(host, "example.com");
        assert_eq!(port, 8443);
    }

    #[test]
    fn compute_request_length_ignores_trailing_zeroes() {
        let request = b"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n";
        let mut request_buffer = [0u8; 256];
        request_buffer[..request.len()].copy_from_slice(request);
        let parser = HttpRequestParser::from_buffer(&request_buffer);

        let length = compute_request_length(&request_buffer, parser).unwrap();
        assert_eq!(length, request.len());
    }

    #[test]
    fn compute_request_length_accepts_no_header_terminator() {
        let request = b"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n";
        let mut request_buffer = [0u8; 256];
        request_buffer[..request.len()].copy_from_slice(request);
        let parser = HttpRequestParser::from_buffer(&request_buffer);

        let length = compute_request_length(&request_buffer, parser).unwrap();
        assert_eq!(length, request.len());
    }
}
