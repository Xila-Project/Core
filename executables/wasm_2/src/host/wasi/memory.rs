use wasmi::Caller;

use crate::host::store::GlobalStore;

pub struct WasmMemory<'a> {
    data: &'a mut [u8],
}

impl<'a> WasmMemory<'a> {
    pub fn from_caller(caller: &'a mut Caller<'a, GlobalStore>) -> Result<Self, wasmi::Error> {
        let memory = caller
            .get_export("memory")
            .and_then(|export| export.into_memory())
            .ok_or_else(|| wasmi::Error::new("missing memory export"))?;

        Ok(WasmMemory {
            data: memory.data_mut(caller),
        })
    }

    pub fn read<T: Copy>(&self, offset: usize) -> Result<T, wasmi::Error> {
        let size = size_of::<T>();
        let bytes: &[u8] = &self.data[offset..offset + size];
        Ok(unsafe { *(bytes.as_ptr() as *const T) })
    }

    pub fn write<T: AsRef<[u8]>>(&mut self, offset: usize, value: T) -> Result<(), wasmi::Error> {
        let bytes = value.as_ref();
        self.data[offset..offset + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    pub fn read_bytes(&self, offset: usize, len: usize) -> Result<&[u8], wasmi::Error> {
        Ok(&self.data[offset..offset + len])
    }

    pub fn write_bytes(&mut self, offset: usize, data: &[u8]) -> Result<(), wasmi::Error> {
        self.data[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }
}

pub fn get_memory(caller: &Caller<GlobalStore>) -> Result<wasmi::Memory, wasmi::Error> {
    caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or_else(|| wasmi::Error::new("missing memory"))
}

pub unsafe fn read_memory<T>(data: &[u8], offset: usize) -> Option<T>
where
    T: Copy,
{
    let size = core::mem::size_of::<T>();
    if offset + size > data.len() {
        return None;
    }
    let bytes: &[u8] = &data[offset..offset + size];
    Some(unsafe { *(bytes.as_ptr() as *const T) })
}

pub unsafe fn write_memory<T>(data: &mut [u8], offset: usize, value: T) -> Option<()>
where
    T: Copy,
{
    let size = core::mem::size_of::<T>();
    if offset + size > data.len() {
        return None;
    }
    let bytes: &[u8] =
        unsafe { core::slice::from_raw_parts((&value as *const T) as *const u8, size) };
    data[offset..offset + size].copy_from_slice(bytes);
    Some(())
}

pub fn read_i32(data: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
}

pub fn write_i32(data: &mut [u8], offset: usize, value: i32) {
    data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
}

pub fn write_u64(data: &mut [u8], offset: usize, value: u64) {
    data[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

pub fn read_u64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap())
}
