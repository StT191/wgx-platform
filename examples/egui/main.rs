
use platform::winit::{dpi::{PhysicalSize}, event_loop::ControlFlow, window::{WindowBuilder}, event::*};
use platform::{*, WinitEvent as Event};
use wgx::{*};

use platform::egui::{*};

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


  // egui setup

  let mut ui = ui::new();
  let mut renderer = renderer(&gx, &target);
  let mut egs = EguiCtx::new(&window);

  // run to initialize
  egs.context.set_fonts(FontDefinitions::default());
  egs.run(&window, |_ctx| {});


  let circle = Shape::circle_filled([100.0, 100.0].into(), 80.0, Color32::from_rgb(0x00, 0xF0, 0xF0));

  let text = egs.context.fonts(|fonts| {
    Shape::text(
      fonts, [200.0, 200.0].into(), Align2::LEFT_CENTER,
      "HALLO TEST Hallo Test!",
      FontId { size: 14.0, family: FontFamily::default()},
      Color32::from_rgb(0xFF, 0xFF, 0xFF),
    )
  });

  let cps = egs.context.tessellate(
    clip_shapes([circle, text], egs.context.screen_rect()).collect(),
    egs.screen_dsc.pixels_per_point,
  );


  // let mut frame_counter = IntervalCounter::from_secs(3.0);

  const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);

  let mut redraw_scheduled = false;
  let mut next = Instant::now() + FRAME_DURATION;

  event_loop.run(move |event, event_target| {

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

            // record last frame time
            let last = Instant::now();


            // gui handling
            let mut output = egs.run(&window, &mut ui);

            output.clipped_primitives.extend_from_slice(&cps);

            // draw
            target.with_frame(None, |frame| gx.with_encoder(|encoder| {

              output.prepare(&mut renderer, &gx, encoder);

              output.render(
                &mut renderer,
                &mut encoder.render_pass(frame.attachments(Some(Color::WHITE.into()), None))
              );

            })).expect("frame error");


            // frame timing
            next = last + FRAME_DURATION;
            redraw_scheduled = false;

            if output.repaint_delay < FRAME_DURATION {
              // reschedule frame because we're animating
              event_target.set_control_flow(ControlFlow::WaitUntil(next));
              redraw_scheduled = true;
            }
            else {
              if let Some(instant) = last.checked_add(output.repaint_delay) {
                event_target.set_control_flow(ControlFlow::WaitUntil(instant));
              }
              else {
                event_target.set_control_flow(ControlFlow::Wait);
              }
            }

            for command in output.commands {
              println!("Cmd: {:#?}", command);
              if command == ViewportCommand::Close {
                event_target.exit();
              }
            }

            /*frame_counter.add();
            if let Some(counted) = frame_counter.count() { println!("{:?}", counted) }*/

          }

          _ => (),
        }

        let (repaint, _) = egs.event(&window, &event);

        // redraw handling
        if !redraw_scheduled && repaint {

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


      _ => {}
    }

  }).unwrap();
}

fn main() {
  init(LogLevel::Warn);
  spawn_local(run());
}