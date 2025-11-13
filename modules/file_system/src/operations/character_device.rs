use crate::{BaseOperations, DirectFileOperations};

pub trait CharacterDevice: BaseOperations {}

pub trait DirectCharacterDevice: DirectFileOperations {}
