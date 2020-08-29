pub trait MaterialBase {
    fn begin_render_pass<'a>(
        &'a self,
        device: &wgpu::Device,
        encoder: &'a mut wgpu::CommandEncoder,
        rp_desc: &'a wgpu::RenderPassDescriptor,
        transform_data: &TransformBindGroup,
        lighting_data: &LightingBindGroup,
    ) -> wgpu::RenderPass<'a>;
}

pub struct TransformBindGroup {
    pub model_matrix: nalgebra::Matrix4<f32>,
    pub view_matrix: nalgebra::Matrix4<f32>,
    pub proj_matrix: nalgebra::Matrix4<f32>,
    pub camera_world_position: nalgebra::Vector3<f32>,
}

pub struct LightingBindGroup {
    pub position: nalgebra::Vector3<f32>,
    pub _padding: u32,
}

pub fn build_render_pipeline(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
    vertex_shader_src: &str,
    fragment_shader_src: &str,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    vertex_state_desc: wgpu::VertexStateDescriptor,
    colour_states: &[wgpu::ColorStateDescriptor],
    depth_state: Option<wgpu::DepthStencilStateDescriptor>,
) -> wgpu::RenderPipeline {
    // Init shaders.
    let mut compiler = shaderc::Compiler::new().unwrap();
    let options = shaderc::CompileOptions::new().unwrap();

    let vs_spirv = compiler
        .compile_into_spirv(
            vertex_shader_src,
            shaderc::ShaderKind::Vertex,
            "vertex",
            "main",
            Some(&options),
        )
        .unwrap();
    let fs_spirv = compiler
        .compile_into_spirv(
            fragment_shader_src,
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

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: bind_group_layouts,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
        color_states: colour_states,
        depth_stencil_state: depth_state,
        vertex_state: vertex_state_desc,
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    })
}

pub fn update_uniform_buffer<T>(
    device: &wgpu::Device,
    uniform_buffer: &wgpu::Buffer,
    encoder: &mut wgpu::CommandEncoder,
    uniform_data: &T,
) {
    // TODO: Replace this with a function.
    let uniform_data_bytes = unsafe {
        let len = std::mem::size_of_val(uniform_data);
        let ptr = (uniform_data as *const _) as *const u8;
        std::slice::from_raw_parts(ptr, len)
    };

    let staging_buffer =
        device.create_buffer_with_data(uniform_data_bytes, wgpu::BufferUsage::COPY_SRC);

    encoder.copy_buffer_to_buffer(
        &staging_buffer,
        0,
        &uniform_buffer,
        0,
        std::mem::size_of::<T>() as wgpu::BufferAddress,
    );
}
