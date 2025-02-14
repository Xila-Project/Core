use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, RwLock},
};

use File_system::{Inode_type, Local_file_identifier_type, Path_owned_type};
use Virtual_file_system::Virtual_file_system_type;

use crate::Result_type;

struct Socket_type<'a> {
    Data: VecDeque<&'a [u8]>,
    Connection: Option<()>,
}

pub struct Local_socket_manager_type<'a> {
    Virtual_file_system: &'a Virtual_file_system_type<'a>,
    Open_sockets: RwLock<BTreeMap<Local_file_identifier_type, Arc<Socket_type<'a>>>>,
    Sockets: RwLock<BTreeMap<Inode_type, Arc<Socket_type<'a>>>>,
}

impl<'a> Local_socket_manager_type<'a> {
    pub fn Is_socket_identifier_used(
        &self,
        Socket: Local_file_identifier_type,
    ) -> Result_type<bool> {
        Ok(self.Open_sockets.read().unwrap().contains_key(&Socket))
    }

    pub fn New(Virtual_file_system: &'a Virtual_file_system_type<'a>) -> Self {
        Self {
            Virtual_file_system,
            Open_sockets: RwLock::new(BTreeMap::new()),
            Sockets: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn Bind(Path: Path_owned_type, Socket: Local_file_identifier_type) -> Result_type<()> {
        todo!()
    }
}
