
// re-exports
pub use winit;
pub use web_time::{Instant, Duration};

#[cfg(not(target_family="wasm"))]
pub use pollster;

pub use log::{self, Level as LogLevel};

// mods
mod conditional_execution;
pub use conditional_execution::*;

mod entry_point;
pub use entry_point::*;

mod timer;
pub use timer::*;

pub mod frame_ctx;

pub mod error;

mod future;
pub use future::*;

// rng
#[cfg(feature = "rng")]
pub mod rng;

// wgx
#[cfg(feature = "wgx")]
pub use wgx;

#[cfg(feature = "wgx")]
mod gx_ctx;
#[cfg(feature = "wgx")]
pub use gx_ctx::*;

// icon loader
#[cfg(feature = "icon_loader")]
#[cfg(target_os = "linux")]
pub mod icon_loader;

// wake_lock
#[cfg(feature = "wake_lock")]
mod wake_lock;
#[cfg(feature = "wake_lock")]
pub use wake_lock::*;

// egui
#[cfg(any(feature = "epaint", feature = "egui"))]
#[path="."]
pub mod egui {

    pub use epaint::{self, ecolor, emath};
    pub use egui_wgpu;

    #[cfg(feature = "egui")]
    pub use egui::*;

    #[cfg(feature = "egui")]
    pub use egui_winit;

    mod egui_ctx;
    pub use egui_ctx::*;
}