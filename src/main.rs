#![allow(dead_code, unused)]
use controls::Controls;
use iced_wgpu::{wgpu, Backend, Renderer, Settings, Viewport};
use iced_winit::{conversion, futures, program, renderer, winit, Clipboard, Color, Debug, Size};
use scene::Scene;
use winit::{
    dpi::PhysicalPosition,
    event::{Event, ModifiersState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod controls;
mod scene;

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
    let mut clipboard = iced_winit::Clipboard::connect(&window);

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
    let mut staging_belt = wgpu::util::StagingBelt::new(5 * 1024);

    let scene = Scene::new(&device, format);
    let controls = Controls::new();

    let mut debug = Debug::new();
    let mut renderer = Renderer::new(Backend::new(&device, Settings::default(), format));

    let mut state =
        program::State::new(controls, viewport.logical_size(), &mut renderer, &mut debug);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Wait;

        match event {
            Event::WindowEvent { window_id, event } => {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        cursor_position = position;
                    }

                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = new_modifiers;
                    }
                    WindowEvent::Resized(_) => {
                        resized = true;
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
                if let Some(event) =
                    iced_winit::conversion::window_event(&event, window.scale_factor(), modifiers)
                {
                    state.queue_event(event);
                }
            }
            Event::MainEventsCleared => {
                if !state.is_queue_empty() {
                    let _ = state.update(
                        viewport.logical_size(),
                        conversion::cursor_position(cursor_position, viewport.scale_factor()),
                        &mut renderer,
                        &iced_wgpu::Theme::Dark,
                        &renderer::Style {
                            text_color: Color::WHITE,
                        },
                        &mut clipboard,
                        &mut debug,
                    );
                    window.request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                if resized {
                    let size = window.inner_size();
                    viewport = Viewport::with_physical_size(
                        Size::new(size.width, size.height),
                        window.scale_factor(),
                    );
                    surface.configure(
                        &device,
                        &wgpu::SurfaceConfiguration {
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            format,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::AutoVsync,
                            alpha_mode: wgpu::CompositeAlphaMode::Auto,
                        },
                    );
                    resized = false;
                }
                match surface.get_current_texture() {
                    Ok(frame) => {
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });
                        let program = state.program();

                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        let size = window.inner_size();
                        let data = scene::ShaderDataUniforms {
                            aspect: size.width as f32 / size.height as f32,
                            ..Default::default()
                        }
                        .to_uniform_data();
                        staging_belt
                            .write_buffer(
                                &mut encoder,
                                &scene.buffer,
                                0,
                                wgpu::BufferSize::new((data.len() * 4) as wgpu::BufferAddress)
                                    .unwrap(),
                                &device,
                            )
                            .copy_from_slice(bytemuck::cast_slice(&data));

                        let colors: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
                        staging_belt
                            .write_buffer(
                                &mut encoder,
                                &scene.storage_buffer,
                                0,
                                wgpu::BufferSize::new((colors.len() * 4) as wgpu::BufferAddress)
                                    .unwrap(),
                                &device,
                            )
                            .copy_from_slice(bytemuck::cast_slice(&colors));

                        {
                            let mut render_pass = scene.clear(&view, &mut encoder, Color::WHITE);
                            render_pass.set_bind_group(0, &scene.bind_group, &[]);
                            scene.draw(&mut render_pass);
                        }

                        renderer.with_primitives(|backend, primitive| {
                            backend.present(
                                &device,
                                &mut staging_belt,
                                &mut encoder,
                                &view,
                                primitive,
                                &viewport,
                                &debug.overlay(),
                            );
                        });
                        staging_belt.finish();
                        queue.submit(Some(encoder.finish()));
                        frame.present();
                        staging_belt.recall();
                    }
                    Err(_) => todo!(),
                }
            }

            _ => {}
        }
    })
}
