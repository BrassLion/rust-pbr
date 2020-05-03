extern crate image;

mod graphics;

struct ExampleRenderer {
    points_renderer: graphics::renderer_points::PointsRenderer,
}

impl graphics::renderer_system::RenderSystem for ExampleRenderer {
    fn init(sc_desc: &wgpu::SwapChainDescriptor, device: &wgpu::Device) -> Self {
        let vertices = &[
            graphics::renderer_points::Vertex {
                position: [0.0, 0.5, 0.0],
            },
            graphics::renderer_points::Vertex {
                position: [-0.5, -0.5, 0.0],
            },
            graphics::renderer_points::Vertex {
                position: [0.5, -0.5, 0.0],
            },
        ];

        let points_renderer =
            graphics::renderer_points::PointsRenderer::new(sc_desc, device, vertices);

        Self { points_renderer }
    }

    fn resize(&mut self, _sc_desc: &wgpu::SwapChainDescriptor, _device: &wgpu::Device) {}

    fn update(&mut self, event: &winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => println!("{:?}", input),
            _ => {}
        };
    }

    fn render(
        &mut self,
        frame: &wgpu::SwapChainOutput,
        device: &wgpu::Device,
    ) -> wgpu::CommandBuffer {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::BLACK,
                }],
                depth_stencil_attachment: None,
            });

            self.points_renderer.draw(&mut render_pass);
        }

        encoder.finish()
    }
}

fn main() {
    let img = image::open("/Users/kudansam/Downloads/3Ddataset/shoe/shoe_9/Shoe_9_A1_H1_S1.bmp")
        .unwrap()
        .to_rgba();
    let img_dimensions = img.dimensions();

    graphics::renderer_system::run::<ExampleRenderer>();
}
