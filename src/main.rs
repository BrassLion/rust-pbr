mod graphics;

struct ExampleRenderer {
    camera: graphics::Camera,
    points_renderer: graphics::PointsRenderer,
}

impl graphics::RenderSystem for ExampleRenderer {
    fn init(sc_desc: &wgpu::SwapChainDescriptor, device: &wgpu::Device) -> Self {
        let camera_position = nalgebra::Point3::new(1.0, 0.0, 1.0);
        let camera_target = nalgebra::Point3::new(0.0, 0.0, 0.0);
        let camera_up = nalgebra::Vector3::y_axis();

        let camera = graphics::Camera::new(
            &device,
            &camera_position,
            &camera_target,
            &camera_up,
            1920.0 / 1080.0,
            std::f32::consts::PI / 180.0 * 100.0,
            0.1,
            1000.0,
        );

        let vertices = &[
            graphics::Vertex {
                position: [-0.5, -0.5, -0.5],
            },
            graphics::Vertex {
                position: [-0.5, 0.5, -0.5],
            },
            graphics::Vertex {
                position: [0.5, 0.5, -0.5],
            },
            graphics::Vertex {
                position: [-0.5, -0.5, -0.5],
            },
            graphics::Vertex {
                position: [0.5, 0.5, -0.5],
            },
            graphics::Vertex {
                position: [0.5, -0.5, -0.5],
            },
            graphics::Vertex {
                position: [-0.5, -0.5, 0.5],
            },
            graphics::Vertex {
                position: [-0.5, 0.5, 0.5],
            },
            graphics::Vertex {
                position: [0.5, 0.5, 0.5],
            },
            graphics::Vertex {
                position: [-0.5, -0.5, 0.5],
            },
            graphics::Vertex {
                position: [0.5, 0.5, 0.5],
            },
            graphics::Vertex {
                position: [0.5, -0.5, 0.5],
            },
        ];

        let points_renderer = graphics::PointsRenderer::new(&camera, sc_desc, device, vertices);

        Self {
            camera,
            points_renderer,
        }
    }

    fn resize(&mut self, _sc_desc: &wgpu::SwapChainDescriptor, _device: &wgpu::Device) {}

    fn handle_event(&mut self, window: &winit::window::Window, event: &winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::CursorMoved { .. }
            | winit::event::WindowEvent::MouseInput { .. }
            | winit::event::WindowEvent::MouseWheel { .. } => {
                self.camera.handle_event(window, event)
            }
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

        self.camera.update(&device, &mut encoder);

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

            self.points_renderer.draw(&self.camera, &mut render_pass);
        }

        encoder.finish()
    }
}

fn main() {
    graphics::run::<ExampleRenderer>();
}
