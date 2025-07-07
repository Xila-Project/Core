use core::num::NonZeroUsize;

use alloc::string::String;
use Executable::Standard_type;

use crate::Settings::Settings_type;

pub async fn Main(_: Standard_type, _: String) -> Result<(), NonZeroUsize> {
    let mut settings = Settings_type::new()
        .await
        .map_err(|_| NonZeroUsize::new(1).unwrap())?;

    // Run the main loop
    settings.Run().await;

    Ok(())
}
