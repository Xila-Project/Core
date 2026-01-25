use crate::{
    Shell,
    error::{Error, Result},
};
use core::fmt::Write;
use xila::{
    file_system::{AccessFlags, Path},
    log,
    network::{GET_IP_ADDRESS, GET_IP_ADDRESS_COUNT, GET_ROUTE, GET_ROUTE_COUNT, GET_STATE},
    task::{self, TaskIdentifier},
    virtual_file_system::{self, Directory, File, FileControlIterator, VirtualFileSystem},
};

impl Shell {
    pub async fn open_interface(
        virtual_file_system: &VirtualFileSystem,
        interface: &str,
    ) -> Result<File> {
        let task_manager = xila::task::get_instance();

        let task = task_manager.get_current_task_identifier().await;

        let path = Path::NETWORK_DEVICES
            .join(interface)
            .ok_or(Error::FailedToJoinPath)?;

        File::open(virtual_file_system, task, &path, AccessFlags::Read.into())
            .await
            .map_err(Error::FailedToOpenFile)
    }

    async fn show_routes_interface(
        &mut self,
        interface: &str,
        file: &mut File,
    ) -> crate::Result<()> {
        let mut routes = FileControlIterator::new(file, GET_ROUTE_COUNT, GET_ROUTE)
            .await
            .map_err(Error::FailedToOpenFile)?;

        while let Some(route) = routes.next().await.map_err(Error::FailedToOpenFile)? {
            writeln!(
                self.standard.out(),
                "{} via {} device {}",
                route.cidr,
                route.via_router,
                interface
            )?;
        }

        Ok(())
    }

    async fn show_routes(
        &mut self,
        virtual_file_system: &VirtualFileSystem,
        task: TaskIdentifier,
    ) -> crate::Result<()> {
        let mut directory = Directory::open(virtual_file_system, task, Path::NETWORK_DEVICES)
            .await
            .map_err(Error::FailedToOpenDirectory)?;

        while let Some(entry) = directory
            .read()
            .await
            .map_err(Error::FailedToReadDirectoryEntry)?
        {
            if entry.name == "." || entry.name == ".." {
                continue;
            }

            let mut file = Self::open_interface(virtual_file_system, &entry.name).await?;

            self.show_routes_interface(&entry.name, &mut file).await?;
        }

        Ok(())
    }

    async fn show_address_interface(&mut self, file: &mut File) -> crate::Result<()> {
        let mut addresses = FileControlIterator::new(file, GET_IP_ADDRESS_COUNT, GET_IP_ADDRESS)
            .await
            .map_err(Error::FailedToOpenFile)?;

        while let Some(address) = addresses.next().await.map_err(Error::FailedToOpenFile)? {
            writeln!(self.standard.out(), "   {} ", address)?;
        }

        Ok(())
    }

    async fn show_address(
        &mut self,
        virtual_file_system: &VirtualFileSystem,
        task: TaskIdentifier,
    ) -> crate::Result<()> {
        let mut directory = Directory::open(virtual_file_system, task, Path::NETWORK_DEVICES)
            .await
            .map_err(Error::FailedToOpenDirectory)?;

        let mut index = 1;

        while let Some(entry) = directory
            .read()
            .await
            .map_err(Error::FailedToReadDirectoryEntry)?
        {
            if entry.name == "." || entry.name == ".." {
                continue;
            }

            log::information!("Showing address for interface {}", entry.name);

            let mut file = Self::open_interface(virtual_file_system, &entry.name).await?;

            let state = file
                .control(GET_STATE, &())
                .await
                .map_err(Error::FailedToOpenFile)?;

            let state = if state { "Enabled" } else { "Disabled" };

            let status = file
                .control(xila::network::IS_LINK_UP, &())
                .await
                .map_err(Error::FailedToOpenFile)?;

            let status: &str = if status { "Up" } else { "Down" };

            let hardware_address = file
                .control(xila::network::GET_HARDWARE_ADDRESS, &())
                .await
                .map_err(Error::FailedToOpenFile)?;

            let maximum_transmission_unit = file
                .control(xila::network::GET_MAXIMUM_TRANSMISSION_UNIT, &())
                .await
                .map_err(Error::FailedToOpenFile)?;

            writeln!(
                self.standard.out(),
                "{}. {} [{}, {}, MTU: {}]",
                index,
                entry.name,
                state,
                status,
                maximum_transmission_unit
            )?;
            writeln!(
                self.standard.out(),
                "   Hardware Address: {:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
                hardware_address[0],
                hardware_address[1],
                hardware_address[2],
                hardware_address[3],
                hardware_address[4],
                hardware_address[5],
            )?;

            self.show_address_interface(&mut file).await?;

            index += 1;
        }

        Ok(())
    }

    pub async fn ip<'a, I>(
        &mut self,
        options: &mut getargs::Options<&'a str, I>,
    ) -> crate::Result<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let command = options
            .next_positional()
            .ok_or(crate::Error::MissingPositionalArgument("command"))?;

        let virtual_file_system = virtual_file_system::get_instance();
        let task_manager = task::get_instance();
        let task = task_manager.get_current_task_identifier().await;

        match command {
            "address" | "a" => self.show_address(virtual_file_system, task).await?,
            "route" | "r" => self.show_routes(virtual_file_system, task).await?,
            _ => {
                return Err(crate::Error::InvalidOption);
            }
        }

        Ok(())
    }
}
