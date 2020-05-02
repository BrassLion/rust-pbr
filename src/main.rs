extern crate image;

mod graphics;

struct ExampleRenderer {
    render_pipeline: wgpu::RenderPipeline,
}

impl graphics::renderer::Renderer for ExampleRenderer {
    fn init(sc_desc: &wgpu::SwapChainDescriptor, device: &wgpu::Device) -> Self {
        // Init shaders.
        let vs_src = include_str!("graphics/shaders/shader.vert");
        let fs_src = include_str!("graphics/shaders/shader.frag");

        let mut compiler = shaderc::Compiler::new().unwrap();
        let options = shaderc::CompileOptions::new().unwrap();

        let vs_spirv = compiler
            .compile_into_spirv(
                vs_src,
                shaderc::ShaderKind::Vertex,
                "vertex",
                "main",
                Some(&options),
            )
            .unwrap();
        let fs_spirv = compiler
            .compile_into_spirv(
                fs_src,
                shaderc::ShaderKind::Fragment,
                "fragment",
                "main",
                Some(&options),
            )
            .unwrap();

        let vs_buffer = std::io::Cursor::new(vs_spirv.as_binary_u8());
        let fs_buffer = std::io::Cursor::new(fs_spirv.as_binary_u8());

        let vs_data = wgpu::read_spirv(vs_buffer).unwrap();
        let fs_data = wgpu::read_spirv(fs_buffer).unwrap();

        let vs_module = device.create_shader_module(&vs_data);
        let fs_module = device.create_shader_module(&fs_data);

        // // Init pipeline.
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self { render_pipeline }
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        encoder.finish()
    }
}

fn main() {
    let img = image::open("/Users/kudansam/Downloads/3Ddataset/shoe/shoe_9/Shoe_9_A1_H1_S1.bmp")
        .unwrap()
        .to_rgba();
    let img_dimensions = img.dimensions();

    graphics::renderer::run::<ExampleRenderer>();
}
