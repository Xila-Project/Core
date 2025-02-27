use core::fmt::Debug;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Protection_type(u8);

impl Debug for Protection_type {
    fn fmt(&self, Formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Formatter
            .debug_struct("Protection_type")
            .field("Read", &self.Get_read())
            .field("Write", &self.Get_write())
            .field("Execute", &self.Get_execute())
            .finish()
    }
}

impl Protection_type {
    pub const Read_bit: u8 = 1 << 0;
    pub const Write_bit: u8 = 1 << 1;
    pub const Execute_bit: u8 = 1 << 2;

    pub fn New(Read: bool, Write: bool, Execute: bool) -> Self {
        let mut Protection = Self(0);
        *Protection
            .Set_read(Read)
            .Set_write(Write)
            .Set_execute(Execute)
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

    pub fn Set_read(&mut self, Value: bool) -> &mut Self {
        self.Set_bits(Self::Read_bit, Value)
    }

    pub fn Set_write(&mut self, Value: bool) -> &mut Self {
        self.Set_bits(Self::Write_bit, Value)
    }

    pub fn Set_execute(&mut self, Value: bool) -> &mut Self {
        self.Set_bits(Self::Execute_bit, Value)
    }

    pub const fn Get_read(&self) -> bool {
        self.Get_bits(Self::Read_bit)
    }

    pub const fn Get_write(&self) -> bool {
        self.Get_bits(Self::Write_bit)
    }

    pub const fn Get_execute(&self) -> bool {
        self.Get_bits(Self::Execute_bit)
    }

    pub const fn As_u8(&self) -> u8 {
        self.0
    }

    pub const fn From_u8(Value: u8) -> Self {
        Self(Value)
    }
}

impl From<Protection_type> for u8 {
    fn from(Protection: Protection_type) -> Self {
        Protection.0
    }
}

impl From<u8> for Protection_type {
    fn from(Protection: u8) -> Self {
        Self(Protection)
    }
}

trait Protection_trait {
    fn Set_protection(&self, Address: *mut u8, Size: usize, Protection: Protection_type) -> bool;
}
