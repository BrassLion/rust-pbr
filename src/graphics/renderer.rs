use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::graphics::*;

pub trait Renderer: 'static + Sized {
    fn init(sc_desc: &wgpu::SwapChainDescriptor, device: &wgpu::Device) -> Self;

    fn resize(&mut self, sc_desc: &wgpu::SwapChainDescriptor, device: &wgpu::Device);

    fn update(&mut self, event: &WindowEvent);
    fn render(
        &mut self,
        frame: &wgpu::SwapChainOutput,
        device: &wgpu::Device,
    ) -> wgpu::CommandBuffer;
}

pub fn run<R: Renderer>() {
    // Init window.
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Init render state.
    let mut render_state = futures::executor::block_on(render_state::RenderState::new(&window));

    let mut renderer = R::init(&render_state.swap_chain_desc, &render_state.device);

    // Run event loop.
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::RedrawRequested(_) => {
                let frame = render_state
                    .swap_chain
                    .get_next_texture()
                    .expect("Timeout getting texture");

                let command_buffer = renderer.render(&frame, &render_state.device);

                render_state.queue.submit(&[command_buffer]);
            }
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() =>
            /*if !state.input(event)*/
            {
                match event {
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
                    WindowEvent::Resized(physical_size) => render_state.resize(*physical_size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        render_state.resize(**new_inner_size)
                    }
                    _ => renderer.update(event),
                }
            }
            _ => {}
        }
    });
}
