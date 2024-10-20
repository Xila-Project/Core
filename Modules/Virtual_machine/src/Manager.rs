use std::{
    collections::BTreeMap,
    sync::{OnceLock, RwLock},
};

use File_system::Unique_file_identifier_type;
use Task::Thread_identifier_type;

use crate::{Instance_type, Module_type, Registrable_trait, Result_type, Runtime_type};

static Manager_instance: OnceLock<Manager_type> = OnceLock::new();

pub fn Create_instance(Registrables: Vec<&dyn Registrable_trait>) -> &'static Manager_type {
    Manager_instance.get_or_init(|| {
        Manager_type::New(Registrables).expect("Cannot create virtual machine manager")
    });

    Get_instance()
}

pub fn Get_instance() -> &'static Manager_type {
    Manager_instance
        .get()
        .expect("Cannot get virtual machine manager instance before initialization")
}

struct Inner_type {
    pub Runtime: Runtime_type,
    pub Modules: Vec<Module_type>,
    pub Instances: BTreeMap<Thread_identifier_type, Instance_type>,
}

pub struct Manager_type(RwLock<Inner_type>);

unsafe impl Send for Manager_type {}

unsafe impl Sync for Manager_type {}

impl Manager_type {
    pub fn New(Registrables: Vec<&dyn Registrable_trait>) -> Result_type<Self> {
        let mut Runtime_builder = Runtime_type::Builder();

        for Registrable in Registrables {
            Runtime_builder = Runtime_builder.Register(Registrable);
        }

        let Runtime = Runtime_builder.Build()?;

        Ok(Self(RwLock::new(Inner_type {
            Runtime,
            Modules: Vec::new(),
            Instances: BTreeMap::new(),
        })))
    }

    pub fn Load_module(&self, Name: &str, Buffer: Vec<u8>) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        let Module = Module_type::From_buffer(&Inner.Runtime, Buffer, Name)?;

        Inner.Modules.push(Module);

        Ok(())
    }

    pub fn Instantiate(
        &self,
        Module_name: &str,
        Stack_size: usize,
        Standard_in: Unique_file_identifier_type,
        Standard_out: Unique_file_identifier_type,
        Standard_error: Unique_file_identifier_type,
    ) -> Result_type<()> {
        let mut Inner = self.0.write()?;

        todo!()
    }
}
