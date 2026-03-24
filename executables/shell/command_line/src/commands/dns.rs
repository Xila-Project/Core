use crate::{Error, Result, Shell};
use core::fmt::Write;
use executable_macros::GetArgs;
use getargs::Options;
use xila::{
    internationalization::translate,
    network::{self, DnsQueryKind, DnsSocket},
};

#[derive(GetArgs)]
struct DnsResolveArguments<'a> {
    domain: &'a str,
    #[arg(flag, short = 'a', long = "a")]
    a_enabled: bool,
    #[arg(flag, short = 'A', long = "aaaa")]
    aaaa_enabled: bool,
    #[arg(flag, short = 'c', long = "cname")]
    cname_enabled: bool,
    #[arg(flag, short = 'n', long = "ns")]
    ns_enabled: bool,
    #[arg(flag, short = 's', long = "soa")]
    soa_enabled: bool,
}

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
        let DnsResolveArguments {
            domain,
            a_enabled,
            aaaa_enabled,
            cname_enabled,
            ns_enabled,
            soa_enabled,
        } = DnsResolveArguments::parse(options)?;

        let default = !a_enabled && !aaaa_enabled && !cname_enabled && !ns_enabled && !soa_enabled;

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
