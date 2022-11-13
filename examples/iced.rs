#![feature(duration_consts_float)]

use platform::winit::{
  dpi::{PhysicalSize}, event_loop::ControlFlow,
  window::Window, event::*,
};

use platform::{self, *, Event, iced::{Clipboard, *}};

use iced_wgpu::Settings;
const LOG_LEVEL: LogLevel = LogLevel::Warn;

use wgx::{*, /*cgmath::**/};


// gui

use iced_wgpu::Renderer;
use iced_native::{
    Alignment, Command, Element, Length, Program,
    widget::{Column, Row, Text, TextInput, Slider}
};

#[cfg(not(target_family = "wasm"))]
use iced_graphics::{widget::{Canvas, canvas::{self, Cursor, Geometry, Frame, Path, event::Status}}, Rectangle};


#[derive(Debug, Clone)]
pub enum Message {
    Color(Color),
    Text(String),
}

pub struct Controls {
    pub color: Color,
    text: String,
}

impl Controls {
    pub fn new() -> Controls {
        Controls { color: Color::from([0.46, 0.60, 0.46]), text: "".to_string() }
    }
}


#[cfg(not(target_family = "wasm"))]
struct Circle(f32);

#[cfg(not(target_family = "wasm"))]
impl<T> canvas::Program<Message, T> for Circle {
    type State = Color;
    fn draw(&self, state: &Color, _theme: &T, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry>{
        let mut frame = Frame::new(bounds.size());
        let circle = Path::circle(frame.center(), self.0);
        frame.fill(&circle, state.iced());
        vec![frame.into_geometry()]
    }
    fn update(&self, state: &mut Color, _event: canvas::Event, bounds: Rectangle, cursor: Cursor) -> (Status, Option<Message>){
        if cursor.is_over(&bounds) {
            *state = Color::GREEN;
            (Status::Captured, None)
        }
        else {
            *state = Color::RED;
            (Status::Ignored, None)
        }
    }
}


impl Program for Controls {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Color(color) => { self.color = color; }
            Message::Text(text) => { self.text = text; }
            // _ => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<Message, Renderer> {
        let color = self.color;

        let column = Column::new()
            .width(Length::Fill).height(Length::Fill)
            .padding(15).spacing(10)
            .align_items(Alignment::Center)
        ;

        #[cfg(not(target_family = "wasm"))]
        let column = column.push(
            Row::new().spacing(65)
            .push(Canvas::new(Circle(color.r * 50.0)))
            .push(Canvas::new(Circle(color.g * 50.0)))
            .push(Canvas::new(Circle(color.b * 50.0)))
        );

        column.push(
            Text::new(&self.text)
            .width(Length::Fill).height(Length::Fill)
            .size(20).style(Color::WHITE.iced())
        )
        .push(
            TextInput::new("input text", &self.text, Message::Text).size(20).padding(4)
        )
        .push(
            Text::new("Background color").style(Color::WHITE.iced())
        )
        .push(
            Row::new().width(Length::Units(500)).spacing(10)
            .push(Slider::new(0.0..=1.0, color.r, move |v| Message::Color(Color {r: v, ..color})).step(0.00390625))
            .push(Slider::new(0.0..=1.0, color.g, move |v| Message::Color(Color {g: v, ..color})).step(0.00390625))
            .push(Slider::new(0.0..=1.0, color.b, move |v| Message::Color(Color {b: v, ..color})).step(0.00390625))
        )
        .push(
            Row::new().width(Length::Units(65)).push(
                Text::new(format!("{}", color.hex_rgb())).size(18).style(Color::WHITE.iced())
            )
        )
        .into()
    }
}

async fn run(window: &'static Window, event_loop: EventLoop) {

    const DEPTH_TESTING:bool = false;
    // const ALPHA_BLENDING:bool = false;
    const MSAA:u32 = 1;

    // window setup
    window.set_title("WgFx");

    let PhysicalSize {width, height} = window.inner_size();

    // wgx setup
    #[cfg(not(target_family = "wasm"))] let limits = Limits::default();
    #[cfg(target_family = "wasm")] let limits = Limits::downlevel_webgl2_defaults();


    let (gx, surface) = Wgx::new(Some(window), Features::empty(), limits).await.unwrap();
    let mut target = SurfaceTarget::new(&gx, surface.unwrap(), (width, height), MSAA, DEPTH_TESTING).unwrap();


    // iced setup
    let renderer = renderer(&gx, Settings::default(), target.format(), Some(4));

    let clipboard = Clipboard::connect(&window);
    let mut gui = Iced::new_with_clipboad(renderer, Controls::new(), (width, height), &window, clipboard);

    let mut frame_timer = timer::StepInterval::from_secs(1.0 / 60.0);
    // let mut frame_counter = timer::IntervalCounter::from_secs(5.0);

    /*#[cfg(target_family = "wasm")]
    let mut clipboard_timer = timer::StepInterval::from_secs(1.0 / 10.0); // max every 100ms*/


    event_loop.run(move |event, _, control_flow| {

        /*#[cfg(target_family = "wasm")]
        if clipboard_timer.advance_if_elapsed() {
            gui.clipboard().fetch();
        }*/

        match event {

            Event::NewEvents(StartCause::ResumeTimeReached {..}) => {
                window.request_redraw(); // request frame
                control_flow.set_wait();
            },

            Event::WindowEvent { event, .. } => {
                match event {

                    WindowEvent::CloseRequested => {
                        control_flow.set_exit();
                    }

                    WindowEvent::Resized(size) => {
                        target.update(&gx, (size.width, size.height));
                        window.request_redraw();
                    }

                    /*WindowEvent::KeyboardInput { input: KeyboardInput {
                        virtual_keycode: Some(keycode), state: ElementState::Pressed, ..
                    }, ..} => {
                        log::warn!("Key: {:?} + {:?}", keycode, gui.modifiers);
                    }

                    WindowEvent::ModifiersChanged(modifiers) => {
                        log::warn!("Modifiers {:?}", modifiers);
                    }*/

                    _ => (),
                }

                // log::warn!("Event {:?}", event);

                gui.event(&event);
            }

            Event::MainEventsCleared => {

                let (need_redraw, _cmd) = gui.update();

                gui.update_cursor(&window);

                let advanced = frame_timer.advance_if_elapsed();

                if need_redraw && *control_flow != ControlFlow::WaitUntil(frame_timer.next) {
                    * control_flow = if advanced {
                        window.request_redraw();
                        ControlFlow::Wait
                    }
                    else { ControlFlow::WaitUntil(frame_timer.next) }
                }
            }

            Event::RedrawRequested(_) => {

                target.with_encoder_frame(&gx, |mut encoder, frame| {

                    encoder.render_pass(frame.attachments(Some(gui.program().color), None));

                    gui.draw(&gx, &mut encoder, frame);

                }).expect("frame error");

                gui.recall_staging_belt();

                // frame_counter.add();
                // if let Some(counted) = frame_counter.count() { println!("{:?}", counted) }
            },

            _ => {}
        }
    });
}

fn main() {
    platform::main(run, LOG_LEVEL);
}