use crate::Type_type;

use super::Inode_type;

pub struct Entry_type {
    Inode: Inode_type,
    Name: String,
    Type: Type_type,
}

impl Entry_type {
    pub fn New(Inode: Inode_type, Name: String, Type: Type_type) -> Self {
        Self { Inode, Name, Type }
    }

    pub fn Get_inode(&self) -> Inode_type {
        self.Inode
    }

    pub fn Get_name(&self) -> &String {
        &self.Name
    }

    pub fn Get_type(&self) -> Type_type {
        self.Type
    }

    pub fn Set_inode(&mut self, Inode: Inode_type) {
        self.Inode = Inode;
    }

    pub fn Set_name(&mut self, Name: String) {
        self.Name = Name;
    }

    pub fn Set_type(&mut self, Type: Type_type) {
        self.Type = Type;
    }
}

impl AsMut<[u8]> for Entry_type {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self as *mut Entry_type as *mut u8,
                core::mem::size_of::<Entry_type>(),
            )
        }
    }
}

impl TryFrom<&mut [u8]> for &mut Entry_type {
    type Error = ();

    fn try_from(Value: &mut [u8]) -> Result<Self, Self::Error> {
        if Value.len() != core::mem::size_of::<Entry_type>() {
            return Err(());
        }
        if Value.as_ptr() as usize % core::mem::align_of::<Entry_type>() != 0 {
            return Err(());
        }

        #[allow(clippy::transmute_ptr_to_ref)]
        Ok(unsafe { core::mem::transmute::<*mut u8, &mut Entry_type>(Value.as_mut_ptr()) })
    }
}
