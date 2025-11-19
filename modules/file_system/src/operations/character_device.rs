use crate::{
    BaseOperations, ControlCommand, ControlDirection, DirectBaseOperations, MountOperations,
};

pub const IS_A_TERMINAL: ControlCommand =
    ControlCommand::new::<bool>(ControlDirection::READ, b'T', 0);

pub trait CharacterDevice: BaseOperations + MountOperations {}

pub trait DirectCharacterDevice: DirectBaseOperations + MountOperations {}

impl<T: DirectCharacterDevice> CharacterDevice for T {}
