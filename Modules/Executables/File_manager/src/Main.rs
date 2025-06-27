use core::num::NonZeroUsize;

use alloc::string::String;
use Executable::Standard_type;
use Log::Information;

use crate::File_manager_type;

pub async fn Main(_: Standard_type, _: String) -> Result<(), NonZeroUsize> {
    Information!("File manager started...");

    let mut File_manager = File_manager_type::New()
        .await
        .map_err(|_| NonZeroUsize::new(1).unwrap())?;

    // Run the main loop
    File_manager.Run().await;

    Ok(())
}
