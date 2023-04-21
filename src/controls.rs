use iced_wgpu::Color;
use iced_winit::{
    alignment, column, row,
    widget::{button, container, pick_list},
    Command, Length, Program,
};

#[derive(Default, Clone, Debug, PartialEq, Eq)]
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
            Fractals::Mandelbrot => write!(f, "Mandelbrot set"),
            Fractals::BurningShip => write!(f, "Burning ship"),
            Fractals::Tricorn => write!(f, "Tricorn"),
            Fractals::Feather => write!(f, "Feather"),
            Fractals::Eye => write!(f, "Eye"),
        }
    }
}

#[derive(Default, Clone)]
pub struct Controls {
    ui_open: bool,
    pub current_fractal: Fractals,
    pub colors: Vec<Color>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleUi,
    ChangeFractal(Fractals),
}

fn color_raw(color: &Color) -> Vec<f32> {
    vec![color.r, color.g, color.b, color.a]
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            //The trans flag colors uwu 🏳️‍⚧️
            colors: vec![
                Color::from_rgba(85.0 / 255.0, 205.0 / 255.0, 252.0 / 255.0, 1.0),
                Color::from_rgba(247.0 / 255.0, 168.0 / 255.0, 184.0 / 255.0, 1.0),
                Color::from_rgba(1.0, 1.0, 1.0, 1.0),
                Color::from_rgba(247.0 / 255.0, 168.0 / 255.0, 184.0 / 255.0, 1.0),
                Color::from_rgba(85.0 / 255.0, 205.0 / 255.0, 252.0 / 255.0, 1.0),
            ],
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
            let fractal_list = pick_list(
                fractals,
                Some(self.current_fractal.clone()),
                Message::ChangeFractal,
            );
            row![column![close_button, fractal_list]]
        };
        container(content)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Right)
            .into()
    }
}
