
// winit
#[cfg(feature = "iced")]
pub use iced_winit::winit;

#[cfg(not(feature = "iced"))]
pub use winit;

// log
pub use log;
pub use log::Level as LogLevel;


// mods
mod refs;
pub use refs::Static;

mod entry_point;
pub use entry_point::main;