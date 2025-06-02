#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos",))]
pub mod Std;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos",))]
pub use Std::*;
