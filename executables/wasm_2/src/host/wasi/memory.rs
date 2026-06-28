use core::cell::RefMut;

use wasmi::Caller;

use crate::GlobalStore;

/// Wrapper to access WASM linear memory safely
pub struct WasmMemory<'a> {
    data: RefMut<'a, [u8]>,
}

impl<'a> WasmMemory<'a> {
    /// Extract memory from Wasmi Caller
    pub fn from_caller(caller: &mut Caller<'a, GlobalStore>) -> Result<Self, wasmi::Error> {
        let memory = caller
            .get_export("memory")
            .and_then(|export| export.into_memory())
            .ok_or_else(|| wasmi::Error::new("missing memory export"))?;

        Ok(WasmMemory {
            data: memory.data_mut(caller),
        })
    }

    /// Read a value from memory
    pub fn read<T: Copy>(&self, offset: usize) -> Result<T, wasmi::Error> {
        let size = size_of::<T>();
        let bytes: &[u8] = &self.data[offset..offset + size];
        Ok(unsafe { *(bytes.as_ptr() as *const T) })
    }

    /// Write a value to memory
    pub fn write<T: AsRef<[u8]>>(&mut self, offset: usize, value: T) -> Result<(), wasmi::Error> {
        let bytes = value.as_ref();
        self.data[offset..offset + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    /// Read bytes
    pub fn read_bytes(&self, offset: usize, len: usize) -> Result<&[u8], wasmi::Error> {
        Ok(&self.data[offset..offset + len])
    }

    /// Write bytes
    pub fn write_bytes(&mut self, offset: usize, data: &[u8]) -> Result<(), wasmi::Error> {
        self.data[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }
}
