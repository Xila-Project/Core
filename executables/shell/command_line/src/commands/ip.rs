use crate::error::{Error, Result};
use executable_macros::GetArgs;
use xila::{
    file_system::{AccessFlags, Path},
    log,
    network::{GET_IP_ADDRESS, GET_IP_ADDRESS_COUNT, GET_ROUTE, GET_ROUTE_COUNT, GET_STATE},
    task,
    virtual_file_system::{self, Directory, File, FileControlIterator, VirtualFileSystem},
};

use super::{CommandContext, UserCommand};

pub struct IpCommand;

impl UserCommand for IpCommand {
    async fn execute<'a, I, C>(
        &self,
        context: &mut C,
        options: &mut getargs::Options<&'a str, I>,
        _paths: &[&Path],
    ) -> crate::Result<()>
    where
        I: Iterator<Item = &'a str>,
        C: CommandContext,
    {
        execute_ip(context, options).await
    }
}

#[derive(GetArgs)]
struct IpArguments<'a> {
    command: &'a str,
}

async fn open_interface(virtual_file_system: &VirtualFileSystem, interface: &str) -> Result<File> {
    let task = task::get_instance().get_current_task_identifier().await;
    let path = Path::NETWORK_DEVICES
        .join(interface)
        .ok_or(Error::FailedToJoinPath)?;

    File::open(virtual_file_system, task, &path, AccessFlags::Read.into())
        .await
        .map_err(Error::FailedToOpenFile)
}

async fn show_routes_interface<C: CommandContext>(
    context: &mut C,
    interface: &str,
    file: &mut File,
) -> crate::Result<()> {
    let mut routes = FileControlIterator::new(file, GET_ROUTE_COUNT, GET_ROUTE)
        .await
        .map_err(Error::FailedToOpenFile)?;

    while let Some(route) = routes.next().await.map_err(Error::FailedToOpenFile)? {
        context.write_out_fmt(format_args!(
            "{} via {} device {}\n",
            route.cidr, route.via_router, interface
        ))?;
    }

    Ok(())
}

async fn show_routes<C: CommandContext>(
    context: &mut C,
    virtual_file_system: &VirtualFileSystem,
    task: xila::task::TaskIdentifier,
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

        let mut file = open_interface(virtual_file_system, &entry.name).await?;
        show_routes_interface(context, &entry.name, &mut file).await?;
    }

    Ok(())
}

async fn show_address_interface<C: CommandContext>(
    context: &mut C,
    file: &mut File,
) -> crate::Result<()> {
    let mut addresses = FileControlIterator::new(file, GET_IP_ADDRESS_COUNT, GET_IP_ADDRESS)
        .await
        .map_err(Error::FailedToOpenFile)?;

    while let Some(address) = addresses.next().await.map_err(Error::FailedToOpenFile)? {
        context.write_out_fmt(format_args!("   {} \n", address))?;
    }

    Ok(())
}

async fn show_address<C: CommandContext>(
    context: &mut C,
    virtual_file_system: &VirtualFileSystem,
    task: xila::task::TaskIdentifier,
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

        let mut file = open_interface(virtual_file_system, &entry.name).await?;

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

        context.write_out_fmt(format_args!(
            "{}. {} [{}, {}, MTU: {}]\n",
            index, entry.name, state, status, maximum_transmission_unit
        ))?;
        context.write_out_fmt(format_args!(
            "   Hardware Address: {:x}:{:x}:{:x}:{:x}:{:x}:{:x}\n",
            hardware_address[0],
            hardware_address[1],
            hardware_address[2],
            hardware_address[3],
            hardware_address[4],
            hardware_address[5]
        ))?;

        show_address_interface(context, &mut file).await?;

        index += 1;
    }

    Ok(())
}

async fn execute_ip<'a, I, C>(
    context: &mut C,
    options: &mut getargs::Options<&'a str, I>,
) -> crate::Result<()>
where
    I: Iterator<Item = &'a str>,
    C: CommandContext,
{
    let IpArguments { command } = IpArguments::parse(options)?;

    let virtual_file_system = virtual_file_system::get_instance();
    let task = task::get_instance().get_current_task_identifier().await;

    match command {
        "address" | "a" => show_address(context, virtual_file_system, task).await?,
        "route" | "r" => show_routes(context, virtual_file_system, task).await?,
        _ => return Err(crate::Error::InvalidOption),
    }

    Ok(())
}
