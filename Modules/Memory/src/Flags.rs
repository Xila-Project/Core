use std::fmt::Debug;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Flags_type(u8);

impl Debug for Flags_type {
    fn fmt(&self, Formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Formatter
            .debug_struct("Flags_type")
            .field("Anonymous", &self.Get_anonymous())
            .field("Fixed", &self.Get_fixed())
            .field("Private", &self.Get_private())
            .field("Address_32_bits", &self.Get_address_32_bits())
            .finish()
    }
}

impl Flags_type {
    pub const Anonymous_bit: u8 = 1 << 0;
    pub const Fixed_bit: u8 = 1 << 1;
    pub const Private_bit: u8 = 1 << 2;
    pub const Address_32_bits: u8 = 1 << 3;

    pub fn New(Anonymous: bool, Fixed: bool) -> Self {
        let mut Flags = Self(0);
        Flags.Set_anonymous(Anonymous).Set_fixed(Fixed);
        Flags
    }

    pub fn New_none() -> Self {
        Self(0)
    }

    fn Set_bits(&mut self, Bits: u8, Value: bool) -> &mut Self {
        if Value {
            self.0 |= Bits;
        } else {
            self.0 &= !Bits;
        }
        self
    }

    const fn Get_bits(&self, Bits: u8) -> bool {
        self.0 & Bits != 0
    }

    /// Set the anonymous flag.
    ///
    pub fn Set_anonymous(&mut self, Value: bool) -> &mut Self {
        self.Set_bits(Self::Anonymous_bit, Value)
    }

    pub fn Set_fixed(&mut self, Value: bool) -> &mut Self {
        self.Set_bits(Self::Fixed_bit, Value)
    }

    pub fn Set_private(&mut self, Value: bool) -> &mut Self {
        self.Set_bits(Self::Private_bit, Value)
    }

    pub fn Set_address_32_bits(&mut self, Value: bool) -> &mut Self {
        self.Set_bits(Self::Address_32_bits, Value)
    }

    pub const fn Get_anonymous(&self) -> bool {
        self.Get_bits(Self::Anonymous_bit)
    }

    pub const fn Get_fixed(&self) -> bool {
        self.Get_bits(Self::Fixed_bit)
    }

    pub const fn Get_private(&self) -> bool {
        self.Get_bits(Self::Private_bit)
    }

    pub const fn Get_address_32_bits(&self) -> bool {
        self.Get_bits(Self::Address_32_bits)
    }
}

impl From<Flags_type> for u8 {
    fn from(Flags: Flags_type) -> Self {
        Flags.0
    }
}

impl From<u8> for Flags_type {
    fn from(Flags: u8) -> Self {
        Self(Flags)
    }
}
