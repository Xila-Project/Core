use std::os::raw::c_void;

use crate::Unique_file_identifier_type;

use super::{littlefs, Callbacks};

#[derive(Debug, Clone)]
pub struct Configuration_type {
    Context: Option<Unique_file_identifier_type>,
    Read_size: usize,
    Program_size: usize,
    Block_size: usize,
    Block_count: Option<usize>,
    Block_cycles: Option<u16>,
    Cache_size: Option<usize>,
    Maximum_file_size: Option<usize>,
    Maximum_attributes_size: Option<usize>,
    Maximum_name_size: Option<usize>,
    Read_buffer: Option<*mut c_void>,
    Write_buffer: Option<*mut c_void>,
    Lookahead_buffer: Option<*mut c_void>,
}

impl Configuration_type {
    pub const fn Set_read_size(mut self, Read_size: usize) -> Self {
        self.Read_size = Read_size;
        self
    }

    pub const fn Set_program_size(mut self, Program_size: usize) -> Self {
        self.Program_size = Program_size;
        self
    }

    pub const fn Set_block_size(mut self, Block_size: usize) -> Self {
        self.Block_size = Block_size;
        self
    }

    pub const fn Set_block_count(mut self, Block_count: usize) -> Self {
        self.Block_count = Some(Block_count);
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

    pub(crate) const fn Set_cache_size(mut self, Cache_size: usize) -> Self {
        self.Cache_size = Some(Cache_size);
        self
    }

    pub(crate) const fn Set_context(mut self, Context: Unique_file_identifier_type) -> Self {
        self.Context = Some(Context);
        self
    }

    pub(crate) fn Set_buffers(
        mut self,
        Read_buffer: *mut u8,
        Write_buffer: *mut u8,
        Lookahead_buffer: *mut u8,
    ) -> Self {
        self.Read_buffer = Some(Read_buffer as *mut _ as *mut c_void);
        self.Write_buffer = Some(Write_buffer as *mut _ as *mut c_void);
        self.Lookahead_buffer = Some(Lookahead_buffer as *mut _ as *mut c_void);
        self
    }
}

impl Default for Configuration_type {
    fn default() -> Self {
        Self {
            Read_size: 16,
            Program_size: 16,
            Block_size: 512,
            Block_count: None,
            Block_cycles: None,
            Cache_size: None,
            Maximum_file_size: None,
            Maximum_attributes_size: None,
            Maximum_name_size: None,
            Context: None,
            Read_buffer: None,
            Write_buffer: None,
            Lookahead_buffer: None,
        }
    }
}

impl TryFrom<Configuration_type> for littlefs::lfs_config {
    type Error = ();

    fn try_from(Configuration: Configuration_type) -> Result<Self, Self::Error> {
        Ok(littlefs::lfs_config {
            context: unsafe { core::mem::transmute(Configuration.Context.ok_or(())?) },
            read: Some(Callbacks::Read_callback),
            prog: Some(Callbacks::Programm_callback),
            erase: Some(Callbacks::Erase_callback),
            sync: Some(Callbacks::Flush_callback),
            read_size: Configuration.Read_size as u32,
            prog_size: Configuration.Program_size as u32,
            block_size: Configuration.Block_size as u32,
            block_count: Configuration.Block_count.unwrap_or(0) as u32,
            block_cycles: match Configuration.Block_cycles {
                Some(Block_cycles) => Block_cycles as i32,
                None => -1,
            },
            cache_size: Configuration.Cache_size.ok_or(())? as u32,
            lookahead_size: Configuration.Cache_size.ok_or(())? as u32,
            read_buffer: Configuration.Read_buffer.ok_or(())?,
            prog_buffer: Configuration.Write_buffer.ok_or(())?,
            lookahead_buffer: Configuration.Lookahead_buffer.ok_or(())?,
            name_max: Configuration.Maximum_name_size.unwrap_or(0) as u32, // Default value : 255 (LFS_NAME_MAX)
            file_max: Configuration.Maximum_file_size.unwrap_or(0) as u32, // Default value : 2,147,483,647 (2 GiB) (LFS_FILE_MAX)
            attr_max: Configuration.Maximum_attributes_size.unwrap_or(0) as u32, // Default value : 1022 (LFS_ATTR_MAX)
        })
    }
}
