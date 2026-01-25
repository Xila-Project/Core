use core::{
    future::poll_fn,
    task::{Context, Poll},
};

use alloc::{vec, vec::Vec};
use embassy_futures::block_on;
use smoltcp::{socket::dns, wire::DnsQueryType};

use crate::{DnsQueryKind, IpAddress, Result, SocketContext};

pub struct DnsSocket {
    context: SocketContext,
}

impl DnsSocket {
    pub fn new(context: SocketContext) -> Self {
        Self { context }
    }

    fn poll_with_mutable<F, R>(&self, context: &mut Context<'_>, f: F) -> Poll<R>
    where
        F: FnOnce(&mut dns::Socket<'static>, &mut Context<'_>) -> Poll<R>,
    {
        self.context.poll_with_mutable(context, f)
    }

    pub async fn update_servers(&self) -> Result<()> {
        self.context
            .stack
            .with_mutable(|s| {
                let dns_servers = s.get_dns_servers().to_vec();
                let socket = s.sockets.get_mut::<dns::Socket>(self.context.handle);
                socket.update_servers(&dns_servers);
            })
            .await;

        Ok(())
    }

    pub async fn resolve_for_kind(&self, host: &str, kind: DnsQueryType) -> Result<Vec<IpAddress>> {
        if let Ok(host) = IpAddress::try_from(host) {
            return Ok(vec![host]);
        }

        let query = self
            .context
            .stack
            .with_mutable(|s| {
                let socket = s.sockets.get_mut::<dns::Socket>(self.context.handle);

                socket.start_query(s.interface.context(), host, kind)
            })
            .await?;

        self.context.stack.wake_up();

        poll_fn(|cx| {
            self.poll_with_mutable(cx, |socket, cx| match socket.get_query_result(query) {
                Err(dns::GetQueryResultError::Pending) => {
                    socket.register_query_waker(query, cx.waker());
                    Poll::Pending
                }
                Err(e) => Poll::Ready(Err(e.into())),
                Ok(ip_addresses) => {
                    let ip_addresses = ip_addresses
                        .into_iter()
                        .map(|a| IpAddress::from_smoltcp(&a))
                        .collect();

                    Poll::Ready(Ok(ip_addresses))
                }
            })
        })
        .await
    }

    pub async fn resolve(&self, host: &str, kind: DnsQueryKind) -> Result<Vec<IpAddress>> {
        let mut results = Vec::new();

        if kind.contains(DnsQueryKind::A) {
            let mut a_results = self.resolve_for_kind(host, DnsQueryType::A).await?;
            results.append(&mut a_results);
        }

        if kind.contains(DnsQueryKind::Aaaa) {
            let mut aaaa_results = self.resolve_for_kind(host, DnsQueryType::Aaaa).await?;
            results.append(&mut aaaa_results);
        }

        if kind.contains(DnsQueryKind::Cname) {
            let mut cname_results = self.resolve_for_kind(host, DnsQueryType::Cname).await?;
            results.append(&mut cname_results);
        }

        if kind.contains(DnsQueryKind::Ns) {
            let mut ns_results = self.resolve_for_kind(host, DnsQueryType::Ns).await?;
            results.append(&mut ns_results);
        }

        if kind.contains(DnsQueryKind::Soa) {
            let mut soa_results = self.resolve_for_kind(host, DnsQueryType::Soa).await?;
            results.append(&mut soa_results);
        }

        Ok(results)
    }

    pub async fn close(mut self) -> Result<()> {
        if self.context.closed {
            return Ok(());
        }

        self.context
            .stack
            .with_mutable(|s| {
                let _ = s.remove_socket(self.context.handle);
            })
            .await;

        self.context.closed = true;

        Ok(())
    }
}

impl Drop for DnsSocket {
    fn drop(&mut self) {
        if self.context.closed {
            return;
        }

        log::warning!("DNS socket dropped without being closed. Forcing closure...");

        block_on(self.context.stack.with_mutable(|s| {
            let _ = s.remove_socket(self.context.handle);
        }));
    }
}
