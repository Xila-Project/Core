use crate::{Error, Result, Shell, commands::check_no_more_arguments};
use core::fmt::Write;
use getargs::{Arg, Options};
use xila::{
    internationalization::translate,
    network::{self, DnsQueryKind, DnsSocket},
};

impl Shell {
    fn format_kind(kind: DnsQueryKind) -> &'static str {
        match kind {
            DnsQueryKind::A => "A",
            DnsQueryKind::Aaaa => "AAAA",
            DnsQueryKind::Cname => "CNAME",
            DnsQueryKind::Ns => "NS",
            DnsQueryKind::Soa => "SOA",
            _ => "UNKNOWN",
        }
    }

    async fn resolve(
        &mut self,
        socket: &DnsSocket,
        domain: &str,
        kind: DnsQueryKind,
    ) -> Result<()> {
        match socket.resolve(domain, kind).await {
            Ok(ip) => {
                writeln!(
                    self.standard.out(),
                    translate!("{} record(s) for domain '{}':"),
                    Self::format_kind(kind),
                    domain
                )?;
                for address in &ip {
                    writeln!(self.standard.out(), " - {}", address)?;
                }
            }
            Err(network::Error::Failed) => {
                writeln!(
                    self.standard.out(),
                    translate!("No {} records found for domain '{}'"),
                    Self::format_kind(kind),
                    domain
                )?;
            }
            Err(e) => {
                write!(
                    self.standard.out(),
                    translate!("Failed to resolve domain '{}' for {} records: {}"),
                    domain,
                    Self::format_kind(kind),
                    e
                )?;
            }
        }
        Ok(())
    }

    pub async fn dns_resolve<'a, I>(&mut self, options: &mut Options<&'a str, I>) -> Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut domain = "";
        let mut a_enabled = false;
        let mut aaaa_enabled = false;
        let mut cname_enabled = false;
        let mut ns_enabled = false;
        let mut soa_enabled = false;
        let mut default = true;

        while let Some(argument) = options.next_arg()? {
            if let Arg::Long(_) | Arg::Short(_) = &argument {
                default = false;
            }

            match argument {
                Arg::Short('a') | Arg::Long("a") => {
                    a_enabled = true;
                }
                Arg::Short('A') | Arg::Long("aaaa") => {
                    aaaa_enabled = true;
                }
                Arg::Short('c') | Arg::Long("cname") => {
                    cname_enabled = true;
                }
                Arg::Short('n') | Arg::Long("ns") => {
                    ns_enabled = true;
                }
                Arg::Short('s') | Arg::Long("soa") => {
                    soa_enabled = true;
                }
                Arg::Positional(p) => {
                    if !domain.is_empty() {
                        return Err(crate::Error::InvalidNumberOfArguments);
                    }
                    domain = p;
                }
                _ => {
                    return Err(crate::Error::InvalidOption);
                }
            }
        }

        check_no_more_arguments(options)?;

        let socket = network::get_instance()
            .new_dns_socket(None)
            .await
            .map_err(Error::FailedToCreateSocket)?;

        if a_enabled || default {
            self.resolve(&socket, domain, DnsQueryKind::A).await?;
        }
        if aaaa_enabled || default {
            self.resolve(&socket, domain, DnsQueryKind::Aaaa).await?;
        }
        if cname_enabled {
            self.resolve(&socket, domain, DnsQueryKind::Cname).await?;
        }
        if ns_enabled {
            self.resolve(&socket, domain, DnsQueryKind::Ns).await?;
        }
        if soa_enabled {
            self.resolve(&socket, domain, DnsQueryKind::Soa).await?;
        }

        socket.close().await.map_err(Error::FailedToCreateSocket)?;

        Ok(())
    }
}
