use iced_wgpu::{wgpu, Viewport};
use iced_winit::{
    conversion, futures, program, renderer,
    winit::{self, window::Window},
    Color, Debug, Size,
};
use once_cell::sync::Lazy;
use scene::Scene;
use std::{
    sync::{
        mpsc::{channel, Receiver},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use winit::{
    dpi::PhysicalPosition,
    event::{Event, ModifiersState, WindowEvent},
    event_loop::ControlFlow,
};

mod controls;
mod scene;
mod theme;

fn lerp<T>(a: T, b: T, t: f32) -> T
where
    T: std::ops::Add<Output = T>
        + std::ops::Mul<f32, Output = T>
        + std::ops::Sub<Output = T>
        + Copy,
{
    a + (b - a) * t
}

fn lerp_arr<T>(a: &[T; 2], b: &[T; 2], t: f32) -> [T; 2]
where
    T: std::ops::Add<Output = T>
        + std::ops::Mul<f32, Output = T>
        + std::ops::Sub<Output = T>
        + Copy,
{
    assert_eq!(a.len(), b.len(), "Arrays must have the same length");

    [lerp(a[0], b[0], t), lerp(a[1], b[1], t)]
}

static RECIEVER: Lazy<Arc<Mutex<Option<Receiver<()>>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));
static WINDOW: Lazy<Arc<Mutex<Option<Window>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
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
                //This fix may not work consistently on all devices, so I need to come up with
                //something better, cause srgb is fucking BS
                //.first()
                .last()
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
    //Assuming the maximum number of colors is 64 the size needs to be 40(uniforms) + 16 * 64 = 1064
    //I'm going to go for 1.5k just in case and cause it's a nicer number
    let mut staging_belt = wgpu::util::StagingBelt::new(1536);

    println!("Format is {:#?}", format);
    let scene = Scene::new(&device, format);
    let controls = controls::Controls::new();

    let mut debug = Debug::new();
    let mut renderer = iced_wgpu::Renderer::new(iced_wgpu::Backend::new(
        &device,
        iced_wgpu::Settings::default(),
        format,
    ));

    let mut state =
        program::State::new(controls, viewport.logical_size(), &mut renderer, &mut debug);

    let mut zoom = 500.0;
    let mut zoom_dst = zoom;
    let mut zooming = false;
    let mut zoom_dst_position = [0.0, 0.0];
    let mut position = [0.0, 0.0];
    let mut position_dst: [f32; 2] = [0.0, 0.0];
    let mut dragging = false;

    //Just to send a ping
    let (tx, rx) = channel::<()>();
    //Prepare data for multithreading
    *RECIEVER.lock().unwrap() = Some(rx);
    *WINDOW.lock().unwrap() = Some(window);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        if dragging {
                            let delta = [
                                (position.x - cursor_position.x) as f32 / zoom,
                                (position.y - cursor_position.y) as f32 / zoom,
                            ];
                            position_dst = [delta[0] + position_dst[0], delta[1] + position_dst[1]];
                        }
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
                    WindowEvent::MouseWheel { delta, .. } => match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => {
                            zoom_dst *= if y < 0.0 { 0.86444 } else { 1.21 };
                            let size = WINDOW.lock().unwrap().as_ref().unwrap().inner_size();
                            zoom_dst_position = [
                                (cursor_position.x as f32 / size.width as f32).mul_add(2.0, -1.0),
                                // / zoom,
                                (cursor_position.y as f32 / size.height as f32).mul_add(-2.0, 1.0),
                                // / zoom,
                            ];
                            zooming = true;
                            //Spawn a thread to request redraws
                            thread::spawn(|| loop {
                                if false {
                                    break;
                                }
                                if RECIEVER
                                    .lock()
                                    .unwrap()
                                    .as_mut()
                                    .unwrap()
                                    .try_recv()
                                    .is_ok()
                                {
                                    return;
                                }
                                WINDOW.lock().unwrap().as_mut().unwrap().request_redraw();
                                thread::sleep(Duration::from_secs_f32(1.0 / 60.0));
                            });
                        }
                        winit::event::MouseScrollDelta::PixelDelta(_) => todo!(),
                    },
                    WindowEvent::MouseInput {
                        button: winit::event::MouseButton::Left,
                        ..
                    } => dragging = !dragging,
                    _ => {}
                }
                if let Some(event) = iced_winit::conversion::window_event(
                    &event,
                    WINDOW.lock().unwrap().as_mut().unwrap().scale_factor(),
                    modifiers,
                ) {
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
                    WINDOW.lock().unwrap().as_mut().unwrap().request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                if resized {
                    let size = WINDOW.lock().unwrap().as_ref().unwrap().inner_size();
                    viewport = Viewport::with_physical_size(
                        Size::new(size.width, size.height),
                        WINDOW.lock().unwrap().as_ref().unwrap().scale_factor(),
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
                        let fp = [
                            zoom_dst_position[0] / zoom - position[0],
                            zoom_dst_position[1] / zoom - position[1],
                        ];

                        if zooming {
                            zoom = lerp(zoom, zoom_dst, 0.2);
                            if f32::abs(zoom_dst - zoom) < 0.05 {
                                zooming = false;
                                tx.send(()).unwrap();
                            }
                        }
                        let pos_delta = [
                            zoom_dst_position[0] / zoom - position[0],
                            zoom_dst_position[1] / zoom - position[1],
                        ];

                        position_dst = [
                            position_dst[0] + pos_delta[0] - fp[0],
                            position_dst[1] + pos_delta[1] - fp[1],
                        ];

                        position = lerp_arr(&position, &position_dst, 0.2);

                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });
                        let program = state.program();

                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        let size = WINDOW.lock().unwrap().as_ref().unwrap().inner_size();

                        let raw_colors = program.get_colors_raw();
                        let raw_data = scene::ShaderDataUniforms {
                            aspect: size.width as f32 / size.height as f32,
                            resolution: [size.width, size.height],
                            num_colors: program.num_colors,
                            arr_len: (raw_colors.len() / 4) as u32,
                            max_iter: program.num_iters,
                            fractal: program.current_fractal as u32
                                | if program.smooth_enabled {
                                    2147483648
                                } else {
                                    0
                                },
                            msaa: program.msaa,
                            zoom,
                            position,
                            ..Default::default()
                        }
                        .to_uniform_data();

                        staging_belt
                            .write_buffer(
                                &mut encoder,
                                &scene.buffer,
                                0,
                                wgpu::BufferSize::new((raw_data.len() * 4) as wgpu::BufferAddress)
                                    .unwrap(),
                                &device,
                            )
                            .copy_from_slice(bytemuck::cast_slice(&raw_data));
                        staging_belt
                            .write_buffer(
                                &mut encoder,
                                &scene.storage_buffer,
                                0,
                                wgpu::BufferSize::new(
                                    (raw_colors.len() * 4) as wgpu::BufferAddress,
                                )
                                .unwrap(),
                                &device,
                            )
                            .copy_from_slice(bytemuck::cast_slice(&raw_colors));

                        {
                            let mut render_pass = scene.clear(&view, &mut encoder);
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
