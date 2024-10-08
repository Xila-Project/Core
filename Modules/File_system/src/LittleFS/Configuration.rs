use std::{ffi::c_void, mem::forget};

use crate::Device::Device_type;

use super::{littlefs, Callbacks};

#[derive(Debug, Clone)]
pub struct Configuration_type {
    Context: Device_type,
    Read_size: usize,
    Program_size: usize,
    Block_size: usize,
    Block_count: usize,
    Block_cycles: Option<u16>,
    Maximum_file_size: Option<usize>,
    Maximum_attributes_size: Option<usize>,
    Maximum_name_size: Option<usize>,
    Cache_size: usize,
    Look_ahead_size: usize,
}

impl Configuration_type {
    pub const Default_raw: littlefs::lfs_config = littlefs::lfs_config {
        context: 0 as *mut c_void,
        read: None,
        prog: None,
        erase: None,
        sync: None,
        read_size: 0,
        prog_size: 0,
        block_size: 0,
        block_count: 0,
        block_cycles: 0,
        cache_size: 0,
        lookahead_size: 0,
        read_buffer: 0 as *mut c_void,
        prog_buffer: 0 as *mut c_void,
        lookahead_buffer: 0 as *mut c_void,
        name_max: 0,
        file_max: 0,
        attr_max: 0,
    };

    pub fn New(
        Device: Device_type,
        Block_size: usize,
        Total_size: usize,
        Cache_size: usize,
        Look_ahead_size: usize,
    ) -> Option<Self> {
        if (Total_size % Block_size) != 0 {
            return None;
        }

        if !(Block_size % Cache_size) == 0 {
            return None;
        }

        if (Look_ahead_size % 8) != 0 {
            return None;
        }

        let Block_count = Total_size / Block_size;

        Some(Self {
            Context: Device,
            Read_size: 16,
            Program_size: 16,
            Block_size,
            Block_count,
            Block_cycles: None,
            Maximum_file_size: None,
            Maximum_attributes_size: None,
            Maximum_name_size: None,
            Cache_size,
            Look_ahead_size,
        })
    }

    pub const fn Set_read_size(mut self, Read_size: usize) -> Self {
        self.Read_size = Read_size;
        self
    }

    pub const fn Set_program_size(mut self, Program_size: usize) -> Self {
        self.Program_size = Program_size;
        self
    }

    pub const fn Set_block_cycles(mut self, Block_cycles: u16) -> Self {
        self.Block_cycles = Some(Block_cycles);
        self
    }

    pub const fn Set_maximum_file_size(mut self, Maximum_file_size: usize) -> Self {
        self.Maximum_file_size = Some(Maximum_file_size);
        self
    }

    pub const fn Set_maximum_attributes_size(mut self, Maximum_attributes_size: usize) -> Self {
        self.Maximum_attributes_size = Some(Maximum_attributes_size);
        self
    }

    pub const fn Set_maximum_name_size(mut self, Maximum_name_size: usize) -> Self {
        self.Maximum_name_size = Some(Maximum_name_size);
        self
    }
}

impl TryFrom<Configuration_type> for littlefs::lfs_config {
    type Error = ();

    fn try_from(Configuration: Configuration_type) -> Result<Self, Self::Error> {
        let mut Read_buffer = vec![0_u8; Configuration.Cache_size];
        let mut Write_buffer = Read_buffer.clone();
        let mut Look_ahead_buffer = vec![0_u8; Configuration.Look_ahead_size];

        let LFS_Configuration = littlefs::lfs_config {
            context: Box::into_raw(Box::new(Configuration.Context)) as *mut c_void,
            read: Some(Callbacks::Read_callback),
            prog: Some(Callbacks::Programm_callback),
            erase: Some(Callbacks::Erase_callback),
            sync: Some(Callbacks::Flush_callback),
            read_size: Configuration.Read_size as u32,
            prog_size: Configuration.Program_size as u32,
            block_size: Configuration.Block_size as u32,
            block_count: Configuration.Block_count as u32,
            block_cycles: match Configuration.Block_cycles {
                Some(Block_cycles) => Block_cycles as i32,
                None => -1,
            },
            cache_size: Configuration.Cache_size as u32,
            lookahead_size: Configuration.Look_ahead_size as u32,
            read_buffer: Read_buffer.as_mut_ptr() as *mut c_void,
            prog_buffer: Write_buffer.as_mut_ptr() as *mut c_void,
            lookahead_buffer: Look_ahead_buffer.as_mut_ptr() as *mut c_void,
            name_max: Configuration.Maximum_name_size.unwrap_or(0) as u32, // Default value : 255 (LFS_NAME_MAX)
            file_max: Configuration.Maximum_file_size.unwrap_or(0) as u32, // Default value : 2,147,483,647 (2 GiB) (LFS_FILE_MAX)
            attr_max: Configuration.Maximum_attributes_size.unwrap_or(0) as u32, // Default value : 1022 (LFS_ATTR_MAX)
        };

        forget(Read_buffer);
        forget(Write_buffer);
        forget(Look_ahead_buffer);

        Ok(LFS_Configuration)
    }
}
