
use std::sync::Arc;
use platform::winit::{dpi::{PhysicalSize}, event_loop::ControlFlow, window::{Window, WindowBuilder}, event::*};
use platform::{*, WinitEvent as Event, iced::{*}};
use wgx::{*};
use iced_wgpu::Settings;

const LOG_LEVEL: LogLevel = LogLevel::Warn;


mod ui;

async fn run(event_loop: EventLoop, window: Arc<Window>) {

    const DEPTH_TESTING:bool = false;
    // const ALPHA_BLENDING:bool = false;
    const MSAA:u32 = 1;

    // window setup
    window.set_title("WgFx");

    let PhysicalSize {width, height} = window.inner_size();

    let (gx, surface) = unsafe {Wgx::new(Some(&*window), features!(), limits!())}.await.unwrap();
    let mut target = SurfaceTarget::new(&gx, surface.unwrap(), (width, height), MSAA, DEPTH_TESTING).unwrap();


    // iced gui setup
    #[cfg(not(target_family = "wasm"))] let clipboard = Clipboard::connect(&window);
    #[cfg(target_family = "wasm")] let clipboard = Clipboard::connect(&event_loop);

    let renderer = renderer(&gx, Settings::default(), target.format(), Some(4));

    let mut gui = Gui::new(renderer, ui::Ui::new(), (width, height), &window, clipboard);

    gui.theme = ui::theme();

    let mut frame_timer = StepInterval::from_secs(1.0 / 60.0);
    // let mut frame_counter = IntervalCounter::from_secs(5.0);

    /*#[cfg(target_family = "wasm")]
    let mut clipboard_timer = StepInterval::from_secs(1.0 / 10.0); // max every 100ms*/


    event_loop.run(move |event, _, control_flow| {

        /*#[cfg(target_family = "wasm")]
        if clipboard_timer.advance_if_elapsed() {
            gui.clipboard().fetch();
        }*/

        match event {

            Event::NewEvents(StartCause::ResumeTimeReached {..}) => {
                window.request_redraw(); // request frame
                control_flow.set_wait();
            }

            Event::WindowEvent { event, .. } => {
                match event {

                    WindowEvent::CloseRequested => {
                        control_flow.set_exit();
                    }

                    WindowEvent::Resized(size) => {
                        target.update(&gx, (size.width, size.height));
                        window.request_redraw();
                    }

                    _ => (),
                }

                gui.event(&event);
            }

            #[cfg(target_family = "wasm")]
            Event::UserEvent(EventExt::ClipboardPaste) => {
                gui.paste_from_clipboard();
            }

            Event::MainEventsCleared => {

                let (need_redraw, _cmd) = gui.update();

                gui.update_cursor(&window);

                let advanced = frame_timer.step_if_elapsed() >= 1;

                if need_redraw {
                    if advanced {
                        window.request_redraw();
                        control_flow.set_wait();
                    } else {
                        *control_flow = ControlFlow::WaitUntil(frame_timer.next);
                    }
                }
            }

            Event::RedrawRequested(_) => {

                target.with_encoder_frame(&gx, |encoder, frame| {

                    encoder.render_pass(frame.attachments(Some(gui.program().bg_color), None));

                    gui.draw(&gx, encoder, frame);

                }).expect("frame error");

                // gui.recall_staging_belt();

                // frame_counter.add();
                // if let Some(counted) = frame_counter.count() { println!("{:?}", counted) }
            }

            _ => {}
        }
    });
}

fn main() {
  init(LOG_LEVEL);
  let event_loop = event_loop();
  let window = window(WindowBuilder::new(), &event_loop);
  spawn_local(run(event_loop, window));
}