use super::*;

pub struct PointsRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,

    num_vertices: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl PointsRenderer {
    pub fn new(
        camera: &Camera,
        swap_chain_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        vertex_data: &[Vertex],
    ) -> Self {
        // Init shaders.
        let vs_src = include_str!("shaders/shader.vert");
        let fs_src = include_str!("shaders/shader.frag");

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

        // Upload vertex data.
        let vertex_data_bytes = unsafe {
            let len = vertex_data.len() * std::mem::size_of::<Vertex>();
            let ptr = vertex_data.as_ptr() as *const u8;
            std::slice::from_raw_parts(ptr, len)
        };

        let vertex_buffer =
            device.create_buffer_with_data(vertex_data_bytes, wgpu::BufferUsage::VERTEX);

        // Init pipeline.
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&camera.uniform_bind_group_layout],
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
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: swap_chain_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[wgpu::VertexAttributeDescriptor {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float3,
                    }],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            num_vertices: vertex_data.len() as u32,
        }
    }

    pub fn draw<'a>(&'a self, camera: &'a Camera, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        render_pass.set_bind_group(0, &camera.uniform_bind_group, &[]);
        render_pass.draw(0..self.num_vertices, 0..1);
    }
}
