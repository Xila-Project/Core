pub type Xila_memory_protection_type = u8;

#[no_mangle]
pub static Xila_memory_protection_read: u8 = Memory::Protection_type::Read_bit;
#[no_mangle]
pub static Xila_memory_protection_write: u8 = Memory::Protection_type::Write_bit;
#[no_mangle]
pub static Xila_memory_protection_execute: u8 = Memory::Protection_type::Execute_bit;

pub type Xila_memory_flags_type = u8;

#[no_mangle]
pub static Xila_memory_flag_anonymous: u8 = Memory::Flags_type::Anonymous_bit;
#[no_mangle]
pub static Xila_memory_flag_fixed: u8 = Memory::Flags_type::Fixed_bit;
#[no_mangle]
pub static Xila_memory_flag_private: u8 = Memory::Flags_type::Private_bit;
#[no_mangle]
pub static Xila_memory_flag_address_32_bits: u8 = Memory::Flags_type::Address_32_bits;
