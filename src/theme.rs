#![allow(dead_code)]
use iced_wgpu::Color;
use iced_winit::widget::button;

pub struct Theme {
    pub color: Color,
}
impl button::StyleSheet for Theme {
    type Style = iced_wgpu::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced_winit::Background::Color(self.color)),
            ..Default::default()
        }
    }
}
