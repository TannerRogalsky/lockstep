use glutin::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let size = glutin::dpi::PhysicalSize::new(720, 480);
    let wb = WindowBuilder::new().with_inner_size(size);
    let window = glutin::ContextBuilder::new()
        .with_multisampling(16)
        .build_windowed(wb, &event_loop)
        .unwrap();
    let window = unsafe { window.make_current().unwrap() };

    let glow_ctx = unsafe {
        graphics::glow::Context::from_loader_function(|name| window.get_proc_address(name))
    };
    let context = graphics::Context::new(glow_ctx);
    let mut renderer = renderer::State::new(context, size.width, size.height).unwrap();

    let mut state = shared::State::new();
    state
        .simulation
        .add_body(shared::nbody::Body::new_lossy(0., 0., 100.));
    state
        .simulation
        .add_body(shared::nbody::Body::new_lossy(300., 300., 100.));

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
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {}
                },
                WindowEvent::Resized(new_inner_size) => {
                    renderer.resize(*new_inner_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    renderer.resize(**new_inner_size);
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
