use core::{ffi::c_void, mem::forget};

use alloc::{boxed::Box, vec};
use file_system::Device;

use super::{callbacks, littlefs};

#[derive(Debug, Clone)]
pub struct Configuration {
    context: Device,
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

impl Configuration {
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
        flags: 0,
    };

    pub fn new(
        device: Device,
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

        let block_count = total_size / block_size;

        Some(Self {
            context: device,
            read_size: 16,
            program_size: 16,
            block_size,
            block_count,
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

    pub const fn set_read_size(mut self, read_size: usize) -> Self {
        self.read_size = read_size;
        self
    }

    pub const fn set_program_size(mut self, program_size: usize) -> Self {
        self.program_size = program_size;
        self
    }

    pub const fn set_block_cycles(mut self, block_cycles: u16) -> Self {
        self.block_cycles = Some(block_cycles);
        self
    }

    pub const fn set_maximum_file_size(mut self, maximum_file_size: usize) -> Self {
        self.maximum_file_size = Some(maximum_file_size);
        self
    }

    pub const fn set_maximum_attributes_size(mut self, maximum_attributes_size: usize) -> Self {
        self.maximum_attributes_size = Some(maximum_attributes_size);
        self
    }

    pub const fn set_maximum_name_size(mut self, maximum_name_size: usize) -> Self {
        self.maximum_name_size = Some(maximum_name_size);
        self
    }

    pub const fn set_cache_size(mut self, cache_size: usize) -> Self {
        self.cache_size = cache_size;
        self
    }

    pub const fn set_look_ahead_size(mut self, look_ahead_size: usize) -> Self {
        self.look_ahead_size = look_ahead_size;
        self
    }

    pub const fn set_compact_threshold(mut self, compact_threshold: usize) -> Self {
        self.compact_threshold = Some(compact_threshold);

        self
    }

    pub const fn set_metadata_maximum(mut self, metadata_maxium: usize) -> Self {
        self.metadata_maxium = Some(metadata_maxium);
        self
    }

    pub const fn set_inline_maximum(mut self, inline_maximum: usize) -> Self {
        self.inline_maximum = Some(inline_maximum);
        self
    }
}

impl TryFrom<Configuration> for littlefs::lfs_config {
    type Error = ();

    fn try_from(configuration: Configuration) -> Result<Self, Self::Error> {
        let mut read_buffer = vec![0_u8; configuration.cache_size];
        let mut write_buffer = read_buffer.clone();
        let mut look_ahead_buffer = vec![0_u8; configuration.look_ahead_size];

        let lfs_configuration = littlefs::lfs_config {
            context: Box::into_raw(Box::new(configuration.context)) as *mut c_void,
            read: Some(callbacks::read_callback),
            prog: Some(callbacks::programm_callback),
            erase: Some(callbacks::erase_callback),
            sync: Some(callbacks::flush_callback),
            read_size: configuration.read_size as u32,
            prog_size: configuration.program_size as u32,
            block_size: configuration.block_size as u32,
            block_count: configuration.block_count as u32,
            block_cycles: match configuration.block_cycles {
                Some(block_cycles) => block_cycles as i32,
                None => -1,
            },
            cache_size: configuration.cache_size as u32,
            lookahead_size: configuration.look_ahead_size as u32,
            read_buffer: read_buffer.as_mut_ptr() as *mut c_void,
            prog_buffer: write_buffer.as_mut_ptr() as *mut c_void,
            lookahead_buffer: look_ahead_buffer.as_mut_ptr() as *mut c_void,
            name_max: configuration.maximum_name_size.unwrap_or(0) as u32, // Default value : 255 (LFS_NAME_MAX)
            file_max: configuration.maximum_file_size.unwrap_or(0) as u32, // Default value : 2,147,483,647 (2 GiB) (LFS_FILE_MAX)
            attr_max: configuration.maximum_attributes_size.unwrap_or(0) as u32, // Default value : 1022 (LFS_ATTR_MAX)
            compact_thresh: configuration.compact_threshold.unwrap_or(0) as u32, // Default value : 0 (LFS_COMPACT_THRESH)
            metadata_max: configuration.metadata_maxium.unwrap_or(0) as u32, // Default value : 0 (LFS_METADATA_MAX)
            inline_max: configuration.inline_maximum.unwrap_or(0) as u32, // Default value : 0 (LFS_INLINE_MAX)
            flags: 0, // Default value : 0 (LFS_CONFIG_FLAGS)
        };

        forget(read_buffer);
        forget(write_buffer);
        forget(look_ahead_buffer);

        Ok(lfs_configuration)
    }
}
