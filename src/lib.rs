
// log
pub use log;
pub use log::Level as LogLevel;

// mods
mod conditional_execution;
pub use conditional_execution::*;

mod entry_point;
pub use entry_point::*;

mod application;
pub use application::*;

pub mod error;


// re-exports
pub use async_trait::async_trait;

#[cfg(not(target_family="wasm"))]
pub use pollster;


// timer
#[cfg(feature = "timer")]
mod timer;

#[cfg(feature = "timer")]
pub use timer::*;


// icon loader
#[cfg(feature = "icon_loader")]
#[cfg(target_os = "linux")]
pub mod icon_loader;


// wake_lock
#[cfg(feature = "wake_lock")]
mod wake_lock;
#[cfg(feature = "wake_lock")]
pub use wake_lock::*;


// winit / iced
#[cfg(feature = "iced")]
pub use iced_winit::winit;

#[cfg(feature = "iced")]
pub mod iced;

#[cfg(not(feature = "iced"))]
pub use winit;