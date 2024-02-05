
use std::sync::Arc;
use crate::winit::{window::{Window, /*WindowId,*/}, event::WindowEvent};

#[cfg(feature = "timer")]
use crate::winit::{event::StartCause, event_loop::ControlFlow};

#[cfg(feature = "wake_lock")]
use crate::error::inspect;

use crate::*;


#[cfg(feature = "timer")]
pub const STD_DURATION: Duration = Duration::from_nanos(1_000_000_000/60);


#[derive(Debug)]
pub struct FrameCtx {
  #[cfg(feature = "timer")] pub duration: Duration,
  #[cfg(feature = "timer")] pub animate: bool,
  #[cfg(feature = "wake_lock")] pub wake_lock: bool,
}


impl FrameCtx {

  pub fn new() -> Self { Self {
    #[cfg(feature = "timer")] duration: STD_DURATION,
    #[cfg(feature = "timer")] animate: false,
    #[cfg(feature = "wake_lock")] wake_lock: false,
  }}

  pub fn run(mut self,
    event_loop: EventLoop,
    window: Arc<Window>,
    mut event_handler: impl FnMut(&mut FrameCtx, &WindowEvent) + 'static
  ) {

    // wake lock
    #[cfg(feature = "wake_lock")]
    let mut wake_lock: Option<WakeLock> = WakeLock::new().inspect_err(|err| inspect(err)).ok();


    #[cfg(feature = "timer")]
    let mut animate = DetectChanges::new(!self.animate); // force to change on first event

    // frame timer
    #[cfg(feature = "timer")]
    let mut frame_timer = StepInterval::new(self.duration);


    // event loop
    // let window_id = window.id();

    event_loop.run(move |event, event_target| {

      // handle app input first
      match &event {
        WinitEvent::WindowEvent { window_id: _id, event: window_event } /*if id == &window_id*/ => {
          event_handler(&mut self, window_event);
        },
        _ => {}
      }


      // detect state changes ... set control flow
      #[cfg(feature = "timer")]
      {
        if animate.note_change(&self.animate) {
          if *animate.state() {

            #[cfg(feature = "wake_lock")]
            if self.wake_lock {
              wake_lock.as_mut().map(|lock| lock.request().unwrap_or_else(inspect));
            }

            window.request_redraw();

            // reset frame timer
            frame_timer = StepInterval::new(self.duration);
            event_target.set_control_flow(ControlFlow::WaitUntil(frame_timer.next));

          } else {
            #[cfg(feature = "wake_lock")]
            wake_lock.as_mut().map(|lock| lock.release().unwrap_or_else(inspect));

            event_target.set_control_flow(ControlFlow::Wait);
          }
        }
      }

      match event {

        #[cfg(feature = "timer")]
        WinitEvent::NewEvents(StartCause::ResumeTimeReached {..}) => {
          if *animate.state() {
            window.request_redraw(); // request frame
            frame_timer.step();
            event_target.set_control_flow(ControlFlow::WaitUntil(frame_timer.next));
          }
        },

        WinitEvent::WindowEvent { window_id: _id, event: window_event } /*if id == window_id*/ => {
          match window_event {

            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged {..} => {
              window.request_redraw();
            },

            #[cfg(feature = "wake_lock")]
            WindowEvent::Focused(focus) => {
              if !focus {
                wake_lock.as_mut().map(|lock| lock.release().unwrap_or_else(inspect));
              } else if self.wake_lock {
                wake_lock.as_mut().map(|lock| lock.request().unwrap_or_else(inspect));
              }
            }

            WindowEvent::CloseRequested => {
              event_target.exit();
            },

            _ => {}
          }
        },
        _ => {}
      }
    }).unwrap();

  }

}


// wgx frame ctx
#[cfg(feature = "wgx")]
mod gx_ctx {

  use super::*;
  use wgx::{Wgx, SurfaceTarget, Limits, Features};


  #[derive(Debug)]
  pub struct GxCtx { pub gx: Wgx, pub target: SurfaceTarget }


  impl GxCtx {

    pub async fn new(window: Arc<Window>, features: Features, limits: Limits, msaa: u32, depth_testing: bool) -> Self {

      let size = window.inner_size();

      let (gx, surface) = Wgx::new(Some(window), features, limits).await.unwrap();

      let target = SurfaceTarget::new(&gx, surface.unwrap(), (size.width, size.height), msaa, depth_testing).unwrap();

      Self { gx, target }
    }

    pub fn run(mut self,
      frame_ctx: FrameCtx,
      event_loop: EventLoop,
      window: Arc<Window>,
      mut event_handler: impl FnMut(&mut FrameCtx, &mut GxCtx, &WindowEvent) + 'static,
    ) {
      frame_ctx.run(event_loop, window, move |frame_ctx, event| {

        // resize handler
        if let WindowEvent::Resized(size) = &event {
          self.target.update(&self.gx, (size.width as u32, size.height as u32));
        }

        event_handler(frame_ctx, &mut self, event);

      });
    }
  }

}

#[cfg(feature = "wgx")]
pub use gx_ctx::*;


