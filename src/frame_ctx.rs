
use std::sync::Arc;
use crate::winit::{window::{Window, /*WindowId,*/}, event::WindowEvent, event_loop::{EventLoopWindowTarget, ControlFlow}};

#[cfg(feature = "frame_timer")]
use crate::winit::event::StartCause;

#[cfg(feature = "auto_wake_lock")]
use crate::error::inspect;

use crate::{*, EventLoopWindowTarget as PlatformEventLoopWindowTarget};


#[cfg(feature = "frame_timer")]
pub const STD_DURATION: Duration = Duration::from_nanos(1_000_000_000/60);


#[derive(Debug)]
pub struct FrameCtx {
  #[cfg(feature = "frame_timer")] pub duration: Duration,
  #[cfg(feature = "frame_timer")] pub animate: bool,
  #[cfg(feature = "frame_timer")] pub request: Option<Duration>,
  #[cfg(feature = "auto_wake_lock")] pub auto_wake_lock: bool,
  pub exit: bool,
}

impl Default for FrameCtx {
  fn default() -> Self { Self {
    #[cfg(feature = "frame_timer")] duration: STD_DURATION,
    #[cfg(feature = "frame_timer")] animate: false,
    #[cfg(feature = "frame_timer")] request: None,
    #[cfg(feature = "auto_wake_lock")] auto_wake_lock: false,
    exit: false,
  }}
}


pub trait ControlFlowExtension {
  fn set_poll(&self);
  fn set_wait(&self);
  fn set_wait_until(&self, instant: Instant);
  fn set_earlier(&self, instant: Instant);
}

impl<T> ControlFlowExtension for EventLoopWindowTarget<T> {

  fn set_poll(&self) { self.set_control_flow(ControlFlow::Poll); }
  fn set_wait(&self) { self.set_control_flow(ControlFlow::Wait); }
  fn set_wait_until(&self, instant: Instant) { self.set_control_flow(ControlFlow::WaitUntil(instant)); }

  fn set_earlier(&self, instant: Instant) {
    match self.control_flow() {
      ControlFlow::Poll => {},
      ControlFlow::Wait => self.set_wait_until(instant),
      ControlFlow::WaitUntil(other) => self.set_wait_until(instant.min(other)),
    }
  }
}


impl FrameCtx {

  pub fn new() -> Self { Self::default() }

  pub fn run(mut self,
    window: Arc<Window>,
    mut event_handler: impl FnMut(&mut FrameCtx, &WindowEvent) + 'static
  )
    -> impl FnMut(WinitEvent, &PlatformEventLoopWindowTarget)
  {

    // wake lock
    #[cfg(feature = "auto_wake_lock")]
    let mut wake_lock: Option<WakeLock> = WakeLock::new().inspect_err(|err| inspect(err)).ok();


    #[cfg(feature = "frame_timer")]
    let mut animate = DetectChanges::new(!self.animate); // force to change on first event

    #[cfg(feature = "frame_timer")]
    let mut requested = DetectChanges::new(None); // force to change on first event

    // frame timer
    #[cfg(feature = "frame_timer")]
    let mut last = Instant::now();

    #[cfg(feature = "frame_timer")]
    let mut next = last + self.duration;

    // event loop
    // let window_id = window.id();

    move |event, event_target| match event {

      #[cfg(feature = "frame_timer")]
      WinitEvent::NewEvents(StartCause::ResumeTimeReached {..}) => {
        window.request_redraw();
      },

      WinitEvent::WindowEvent { window_id: _id, event: window_event } /*if id == &window_id*/ => {

        // before user handler
        match &window_event {

          #[cfg(feature = "frame_timer")]
          WindowEvent::RedrawRequested => {

            let now = Instant::now();

            self.request = None;
            requested.set_state(None);

            last = if next > now || (next + self.duration) <= now {
              now
            } else {
              next // avoid timer shifts
            };

            next = last + self.duration;

            if self.animate { event_target.set_wait_until(next); }
            else { event_target.set_wait(); }
          },

          WindowEvent::CloseRequested => {
            self.exit = true;
          },

          _ => {},
        }


        // exec event handler
        event_handler(&mut self, &window_event);


        // handle iteraction

        if self.exit {
          event_target.exit();
          return;
        }

        // animation
        #[cfg(feature = "frame_timer")] // detect state changes ... set control flow
        {
          if animate.note_change(&self.animate) {
            if self.animate {

              self.request = None;
              requested.set_state(None);

              #[cfg(feature = "auto_wake_lock")]
              if self.auto_wake_lock {
                wake_lock.as_mut().map(|lock| lock.request().unwrap_or_else(inspect));
              }

              let now = Instant::now();

              if next <= now {
                next = now; // reset frame_timer
                window.request_redraw();
              }
              else {
                event_target.set_earlier(next);
              }
            }
            else {
              #[cfg(feature = "auto_wake_lock")]
              wake_lock.as_mut().map(|lock| lock.release().unwrap_or_else(inspect));

              event_target.set_wait();
            }
          }

          if requested.changed(&self.request) {

            if self.animate {
              self.request = None;
              requested.set_state(None);
            }
            else if let Some(delay) = self.request {

              let earlier = match requested.state() {
                None => {
                  requested.set_state(Some(delay));
                  true // is definitely eralier
                },
                Some(state) if delay < *state => {
                  requested.set_state(Some(delay));
                  true // is earlier, checked
                },
                Some(previous) => {
                  self.request = Some(*previous); // reset to previous
                  false // is not earlier
                },
              };

              if earlier {
                if let Some(instant) = last.checked_add(delay) {

                  let now = Instant::now();
                  if next < now { next = now }

                  if instant > next {
                    event_target.set_earlier(instant);
                  }
                  else {
                    event_target.set_earlier(next);
                  }
                }
                // else consider as infinite delay, keep ControlFLow::Wait
              }
            }
            else {
              self.request = *requested.state(); // reset to previous
            }
          }
        }

        // other window event handling
        match window_event {

          WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged {..} => {
            window.request_redraw();
          },

          #[cfg(feature = "auto_wake_lock")]
          WindowEvent::Focused(focus) => {
            if !focus {
              wake_lock.as_mut().map(|lock| lock.release().unwrap_or_else(inspect));
            } else if self.auto_wake_lock {
              wake_lock.as_mut().map(|lock| lock.request().unwrap_or_else(inspect));
            }
          },

          _ => {}
        }

      },

      _ => {}

    }
  }
}