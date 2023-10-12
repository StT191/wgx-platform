
use iced_winit::core as iced_core;

// extend wgx color
pub trait IntoIcedCoreColor {
    fn iced_core(self) -> iced_core::Color;
}

impl IntoIcedCoreColor for wgx::Color {
    fn iced_core(self) -> iced_core::Color {
        iced_core::Color { r: self.r, g: self.g, b: self.b, a: self.a }
    }
}