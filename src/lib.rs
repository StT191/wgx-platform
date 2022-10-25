
// winit / iced
#[cfg(feature = "iced")]
pub use iced_winit::winit;

#[cfg(feature = "iced")]
mod iced_clipboard;
#[cfg(feature = "iced")]
pub use iced_clipboard::*;


#[cfg(not(feature = "iced"))]
pub use winit;


// log
pub use log;
pub use log::Level as LogLevel;


// mods
mod entry_point;
pub use entry_point::main;


#[cfg(feature = "wake_lock")]
mod wake_lock;
#[cfg(feature = "wake_lock")]
pub use wake_lock::*;


#[cfg(feature = "icon_loader")]
#[cfg(target_os = "linux")]
pub mod icon_loader;


// error helper
pub mod error;
