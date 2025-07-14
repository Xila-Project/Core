use core::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, RwLock},
};

use file_system::{Inode_type, Local_file_identifier_type, Path_owned_type};
use virtual_file_system::VirtualFileSystem;

use crate::Result;

struct Socket_type<'a> {
    Data: VecDeque<&'a [u8]>,
    Connection: Option<()>,
}

pub struct Local_socket_manager_type<'a> {
    Virtual_file_system: &'a VirtualFileSystem<'a>,
    Open_sockets: RwLock<BTreeMap<Local_file_identifier_type, Arc<Socket_type<'a>>>>,
    Sockets: RwLock<BTreeMap<Inode_type, Arc<Socket_type<'a>>>>,
}

impl<'a> Local_socket_manager_type<'a> {
    pub fn is_socket_identifier_used(
        &self,
        Socket: Local_file_identifier_type,
    ) -> Result<bool> {
        Ok(self.Open_sockets.read().unwrap().contains_key(&Socket))
    }

    pub fn New(Virtual_file_system: &'a VirtualFileSystem<'a>) -> Self {
        Self {
            Virtual_file_system,
            Open_sockets: RwLock::new(BTreeMap::new()),
            Sockets: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn Bind(Path: Path_owned_type, Socket: Local_file_identifier_type) -> Result<()> {
        todo!()
    }
}
