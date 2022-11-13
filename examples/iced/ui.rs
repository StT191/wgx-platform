
use iced_wgpu::Renderer;
use iced_native::{Alignment, Command, Element, Length, Program};
use iced_native::widget::{Column, Row, Text, TextInput, Slider};
use platform::iced::*;
use wgx::{Color};


#[cfg(not(target_family = "wasm"))]
use iced_graphics::{widget::{Canvas, canvas::{self, Cursor, Geometry, Frame, Path, event::Status}}, Rectangle};


// gui


#[derive(Debug, Clone)]
pub enum Message {
    Color(Color),
    Text(String),
}

pub struct Ui {
    pub color: Color,
    text: String,
}

impl Ui {
    pub fn new() -> Ui {
        Ui { color: Color::from([0.46, 0.60, 0.46]), text: "".to_string() }
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


impl Program for Ui {
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