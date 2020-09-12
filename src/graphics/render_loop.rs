use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub trait RenderLoopEvent: 'static + Sized {
    fn init(window: &Window) -> Self;

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);

    fn handle_event(&mut self, window: &Window, event: &WindowEvent);

    fn render(&mut self);
}

pub fn run<R: RenderLoopEvent>() {
    // Init window.
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Rust PBR Example")
        .with_inner_size(winit::dpi::PhysicalSize::new(1920, 1080))
        .build(&event_loop)
        .unwrap();

    let mut render_system = R::init(&window);

    // Run event loop.
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::RedrawRequested(_) => {
                render_system.render();
            }
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => render_system.resize(*physical_size),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    render_system.resize(**new_inner_size)
                }
                _ => render_system.handle_event(&window, event),
            },
            _ => {}
        }
    });
}
