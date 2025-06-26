use core::num::NonZeroUsize;

use alloc::string::String;
use Executable::Standard_type;

use crate::File_manager_type;

pub async fn Main(_: Standard_type, _: String) -> Result<(), NonZeroUsize> {
    let mut File_manager = File_manager_type::New()
        .await
        .map_err(|_| NonZeroUsize::new(1).unwrap())?;

    // Show the file manager
    File_manager.Show();

    // Run the main loop
    File_manager.Run().await;

    Ok(())
}
