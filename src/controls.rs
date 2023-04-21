use iced_winit::{
    alignment, column, row,
    widget::{button, container, pick_list},
    Command, Length, Program,
};

#[derive(Default, Clone, Debug,PartialEq,Eq)]
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
            Fractals::Mandelbrot => write!(f,"Mandelbrot set"),
            Fractals::BurningShip => write!(f,"Burning ship"),
            Fractals::Tricorn => write!(f,"Tricorn"),
            Fractals::Feather => write!(f,"Feather"),
            Fractals::Eye => write!(f,"Eye"),
        }
    }
}

#[derive(Default)]
pub struct Controls {
    ui_open: bool,
    current_fractal: Fractals,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleUi,
    ChangeFractal(Fractals),
}

impl Controls {
    pub fn new() -> Controls {
        Controls::default()
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
            let fractals = vec![Fractals::Mandelbrot,Fractals::BurningShip,Fractals::Tricorn,Fractals::Feather,Fractals::Eye];
            let fractal_list =
                pick_list(fractals,Some(self.current_fractal.clone()) , Message::ChangeFractal);
            row![column![close_button,fractal_list]]
        };
        container(content)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Right)
            .into()
    }
}
