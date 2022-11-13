
// extend wgx color
pub trait IntoIcedColor {
    fn iced(self) -> iced_wgpu::Color;
}

impl IntoIcedColor for wgx::Color {
    fn iced(self) -> iced_wgpu::Color {
        iced_wgpu::Color { r: self.r, g: self.g, b: self.b, a: self.a }
    }
}