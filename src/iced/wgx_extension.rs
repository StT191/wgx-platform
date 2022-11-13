
// extend wgx color
pub trait IntoIcedWgpuColor {
    fn iced_wgpu(self) -> iced_wgpu::Color;
}

impl IntoIcedWgpuColor for wgx::Color {
    fn iced_wgpu(self) -> iced_wgpu::Color {
        iced_wgpu::Color { r: self.r, g: self.g, b: self.b, a: self.a }
    }
}

pub trait IntoIcedNativeColor {
    fn iced_native(self) -> iced_native::Color;
}

impl IntoIcedNativeColor for wgx::Color {
    fn iced_native(self) -> iced_native::Color {
        iced_wgpu::Color { r: self.r, g: self.g, b: self.b, a: self.a }
    }
}