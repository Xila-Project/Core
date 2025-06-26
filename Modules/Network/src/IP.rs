use core::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IPv4_type([u8; 4]);

impl IPv4_type {
    pub const Localhost: Self = Self([127, 0, 0, 1]);

    pub const fn New(value: [u8; 4]) -> Self {
        Self(value)
    }

    pub const fn Into_inner(self) -> [u8; 4] {
        self.0
    }

    pub const fn From_inner(value: [u8; 4]) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for IPv4_type {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut Result = [0; 4];
        let mut Index = 0;

        for Part in value.split('.') {
            if Index >= 4 {
                return Err(());
            }
            let Part = Part.parse::<u8>().map_err(|_| ())?;
            Result[Index] = Part;
            Index += 1;
        }
        if Index != 4 {
            return Err(());
        }

        Ok(Self::New(Result))
    }
}

impl Display for IPv4_type {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IPv6_type([u16; 8]);

impl IPv6_type {
    pub const fn New(value: [u16; 8]) -> Self {
        Self(value)
    }

    pub const fn Into_inner(self) -> [u16; 8] {
        self.0
    }

    pub const fn From_inner(value: [u16; 8]) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for IPv6_type {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut Result = [0; 8];
        let mut Index = 0;

        for Part in value.split(':') {
            if Index >= Result.len() {
                return Err(());
            }

            let Part = u16::from_str_radix(Part, 16).map_err(|_| ())?;
            Result[Index] = Part;
            Index += 1;
        }
        if Index != Result.len() {
            return Err(());
        }

        Ok(Self::New(Result))
    }
}

impl Display for IPv6_type {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6], self.0[7]
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IP_type {
    IPv4(IPv4_type),
    IPv6(IPv6_type),
}

impl Display for IP_type {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            IP_type::IPv4(Value) => write!(f, "{}", Value),
            IP_type::IPv6(Value) => write!(f, "{}", Value),
        }
    }
}

impl From<IPv4_type> for IP_type {
    fn from(value: IPv4_type) -> Self {
        Self::IPv4(value)
    }
}

impl From<IPv6_type> for IP_type {
    fn from(value: IPv6_type) -> Self {
        Self::IPv6(value)
    }
}

#[cfg(test)]
mod Tests {
    use super::*;

    #[test]
    fn Test_ipv4_try_from() {
        let IP = IPv4_type::try_from("0.0.0.0").unwrap();

        assert_eq!(IP.0, [0, 0, 0, 0]);

        IPv4_type::try_from("1.2b.3.4").unwrap_err();

        IPv4_type::try_from("1.2.3.4.5").unwrap_err();

        IPv4_type::try_from("1.2.3").unwrap_err();

        let IP = IPv4_type::try_from("4.3.2.1").unwrap();

        assert_eq!(IP.0, [4, 3, 2, 1]);
    }

    #[test]
    fn Test_ipv6_try_from() {
        let IP = IPv6_type::try_from("0:0:0:0:0:0:0:0").unwrap();

        assert_eq!(IP.0, [0; 8]);

        IPv6_type::try_from("0:0:0:0:0:0:0:0:0").unwrap_err();

        IPv6_type::try_from("0:0:0:0:0:0:0").unwrap_err();

        let IP = IPv6_type::try_from("1234:5678:9abc:def0:1234:5678:9abc:def0").unwrap();

        assert_eq!(
            IP.0,
            [0x1234, 0x5678, 0x9abc, 0xdef0, 0x1234, 0x5678, 0x9abc, 0xdef0]
        );
    }
}
