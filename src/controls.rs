// use iced::Command;
use iced_wgpu;
use iced_winit::{
    widget::{ text, Row, row, container},
    Command, Program, Length,
};

pub struct Controls {}

#[derive(Debug, Clone)]
pub enum Message {}

impl Controls {
    pub fn new() -> Controls {
        Controls {}
    }
}

impl Program for Controls {
    type Renderer = iced_wgpu::Renderer;

    type Message = Message;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> iced_winit::Element<'_, Self::Message, Self::Renderer> {
        let t = text("This is a test").into();
        let content=  row(vec![t]);
        container(content).width(Length::Fill).center_x().into()
    }
}
