
use winit::{window::{WindowBuilder, Window, WindowId}, event::WindowEvent};
use std::sync::mpsc::{Receiver, sync_channel};

use crate::*;
use super::{AppHandler, AppState, AppCtx};


enum MountState<App: AppHandler> {
  Init {
    event_queue: Vec<PlatformEvent>,
    window_builder: WindowBuilder,
    event_loop_proxy: PlatformEventLoopProxy,
    init_data: App::InitData,
  },
  Window {
    event_queue: Vec<PlatformEvent>,
    window: Window,
    event_loop_proxy: PlatformEventLoopProxy,
    init_data: App::InitData,
  },
  Mounting {
    event_queue: Vec<PlatformEvent>,
    window_id: WindowId,
    receiver: Receiver<AppState<App>>,
  },
  Mounted(AppState<App>),
}


pub struct AppMount<App: AppHandler> {
  state: MountState<App>,
}

impl<App: AppHandler> AppMount<App> {

  pub fn mount(event_loop_proxy: PlatformEventLoopProxy, window_builder: WindowBuilder, init_data: App::InitData) -> Self {
    Self { state: MountState::Init { event_queue: Vec::new(), window_builder, event_loop_proxy, init_data } }
  }

  pub fn run(mut self, event_loop: PlatformEventLoop) {
    event_loop.run(move |event, event_target| {
      self.event(event, event_target);
    }).unwrap();
  }

  pub fn start(window_builder: WindowBuilder, init_data: App::InitData) {
    let event_loop = event_loop();
    Self::mount(event_loop.create_proxy(), window_builder, init_data).run(event_loop);
  }

  pub fn event(&mut self, event: PlatformEvent, event_target: &PlatformEventLoopWindowTarget) {

    match &mut self.state {

      // end state
      MountState::Mounted(app_state) => app_state.event(event, event_target),

      // init state
      MountState::Init { event_queue, .. } => match &event {

        PlatformEvent::Resumed => {

          event_queue.push(event);

          take_mut::take(&mut self.state, |state| {
            if let MountState::Init { event_queue, window_builder, init_data, event_loop_proxy } = state {

              let window = crate::window(event_target, window_builder);
              mount_window(&window);

              MountState::Window { window, event_queue, init_data, event_loop_proxy }
            }
            else { unreachable!() }
          });

        },

        _ => event_queue.push(event),
      },

      // after window creation
      MountState::Window { event_queue, window, .. } => match &event {

        PlatformEvent::WindowEvent { event: WindowEvent::Resized {..}, window_id: id } if *id == window.id() => {

          event_queue.push(event);

          take_mut::take(&mut self.state, |state| {
            if let MountState::Window { event_queue, window, init_data, event_loop_proxy } = state {

              let (sender, receiver) = sync_channel(1);
              let window_id = window.id();

              spawn_local(async move {
                let mut app_ctx = AppCtx::new(event_loop_proxy.clone(), window);
                let app = App::init(&mut app_ctx, init_data).await;
                let app_state = AppState::new(app_ctx, app);
                sender.send(app_state).unwrap();
                event_loop_proxy.send_event(PlatformEventExt::AppInit {window_id}).unwrap();
              });

              MountState::Mounting { event_queue, window_id, receiver }
            }
            else { unreachable!() }
          });
        },

        _ => event_queue.push(event),
      },

      // waiting for the app
      MountState::Mounting { event_queue, receiver, window_id } => match &event {

        PlatformEvent::UserEvent(PlatformEventExt::AppInit {window_id: id}) if id == window_id => {

          if let Ok(mut app_state) = receiver.try_recv() {

            for event in event_queue.drain(..) {
              app_state.event(event, event_target);
            }

            self.state = MountState::Mounted(app_state);
          }
        },

        _ => event_queue.push(event),

      },

    }
  }
}