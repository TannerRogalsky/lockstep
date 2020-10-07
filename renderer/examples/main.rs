use glutin::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new().with_inner_size(glutin::dpi::PhysicalSize::new(1280, 720));
    let window = glutin::ContextBuilder::new()
        .with_multisampling(16)
        .with_double_buffer(Some(true))
        .with_vsync(true)
        .build_windowed(wb, &event_loop)
        .unwrap();
    // window
    //     .window()
    //     .set_fullscreen(Some(glutin::window::Fullscreen::Borderless(
    //         window.window().primary_monitor(),
    //     )));
    let window = unsafe { window.make_current().unwrap() };

    let glow_ctx = unsafe {
        solstice::glow::Context::from_loader_function(|name| window.get_proc_address(name))
    };
    let context = solstice::Context::new(glow_ctx);
    let size = window.window().inner_size();
    let mut renderer = renderer::Renderer::new(context, size.width, size.height).unwrap();

    let mut state = shared::State::new();

    state.simulation.add_body(shared::nbody::Body::new_lossy(
        size.width as f32 / 2.,
        size.height as f32 / 2.,
        10000.,
    ));
    state.simulation.add_body({
        let mut body = shared::nbody::Body::new_lossy(
            size.width as f32 / 2.,
            size.height as f32 / 2. - 100.,
            10.,
        );
        body.velocity.x = shared::nbody::Float::from_num(3);
        body
    });
    state.simulation.add_body({
        let mut body = shared::nbody::Body::new_lossy(
            size.width as f32 / 2.,
            size.height as f32 / 2. + 100.,
            10.,
        );
        body.velocity.x = shared::nbody::Float::from_num(-3);
        body
    });

    let mut mouse_down = None;
    let mut mouse_position = glutin::dpi::PhysicalPosition::new(0., 0.);
    let mut mass = 10u32;

    use rand::prelude::*;
    let mut rng = thread_rng();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.window().id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(keycode),
                        ..
                    } => match keycode {
                        VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        VirtualKeyCode::Up => mass = 1_000_000.min(mass * 10),
                        VirtualKeyCode::Down => mass = 10.max(mass / 10),
                        VirtualKeyCode::N => state.step(),
                        VirtualKeyCode::R => state.simulation.bodies.clear(),
                        VirtualKeyCode::P => {
                            let offset = renderer.camera_position();
                            let origin = offset
                                + nalgebra::Vector2::new(
                                    mouse_position.x as f32,
                                    mouse_position.y as f32,
                                );
                            proto_disk(&mut state.simulation, &mut rng, 1000, origin, 400.);
                        }
                        _ => {}
                    },
                    _ => {}
                },
                WindowEvent::CursorMoved { position, .. } => mouse_position = *position,
                WindowEvent::MouseInput {
                    state: mouse_state,
                    button,
                    ..
                } => match (mouse_state, button) {
                    (ElementState::Pressed, MouseButton::Left) => {
                        mouse_down = Some(mouse_position);
                    }
                    (ElementState::Released, MouseButton::Left) => {
                        let offset = renderer.camera_position();
                        let mut body = shared::nbody::Body::new_lossy(
                            offset.x + mouse_position.x as f32,
                            offset.y + mouse_position.y as f32,
                            mass as _,
                        );
                        if let Some(mouse_down) = mouse_down {
                            const VEL_SCALE: f64 = 0.01;
                            let dx = (mouse_position.x - mouse_down.x) * VEL_SCALE;
                            let dy = (mouse_position.y - mouse_down.y) * VEL_SCALE;
                            body.velocity.x = shared::nbody::Float::from_num(dx);
                            body.velocity.y = shared::nbody::Float::from_num(dy);
                        }
                        state.simulation.add_body(body);
                        mouse_down = None;
                    }
                    _ => {}
                },
                WindowEvent::Resized(new_inner_size) => {
                    let glutin::dpi::PhysicalSize { width, height } = *new_inner_size;
                    renderer.resize(width, height);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    let glutin::dpi::PhysicalSize { width, height } = **new_inner_size;
                    renderer.resize(width, height);
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                state.step();
                renderer.render(&state);
                window.swap_buffers().unwrap();
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually request it.
                window.window().request_redraw();
            }
            _ => {}
        }
    });
}

fn proto_disk(
    sim: &mut shared::nbody::Simulation,
    rng: &mut rand::rngs::ThreadRng,
    count: usize,
    origin: nalgebra::Point2<f32>,
    radius: f32,
) {
    use rand::prelude::*;
    for _ in 0..count {
        let rand = rng.gen::<f32>() * 2. * std::f32::consts::PI;
        let rand2 = rng.gen::<f32>();
        let x = (radius * rand2) * rand.cos();
        let y = (radius * rand2) * rand.sin();
        let mag = (x * x + y * y).sqrt();

        let mut body = shared::nbody::Body::new_lossy(origin.x + x, origin.y + y, 1000.);
        body.velocity.x = shared::nbody::Float::from_num(y * (mag / 7000.));
        body.velocity.y = shared::nbody::Float::from_num(-x * (mag / 7000.));
        sim.add_body(body);
    }
}
