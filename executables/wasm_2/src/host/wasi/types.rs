#[repr(C)]
pub struct Filestat {
    dev: [u8; 8],
    ino: [u8; 8],
    filetype: u8,
    padding: [u8; 7],
    nlink: [u8; 8],
    size: [u8; 8],
    atim: [u8; 8],
    mtim: [u8; 8],
    ctim: [u8; 8],
}

impl Filestat {
    pub fn new(
        inode: u64,
        filetype: u8,
        size: u64,
        access_time: u64,
        modification_time: u64,
        creation_time: u64,
    ) -> Self {
        Self {
            dev: 0u64.to_le_bytes(),
            ino: inode.to_le_bytes(),
            filetype,
            padding: [0; 7],
            nlink: 1u64.to_le_bytes(),
            size: size.to_le_bytes(),
            atim: access_time.to_le_bytes(),
            mtim: modification_time.to_le_bytes(),
            ctim: creation_time.to_le_bytes(),
        }
    }

    pub fn as_bytes(&self) -> &[u8; core::mem::size_of::<Self>()] {
        // Byte-only fields make this representation aligned and padding-free.
        unsafe { &*(core::ptr::from_ref(self).cast()) }
    }
}

const _: () = assert!(core::mem::size_of::<Filestat>() == 64);
