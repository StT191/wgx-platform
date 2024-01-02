
#![allow(unused_imports)]

use iced_wgpu::Renderer;

use iced_winit::{
    runtime::{Command, Program},
    core::{Alignment, Element, Length, Rectangle, mouse::Cursor},
    style::{theme::{Theme, Custom, Palette}},
};

use platform::iced::{*};
use wgx::{Color};


use iced_widget::{
    Canvas, canvas::{self, Geometry, Frame, Path, event::Status, Renderer as CanvasRenderer},
    Column, Row, Text, TextInput, Slider,
};


// gui

pub fn theme() -> Theme {
    Theme::Custom(Custom::new(Palette {
        background: Color::WHITE.iced_core(),
        text: Color::BLACK.iced_core(),
        primary: Color::from([0.7; 3]).iced_core(),
        success: Color::GREEN.iced_core(),
        danger: Color::RED.iced_core(),
    }).into())
}


#[derive(Debug, Clone)]
pub enum Msg {
    BgColor(Color),
    Text(String),
}

pub struct Ui {
    pub bg_color: Color,
    text: String,
}

impl Ui {
    pub fn new() -> Ui {
        Ui { bg_color: Color::from([0.46, 0.60, 0.46]), text: "".to_string() }
    }
}


/*struct Circle(f32);

impl<R: CanvasRenderer> canvas::Program<Msg, R> for Circle {

    type State = Color;

    fn draw(&self, state: &Color, renderer: &R, _theme: &R::Theme, bounds: Rectangle, _cursor: Cursor) -> Vec<R::Geometry>{

        let mut frame = Frame::new(renderer, bounds.size());

        let max_radius = bounds.width.min(bounds.height) / 2.0;

        let circle = Path::circle(frame.center(), self.0 * max_radius);

        frame.fill(&circle, state.iced_core());

        vec![frame.into_geometry()]
    }

    fn update(&self, state: &mut Color, _event: canvas::Event, bounds: Rectangle, cursor: Cursor) -> (Status, Option<Msg>){
        if cursor.is_over(&bounds) {
            *state = Color::GREEN;
            (Status::Captured, None)
        }
        else {
            *state = Color::RED;
            (Status::Ignored, None)
        }
    }
}*/


impl Program for Ui {
    type Renderer = Renderer<Theme>;
    type Message = Msg;

    fn update(&mut self, message: Msg) -> Command<Msg> {
        match message {
            Msg::BgColor(color) => { self.bg_color = color; }
            Msg::Text(text) => { self.text = text; }
            // _ => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<Msg, Renderer<Theme>> {
        let bg_color = self.bg_color;

        let column = Column::new()
            .width(Length::Fill).height(Length::Fill)
            .padding(15).spacing(10)
            .align_items(Alignment::Center)
        ;

        let column = column.push(
            Row::new().spacing(65)
            /*.push(Canvas::new(Circle(bg_color.r)).width(100.0).height(100.0))
            .push(Canvas::new(Circle(bg_color.g)).width(100.0).height(100.0))
            .push(Canvas::new(Circle(bg_color.b)).width(100.0).height(100.0))*/
        );

        column.push(
            Text::new(&self.text)
            .width(Length::Fill).height(Length::Fill)
            .size(18).style(Color::WHITE.iced_core())
        )
        .push(
            TextInput::new("input text", &self.text).size(18).padding(4)
            .on_input(|input| Msg::Text(input))
        )
        .push(
            Text::new("Background color").size(16).style(Color::WHITE.iced_core())
        )
        .push(
            Row::new().width(Length::Fixed(500.0)).spacing(10)
            .push(Slider::new(0.0..=1.0, bg_color.r, move |v| Msg::BgColor(Color {r: v, ..bg_color})).step(0.00390625))
            .push(Slider::new(0.0..=1.0, bg_color.g, move |v| Msg::BgColor(Color {g: v, ..bg_color})).step(0.00390625))
            .push(Slider::new(0.0..=1.0, bg_color.b, move |v| Msg::BgColor(Color {b: v, ..bg_color})).step(0.00390625))
        )
        .push(
            Row::new().width(Length::Fixed(80.0)).push(
                Text::new(format!("#{}", bg_color.hex_rgb())).size(16).style(Color::WHITE.iced_core())
            )
        )
        .into()
    }
}