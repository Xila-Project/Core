use core::fmt::Write;
use getargs::Arg;
use xila::{
    internationalization::translate,
    network::{self, DnsQueryKind, Duration, IcmpEndpoint},
};

use crate::{Error, Shell};

const ICMP_IDENTIFIER: u16 = 0x22b;

impl Shell {
    pub async fn ping<'a, I>(
        &mut self,
        options: &mut getargs::Options<&'a str, I>,
    ) -> crate::Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut count = 4;
        let mut timeout_seconds = 5;
        let mut target: &'a str = "";
        let mut payload_size = 56;

        while let Some(argument) = options.next_arg()? {
            match argument {
                Arg::Short('c') | Arg::Long("count") => {
                    let value = options
                        .next_positional()
                        .ok_or(crate::Error::MissingPositionalArgument("count"))?;
                    count = value.parse().map_err(|_| crate::Error::InvalidOption)?;
                }
                Arg::Short('t') | Arg::Long("timeout") => {
                    let value = options
                        .next_positional()
                        .ok_or(crate::Error::MissingPositionalArgument("timeout"))?;
                    timeout_seconds = value.parse().map_err(|_| crate::Error::InvalidOption)?;
                }
                Arg::Short('s') | Arg::Long("size") => {
                    let value = options
                        .next_positional()
                        .ok_or(crate::Error::MissingPositionalArgument("size"))?;
                    payload_size = value.parse().map_err(|_| crate::Error::InvalidOption)?;
                }
                Arg::Positional(p) => {
                    if !target.is_empty() {
                        return Err(crate::Error::InvalidNumberOfArguments);
                    }
                    target = p;
                }
                _ => {
                    return Err(crate::Error::InvalidOption);
                }
            }
        }

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
