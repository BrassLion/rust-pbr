use super::*;

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

pub fn create_uniform_buffer<T>(
    device: &wgpu::Device,
    visibility: wgpu::ShaderStage,
) -> (wgpu::Buffer, wgpu::BindGroup, wgpu::BindGroupLayout) {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: std::mem::size_of::<T>() as u64,
        usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility,
            ty: wgpu::BindingType::UniformBuffer { dynamic: false },
        }],
        label: None,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        bindings: &[wgpu::Binding {
            binding: 0,
            resource: wgpu::BindingResource::Buffer {
                buffer: &buffer,
                range: 0..std::mem::size_of::<T>() as wgpu::BufferAddress,
            },
        }],
        label: None,
    });

    (buffer, bind_group, bind_group_layout)
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

pub fn create_texture_bind_group(
    device: &wgpu::Device,
    visibility: wgpu::ShaderStage,
    textures: &[&Texture],
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        bindings: textures
            .iter()
            .enumerate()
            .flat_map(|(i, tex)| {
                std::iter::once(wgpu::BindGroupLayoutEntry {
                    binding: (2 * i) as u32,
                    visibility,
                    ty: wgpu::BindingType::SampledTexture {
                        dimension: tex.dimension,
                        component_type: wgpu::TextureComponentType::Float,
                        multisampled: false,
                    },
                })
                .chain(std::iter::once(wgpu::BindGroupLayoutEntry {
                    binding: (2 * i + 1) as u32,
                    visibility,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                }))
            })
            .collect::<Vec<wgpu::BindGroupLayoutEntry>>()
            .as_slice(),
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        bindings: textures
            .iter()
            .enumerate()
            .flat_map(|(i, tex)| {
                std::iter::once(wgpu::Binding {
                    binding: (2 * i) as u32,
                    resource: wgpu::BindingResource::TextureView(&tex.view),
                })
                .chain(std::iter::once(wgpu::Binding {
                    binding: (2 * i + 1) as u32,
                    resource: wgpu::BindingResource::Sampler(&tex.sampler),
                }))
            })
            .collect::<Vec<wgpu::Binding>>()
            .as_slice(),
    });

    (bind_group_layout, bind_group)
}
