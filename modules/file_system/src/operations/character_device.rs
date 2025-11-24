use crate::{
    BaseOperations, ControlCommand, ControlDirectionFlags, DirectBaseOperations, MountOperations,
};

pub const IS_A_TERMINAL: ControlCommand =
    ControlCommand::new::<bool>(ControlDirectionFlags::Read, b'T', 0);

pub trait CharacterDevice: BaseOperations + MountOperations {}

pub trait DirectCharacterDevice: DirectBaseOperations + MountOperations {}

impl<T: DirectCharacterDevice> CharacterDevice for T {}
