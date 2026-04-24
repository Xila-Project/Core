use file_system::{Error, Result};
use shared::{HttpRequestParser, HttpResponseBuilder, HttpResponseParser};

pub fn map_network_error(error: network::Error) -> Error {
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

pub fn split_host_port(host_value: &str, default_port: u16) -> (&str, u16) {
    if let Some(stripped) = host_value.strip_prefix('[') {
        if let Some(end) = stripped.find(']') {
            let host = &stripped[..end];
            let remainder = &stripped[end + 1..];
            if let Some(port_string) = remainder.strip_prefix(':') {
                if let Ok(port) = port_string.parse::<u16>() {
                    return (host, port);
                }
            }
            return (host, default_port);
        }
    }

    if let Some((host, port_string)) = host_value.rsplit_once(':') {
        if !host.contains(':') {
            if let Ok(port) = port_string.parse::<u16>() {
                return (host, port);
            }
        }
    }

    (host_value, default_port)
}

pub fn compute_request_length(buffer: &[u8], parser: HttpRequestParser<'_>) -> Result<usize> {
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

pub fn build_serialized_response_headers(raw_headers: &[u8], output: &mut [u8]) -> Result<usize> {
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
