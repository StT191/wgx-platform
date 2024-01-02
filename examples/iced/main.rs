
use platform::winit::{dpi::{PhysicalSize}, event_loop::ControlFlow, window::{WindowBuilder}, event::*};
use platform::{*, WinitEvent as Event, iced::{*}};
use wgx::{*};
use iced_wgpu::Settings;

mod ui;

async fn run() {

  const DEPTH_TESTING:bool = false;
  const MSAA:u32 = 1;
  // const ALPHA_BLENDING:Option<Blend> = None;

  let event_loop = event_loop();
  let window = window(WindowBuilder::new(), &event_loop).await;

  // window setup
  window.set_title("WgFx");

  let PhysicalSize {width, height} = window.inner_size();

  let (gx, surface) = unsafe {Wgx::new(Some(&*window), features!(), limits!(max_inter_stage_shader_components: 60))}.await.unwrap();
  let mut target = SurfaceTarget::new(&gx, surface.unwrap(), (width, height), MSAA, DEPTH_TESTING).unwrap();


  // iced gui setup
  #[cfg(not(target_family = "wasm"))] let clipboard = Clipboard::connect(&window);
  #[cfg(target_family = "wasm")] let clipboard = Clipboard::connect(&event_loop);

  let renderer = renderer(&gx, Settings::default(), target.format(), Some(4));

  let mut gui = Gui::new(renderer, ui::Ui::new(), &window, clipboard);

  gui.theme = ui::theme();

  // let mut frame_counter = IntervalCounter::from_secs(5.0);

  /*#[cfg(target_family = "wasm")]
  let mut clipboard_timer = StepInterval::from_secs(1.0 / 10.0); // max every 100ms*/

  const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 24);

  let mut redraw_scheduled = false;
  let mut next = Instant::now() + FRAME_DURATION;


  event_loop.run(move |event, event_target| {

    /*#[cfg(target_family = "wasm")]
    if clipboard_timer.advance_if_elapsed() {
      gui.clipboard().fetch();
    }*/

    match event {

      Event::WindowEvent { event, .. } => {
        match event {

          WindowEvent::CloseRequested => {
            event_target.exit();
          }

          WindowEvent::Resized(size) => {
            target.update(&gx, (size.width, size.height));
            window.request_redraw();
          }

          WindowEvent::RedrawRequested => {

            // frame handling
            next = Instant::now() + FRAME_DURATION;
            redraw_scheduled = false;
            event_target.set_control_flow(ControlFlow::Wait);

            // gui handling
            gui.update();
            gui.update_cursor(&window);

            // draw
            target.with_frame(None, |frame| gx.with_encoder(|encoder| {

              encoder.render_pass(frame.attachments(Some(gui.program().bg_color), None));

              gui.draw(&gx, encoder, frame);

            })).expect("frame error");

            /*frame_counter.add();
            if let Some(counted) = frame_counter.count() { println!("{:?}", counted) }*/

          }

          _ => (),
        }

        let event_was_queued = gui.event(&event);

        // redraw handling
        if !redraw_scheduled && event_was_queued {

          if Instant::now() < next {
            event_target.set_control_flow(ControlFlow::WaitUntil(next));
          } else {
            window.request_redraw();
          }

          redraw_scheduled = true;
        }
      }

      Event::NewEvents(StartCause::ResumeTimeReached {..}) => {
        window.request_redraw();
      }


      #[cfg(target_family = "wasm")]
      Event::UserEvent(EventExt::ClipboardPaste) => {
        gui.paste_from_clipboard();
      }

      _ => {}
    }

  }).unwrap();
}

fn main() {
  init(LogLevel::Warn);
  spawn_local(run());
}