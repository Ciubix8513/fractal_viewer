use std::ops::RangeInclusive;

use iced_wgpu::Color;
use iced_winit::{
    alignment, column, row,
    widget::{button, checkbox, column, container, pick_list, slider, text},
    Command, Length, Program, Widget,
};

#[derive(Default, Clone, Debug, PartialEq, Eq, Copy)]
pub enum Fractals {
    #[default]
    Mandelbrot = 1,
    BurningShip = 2,
    Tricorn = 4,
    Feather = 8,
    Eye = 16,
}

impl std::fmt::Display for Fractals {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mandelbrot => write!(f, "Mandelbrot set"),
            Self::BurningShip => write!(f, "Burning ship"),
            Self::Tricorn => write!(f, "Tricorn"),
            Self::Feather => write!(f, "Feather"),
            Self::Eye => write!(f, "Eye"),
        }
    }
}

#[derive(Default, Clone)]
pub struct Controls {
    ui_open: bool,
    pub current_fractal: Fractals,
    pub colors: Vec<Color>,
    pub num_iters: u32,
    pub num_colors: u32,
    pub smooth_enabled: bool,
    pub msaa: u32,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleUi,
    ChangeFractal(Fractals),
    NumColorsChanged(u32),
    NumItersChanged(u32),
    ToggleSmooth(bool),
    MsaaChanged(u32),
    ColorRemove(usize),
    OpenColorPicker(usize),
    ColorAdd,
}

fn color_raw(color: &Color) -> Vec<f32> {
    vec![color.r, color.g, color.b, color.a]
}

impl Controls {
    pub fn new() -> Self {
        Self {
            //The trans flag colors uwu 🏳️‍⚧️
            colors: vec![
                Color::from_rgba(85.0 / 255.0, 205.0 / 255.0, 252.0 / 255.0, 1.0),
                Color::from_rgba(247.0 / 255.0, 168.0 / 255.0, 184.0 / 255.0, 1.0),
                Color::from_rgba(1.0, 1.0, 1.0, 1.0),
                Color::from_rgba(247.0 / 255.0, 168.0 / 255.0, 184.0 / 255.0, 1.0),
                Color::from_rgba(85.0 / 255.0, 205.0 / 255.0, 252.0 / 255.0, 1.0),
            ],
            num_iters: 1000,
            num_colors: 200,
            msaa: 1,
            ..Default::default()
        }
    }

    pub fn get_colors_raw(&self) -> Vec<f32> {
        self.colors.clone().iter().flat_map(color_raw).collect()
    }
}

impl Program for Controls {
    type Renderer = iced_wgpu::Renderer;

    type Message = Message;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::ToggleUi => self.ui_open = !self.ui_open,
            Message::ChangeFractal(f) => self.current_fractal = f,
            Message::NumColorsChanged(value) => self.num_colors = value,
            Message::NumItersChanged(value) => self.num_iters = value,
            Message::ToggleSmooth(value) => self.smooth_enabled = value,
            Message::MsaaChanged(value) => self.msaa = value,
            Message::ColorRemove(index) => _ = self.colors.remove(index),
            Message::ColorAdd => self.colors.push(Color::from_rgb(1.0, 1.0, 1.0)),
            Message::OpenColorPicker(_) => todo!(),
        }
        Command::none()
    }

    fn view(&self) -> iced_winit::Element<'_, Self::Message, Self::Renderer> {
        let content = if !self.ui_open {
            let open_button = button("Open UI").on_press(Message::ToggleUi);
            row![open_button].padding(10)
        } else {
            let close_button = button("Close").on_press(Message::ToggleUi);
            let fractals = vec![
                Fractals::Mandelbrot,
                Fractals::BurningShip,
                Fractals::Tricorn,
                Fractals::Feather,
                Fractals::Eye,
            ];
            let fractal_list =
                pick_list(fractals, Some(self.current_fractal), Message::ChangeFractal);
            let num_colors_slider = slider(
                RangeInclusive::new(1, 1000),
                self.num_colors,
                Message::NumColorsChanged,
            );
            let num_iters_slider = slider(
                RangeInclusive::new(1, 2000),
                self.num_iters,
                Message::NumItersChanged,
            );
            let msaa_slider = slider(RangeInclusive::new(1, 8), self.msaa, Message::MsaaChanged);

            let num_colors_label = text("Num colors");
            let num_iters_label = text("Num iters");
            let msaa_label = text("Anti Aliasing");

            let smooth_toggle = checkbox("Smooth?", self.smooth_enabled, Message::ToggleSmooth);

            let colors_label = row![
                text("Colors"),
                button(text("+").horizontal_alignment(alignment::Horizontal::Center))
                    .height(30)
                    .width(30)
                    .on_press(Message::ColorAdd)
            ]
            .spacing(5);
            let colors = column(
                self.colors
                    .iter()
                    .zip(0..self.colors.len())
                    .map(|t| {
                        row![
                            button("")
                                .on_press(Message::OpenColorPicker(t.1))
                                .width(30)
                                .height(30),
                            button(text("X").horizontal_alignment(alignment::Horizontal::Center))
                                .on_press(Message::ColorRemove(t.1))
                                .width(30)
                                .height(30),
                        ]
                        .spacing(20)
                        .into()
                    })
                    .collect(),
            )
            .spacing(10);

            row![column![
                close_button,
                fractal_list,
                num_iters_label,
                num_iters_slider,
                num_colors_label,
                num_colors_slider,
                msaa_label,
                msaa_slider,
                smooth_toggle,
                colors_label,
                colors
            ]
            .spacing(10)]
            .spacing(20)
            .width(200)
            .padding(10)
        };
        container(content)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Right)
            .into()
    }
}
