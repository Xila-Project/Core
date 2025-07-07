use core::num::NonZeroUsize;

use alloc::string::String;
use Executable::Standard_type;

use crate::File_manager_type;

pub async fn Main(_: Standard_type, _: String) -> Result<(), NonZeroUsize> {
    let mut file_manager = File_manager_type::new()
        .await
        .map_err(|_| NonZeroUsize::new(1).unwrap())?;

    // Run the main loop
    file_manager.Run().await;

    Ok(())
}
