use iced::Size;
use iced_wgpu::Viewport;
use iced_winit::Clipboard;
use winit::{event_loop::EventLoop, dpi::PhysicalPosition, event::ModifiersState};

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    let physical_size = window.inner_size();
    let mut viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        window.scale_factor(),
    );
    let mut cursor_position = PhysicalPosition::new(-1.0,-1.0);
    let mut modifiers = ModifiersState::default();
    let mut clipboard = Clipboard::connect(&window);

    let default_backend = wgpu::Backends::PRIMARY;

    let backend = wgpu::util::backend_bits_from_env();
}
