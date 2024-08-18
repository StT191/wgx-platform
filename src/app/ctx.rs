
use std::sync::Arc;
use winit::window::Window;
use crate::*;

#[cfg(feature = "frame_timer")]
use crate::{time::Duration, app::STD_DURATION};


#[derive(Debug)]
pub struct AppCtx {
  #[cfg(feature = "frame_timer")] pub duration: Duration,
  #[cfg(feature = "frame_timer")] pub animate: bool,
  #[cfg(feature = "frame_timer")] pub request: Option<Duration>,
  #[cfg(feature = "auto_wake_lock")] pub auto_wake_lock: bool,
  pub exit: bool,
  _event_loop_proxy: PlatformEventLoopProxy,
  pub window: Arc<Window>,
}


impl AppCtx {
  pub(super) fn new(_event_loop_proxy: PlatformEventLoopProxy, window: Window) -> Self { Self {
    #[cfg(feature = "frame_timer")] duration: STD_DURATION,
    #[cfg(feature = "frame_timer")] animate: false,
    #[cfg(feature = "frame_timer")] request: None,
    #[cfg(feature = "auto_wake_lock")] auto_wake_lock: false,
    exit: false,
    _event_loop_proxy,
    window: Arc::new(window),
  }}
}