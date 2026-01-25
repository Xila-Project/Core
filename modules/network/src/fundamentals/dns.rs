use shared::flags;

flags! {
    pub enum DnsQueryKind: u8 {
        A,
        Aaaa,
        Cname,
        Ns,
        Soa,
    }
}
