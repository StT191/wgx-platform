
#[cfg(feature = "timer")]
use crate::{Duration, StepInterval};

#[cfg(feature = "wake_lock")]
use crate::{WakeLock};

use crate::winit::{window::{Window, WindowBuilder}, event::{WindowEvent, StartCause}};
use crate::{async_trait, LogLevel, main, Event, DetectChanges};


#[cfg(feature = "timer")]
pub const STD_FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000/60);


#[async_trait(?Send)]
pub trait Application: Sized + 'static {

  // animation
  #[cfg(feature = "timer")]
  fn frame_duration(&self) -> Option<Duration>;

  #[cfg(feature = "timer")]
  fn animate(&self) -> bool;


  // wake lock
  #[cfg(feature = "wake_lock")]
  fn wake_lock(&self) -> bool;


  // provided defaults

  fn log_level() -> LogLevel { LogLevel::Warn }

  fn with_window_builder(window_builder: WindowBuilder) -> WindowBuilder {
    // optionally modify window building, default: pass original
    window_builder
  }

  // init
  async fn init(window: &Window) -> Self;

  // event handling
  fn event(&mut self, window: &Window, event: &WindowEvent);

  fn draw_frame(&mut self, window: &Window);


  fn run() {
    main(Self::with_window_builder, |window, event_loop| async move {

      fn inspect(err: impl std::fmt::Display) { log::warn!("{err}") }

      // frame timer
      #[cfg(feature = "timer")]
      let mut frame_timer = StepInterval::new(STD_FRAME_DURATION);

      #[cfg(feature = "timer")]
      let mut animate = DetectChanges::new(false);


      // wake lock
      #[cfg(feature = "wake_lock")]
      let mut wake_lock: Option<WakeLock> = WakeLock::new().inspect_err(|err| inspect(err)).ok();


      // instantiate app
      let mut app = Self::init(window).await;


      // event loop
      // let window_id = window.id();

      event_loop.run(move |event, _, control_flow| {

        // handle app input first
        match &event {
          Event::WindowEvent { window_id: _id, event: window_event } /*if id == &window_id*/ => {
            app.event(window, window_event);
          },
          _ => {}
        }


        // detect state changes ... set control flow
        #[cfg(feature = "timer")]
        {
          if animate.note_change(&app.animate()) {
            if *animate.state() {

              #[cfg(feature = "wake_lock")]
              if app.wake_lock() {
                wake_lock.as_mut().map(|lock| lock.request().unwrap_or_else(inspect));
              }

              window.request_redraw();

              // reset frame timer
              frame_timer = StepInterval::new(app.frame_duration().unwrap_or(STD_FRAME_DURATION));
              control_flow.set_wait_until(frame_timer.next);

            } else {
              #[cfg(feature = "wake_lock")]
              wake_lock.as_mut().map(|lock| lock.release().unwrap_or_else(inspect));

              control_flow.set_wait();
            }
          }
        }

        match event {

          Event::RedrawRequested(_id) /*if id == window_id*/ => {
            app.draw_frame(&window);
          },

          #[cfg(feature = "timer")]
          Event::NewEvents(StartCause::ResumeTimeReached {..}) => {
            if *animate.state() {
              window.request_redraw(); // request frame
              frame_timer.step();
              control_flow.set_wait_until(frame_timer.next);
            }
          },

          Event::WindowEvent { window_id: _id, event: window_event } /*if id == window_id*/ => {
            match window_event {

              WindowEvent::Resized(_)  => {
                window.request_redraw();
              },

              #[cfg(feature = "wake_lock")]
              WindowEvent::Focused(focus) => {
                if !focus {
                  wake_lock.as_mut().map(|lock| lock.release().unwrap_or_else(inspect));
                } else if app.wake_lock() {
                  wake_lock.as_mut().map(|lock| lock.request().unwrap_or_else(inspect));
                }
              }

              WindowEvent::CloseRequested => {
                control_flow.set_exit();
              },

              WindowEvent::ScaleFactorChanged {..} => {
                window.request_redraw();
              },

              _ => {}
            }
          },
          _ => {}
        }
      });

    }, Self::log_level());
  }

}