use core::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IPv4([u8; 4]);

impl IPv4 {
    pub const LOCALHOST: Self = Self([127, 0, 0, 1]);

    pub const fn new(value: [u8; 4]) -> Self {
        Self(value)
    }

    pub const fn into_inner(self) -> [u8; 4] {
        self.0
    }

    pub const fn from_inner(value: [u8; 4]) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for IPv4 {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut result = [0; 4];
        let mut index = 0;

        for part in value.split('.') {
            if index >= 4 {
                return Err(());
            }
            let part = part.parse::<u8>().map_err(|_| ())?;
            result[index] = part;
            index += 1;
        }
        if index != 4 {
            return Err(());
        }

        Ok(Self::new(result))
    }
}

impl Display for IPv4 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IPv6([u16; 8]);

impl IPv6 {
    pub const fn new(value: [u16; 8]) -> Self {
        Self(value)
    }

    pub const fn into_inner(self) -> [u16; 8] {
        self.0
    }

    pub const fn from_inner(value: [u16; 8]) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for IPv6 {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut result = [0; 8];
        let mut index = 0;

        for part in value.split(':') {
            if index >= result.len() {
                return Err(());
            }

            let part = u16::from_str_radix(part, 16).map_err(|_| ())?;
            result[index] = part;
            index += 1;
        }
        if index != result.len() {
            return Err(());
        }

        Ok(Self::new(result))
    }
}

impl Display for IPv6 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6], self.0[7]
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IP {
    IPv4(IPv4),
    IPv6(IPv6),
}

impl Display for IP {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            IP::IPv4(value) => write!(f, "{value}"),
            IP::IPv6(value) => write!(f, "{value}"),
        }
    }
}

impl From<IPv4> for IP {
    fn from(value: IPv4) -> Self {
        Self::IPv4(value)
    }
}

impl From<IPv6> for IP {
    fn from(value: IPv6) -> Self {
        Self::IPv6(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_try_from() {
        let ip = IPv4::try_from("0.0.0.0").unwrap();

        assert_eq!(ip.0, [0, 0, 0, 0]);

        IPv4::try_from("1.2b.3.4").unwrap_err();

        IPv4::try_from("1.2.3.4.5").unwrap_err();

        IPv4::try_from("1.2.3").unwrap_err();

        let ip = IPv4::try_from("4.3.2.1").unwrap();

        assert_eq!(ip.0, [4, 3, 2, 1]);
    }

    #[test]
    fn test_ipv6_try_from() {
        let ip = IPv6::try_from("0:0:0:0:0:0:0:0").unwrap();

        assert_eq!(ip.0, [0; 8]);

        IPv6::try_from("0:0:0:0:0:0:0:0:0").unwrap_err();

        IPv6::try_from("0:0:0:0:0:0:0").unwrap_err();

        let ip = IPv6::try_from("1234:5678:9abc:def0:1234:5678:9abc:def0").unwrap();

        assert_eq!(
            ip.0,
            [0x1234, 0x5678, 0x9abc, 0xdef0, 0x1234, 0x5678, 0x9abc, 0xdef0]
        );
    }
}
