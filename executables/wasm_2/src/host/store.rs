use core::fmt::Display;

use crate::host::wasi::WasiContext;

pub struct GlobalStore {
    pub wasi: WasiContext,
}

impl Display for GlobalStore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "GlobalStore")
    }
}
