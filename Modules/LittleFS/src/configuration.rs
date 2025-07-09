use core::{ffi::c_void, mem::forget};

use alloc::{boxed::Box, vec};
use file_system::Device_type;

use super::{callbacks, littlefs};

#[derive(Debug, Clone)]
pub struct Configuration_type {
    context: Device_type,
    read_size: usize,
    program_size: usize,
    block_size: usize,
    block_count: usize,
    block_cycles: Option<u16>,
    maximum_file_size: Option<usize>,
    maximum_attributes_size: Option<usize>,
    maximum_name_size: Option<usize>,
    cache_size: usize,
    look_ahead_size: usize,
    compact_threshold: Option<usize>,
    metadata_maxium: Option<usize>,
    inline_maximum: Option<usize>,
}

impl Configuration_type {
    pub const DEFAULT_RAW: littlefs::lfs_config = littlefs::lfs_config {
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
        compact_thresh: 0,
        metadata_max: 0,
        inline_max: 0,
    };

    pub fn New(
        device: Device_type,
        block_size: usize,
        total_size: usize,
        cache_size: usize,
        look_ahead_size: usize,
    ) -> Option<Self> {
        if (total_size % block_size) != 0 {
            return None;
        }

        if !(block_size % cache_size) == 0 {
            return None;
        }

        if (look_ahead_size % 8) != 0 {
            return None;
        }

        let Block_count = total_size / block_size;

        Some(Self {
            context: device,
            read_size: 16,
            program_size: 16,
            block_size,
            block_count: Block_count,
            block_cycles: None,
            maximum_file_size: None,
            maximum_attributes_size: None,
            maximum_name_size: None,
            cache_size,
            look_ahead_size,
            compact_threshold: None,
            metadata_maxium: None,
            inline_maximum: None,
        })
    }

    pub const fn Set_read_size(mut self, Read_size: usize) -> Self {
        self.read_size = Read_size;
        self
    }

    pub const fn Set_program_size(mut self, Program_size: usize) -> Self {
        self.program_size = Program_size;
        self
    }

    pub const fn Set_block_cycles(mut self, Block_cycles: u16) -> Self {
        self.block_cycles = Some(Block_cycles);
        self
    }

    pub const fn Set_maximum_file_size(mut self, Maximum_file_size: usize) -> Self {
        self.maximum_file_size = Some(Maximum_file_size);
        self
    }

    pub const fn Set_maximum_attributes_size(mut self, Maximum_attributes_size: usize) -> Self {
        self.maximum_attributes_size = Some(Maximum_attributes_size);
        self
    }

    pub const fn Set_maximum_name_size(mut self, Maximum_name_size: usize) -> Self {
        self.maximum_name_size = Some(Maximum_name_size);
        self
    }

    pub const fn Set_cache_size(mut self, Cache_size: usize) -> Self {
        self.cache_size = Cache_size;
        self
    }

    pub const fn Set_look_ahead_size(mut self, Look_ahead_size: usize) -> Self {
        self.look_ahead_size = Look_ahead_size;
        self
    }

    pub const fn Set_compact_threshold(mut self, Compact_threshold: usize) -> Self {
        self.compact_threshold = Some(Compact_threshold);
        self
    }

    pub const fn Set_metadata_maximum(mut self, Metadata_maxium: usize) -> Self {
        self.metadata_maxium = Some(Metadata_maxium);
        self
    }

    pub const fn Set_inline_maximum(mut self, Inline_maximum: usize) -> Self {
        self.inline_maximum = Some(Inline_maximum);
        self
    }
}

impl TryFrom<Configuration_type> for littlefs::lfs_config {
    type Error = ();

    fn try_from(Configuration: Configuration_type) -> Result<Self, Self::Error> {
        let mut read_buffer = vec![0_u8; Configuration.cache_size];
        let mut write_buffer = read_buffer.clone();
        let mut look_ahead_buffer = vec![0_u8; Configuration.look_ahead_size];

        let LFS_Configuration = littlefs::lfs_config {
            context: Box::into_raw(Box::new(Configuration.context)) as *mut c_void,
            read: Some(callbacks::Read_callback),
            prog: Some(callbacks::Programm_callback),
            erase: Some(callbacks::Erase_callback),
            sync: Some(callbacks::Flush_callback),
            read_size: Configuration.read_size as u32,
            prog_size: Configuration.program_size as u32,
            block_size: Configuration.block_size as u32,
            block_count: Configuration.block_count as u32,
            block_cycles: match Configuration.block_cycles {
                Some(block_cycles) => block_cycles as i32,
                None => -1,
            },
            cache_size: Configuration.cache_size as u32,
            lookahead_size: Configuration.look_ahead_size as u32,
            read_buffer: read_buffer.as_mut_ptr() as *mut c_void,
            prog_buffer: write_buffer.as_mut_ptr() as *mut c_void,
            lookahead_buffer: look_ahead_buffer.as_mut_ptr() as *mut c_void,
            name_max: Configuration.maximum_name_size.unwrap_or(0) as u32, // Default value : 255 (LFS_NAME_MAX)
            file_max: Configuration.maximum_file_size.unwrap_or(0) as u32, // Default value : 2,147,483,647 (2 GiB) (LFS_FILE_MAX)
            attr_max: Configuration.maximum_attributes_size.unwrap_or(0) as u32, // Default value : 1022 (LFS_ATTR_MAX)
            compact_thresh: Configuration.compact_threshold.unwrap_or(0) as u32, // Default value : 0 (LFS_COMPACT_THRESH)
            metadata_max: Configuration.metadata_maxium.unwrap_or(0) as u32, // Default value : 0 (LFS_METADATA_MAX)
            inline_max: Configuration.inline_maximum.unwrap_or(0) as u32, // Default value : 0 (LFS_INLINE_MAX)
        };

        forget(read_buffer);
        forget(write_buffer);
        forget(look_ahead_buffer);

        Ok(LFS_Configuration)
    }
}
