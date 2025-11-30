use file_system::{ControlCommand, ControlDirectionFlags};

use crate::{Area, Point};

pub const SET_DRAWING_AREA: ControlCommand =
    ControlCommand::new::<Area>(ControlDirectionFlags::Write, b'D', 0x01);
pub const GET_RESOLUTION: ControlCommand =
    ControlCommand::new::<Point>(ControlDirectionFlags::Read, b'D', 0x02);
pub const WAS_RESIZED: ControlCommand =
    ControlCommand::new::<bool>(ControlDirectionFlags::Read, b'D', 0x03);
