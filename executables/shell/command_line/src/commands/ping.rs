use core::fmt::Write;
use executable_macros::GetArgs;
use xila::{
    internationalization::translate,
    network::{self, DnsQueryKind, Duration, IcmpEndpoint},
};

use crate::{Error, Shell};

const ICMP_IDENTIFIER: u16 = 0x22b;

#[derive(GetArgs)]
struct PingArguments<'a> {
    target: &'a str,
    #[arg(short = 'c', long = "count", default = 4)]
    count: u16,
    #[arg(short = 't', long = "timeout", default = 5)]
    timeout: u64,
    #[arg(short = 's', long = "size", default = 56)]
    size: usize,
}

impl Shell {
    pub async fn ping<'a, I>(
        &mut self,
        options: &mut getargs::Options<&'a str, I>,
    ) -> crate::Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let PingArguments {
            target,
            count,
            timeout: timeout_seconds,
            size: payload_size,
        } = PingArguments::parse(options)?;

        let network = network::get_instance();

        let dns_socket = network
            .new_dns_socket(None)
            .await
            .map_err(Error::FailedToCreateSocket)?;

        let resolved_target = dns_socket
            .resolve(target, DnsQueryKind::A | DnsQueryKind::Aaaa)
            .await
            .map(|s| s.first().cloned())
            .map_err(Error::FailedToResolve)?;

        dns_socket
            .close()
            .await
            .map_err(Error::FailedToCreateSocket)?;

        let resolved_target = match resolved_target {
            Some(ip) => ip,
            None => {
                writeln!(
                    self.standard.out(),
                    translate!("Cannot resolve {}: Unknown host"),
                    target
                )?;
                return Ok(());
            }
        };

        writeln!(
            self.standard.out(),
            translate!("PING {} ({}): {} data bytes"),
            target,
            &resolved_target,
            56
        )?;

        let socket = network
            .new_icmp_socket(256, 256, 1, 1, None)
            .await
            .map_err(Error::FailedToCreateSocket)?;

        socket
            .bind(IcmpEndpoint::Identifier(ICMP_IDENTIFIER))
            .await
            .map_err(Error::FailedToCreateSocket)?;

        for i in 0..count {
            match socket
                .ping(
                    &resolved_target,
                    i,
                    ICMP_IDENTIFIER,
                    Duration::from_seconds(timeout_seconds),
                    payload_size,
                )
                .await
            {
                Ok(duration) => {
                    writeln!(
                        self.standard.out(),
                        translate!("{} bytes from {}: icmp_seq={} time={:.2} ms"),
                        payload_size,
                        resolved_target,
                        i,
                        duration.as_milliseconds()
                    )?;
                }
                Err(network::Error::TimedOut) => {
                    writeln!(
                        self.standard.out(),
                        translate!("Request timeout for icmp_seq {}"),
                        i
                    )?;
                }
                Err(e) => {
                    writeln!(self.standard.out(), translate!("Error: {}"), e)?;
                }
            }
        }

        Ok(())
    }
}
