#![allow(dead_code, unused)]
use iced_wgpu::{wgpu, Backend, Renderer, Settings, Viewport};
use iced_winit::{conversion, futures, program, renderer, winit, Clipboard, Color, Debug, Size};

use ::winit::{
    dpi::PhysicalPosition,
    event::{Event, ModifiersState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    let physical_size = window.inner_size();
    let mut viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        window.scale_factor(),
    );
    let mut cursor_position = PhysicalPosition::new(-1.0, -1.0);
    let mut modifiers = ModifiersState::default();
    // let mut clipboard = Clipboard::connect(&window);

    let default_backend = wgpu::Backends::PRIMARY;

    let backend = wgpu::util::backend_bits_from_env().unwrap_or(default_backend);
    let instance = wgpu::Instance::new(backend);
    //Mom come pick me up I'm scared
    //It's not that bad tho, it's just that the wgpu API is unsafe and could access invalid memory
    //well at least according to ChatGPT
    let surface = unsafe { instance.create_surface(&window) };

    let (format, (device, queue)) = futures::executor::block_on(async {
        let adapter =
            wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
                .await
                .expect("No suitabple GPUs found");
        let adapter_features = adapter.features();

        let needed_limits = wgpu::Limits::default();

        (
            surface
                .get_supported_formats(&adapter)
                .first()
                .copied()
                .expect("Get preferred format"),
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        features: adapter_features & wgpu::Features::default(),
                        limits: needed_limits,
                    },
                    None,
                )
                .await
                .expect("Request device"),
        )
    });

    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        },
    );

    let mut resized = false;

    //A buffer for transferring things to and from the GPU memory
    let mut staging_belt = wgpu::util::StagingBelt::new(5*1024);
}
