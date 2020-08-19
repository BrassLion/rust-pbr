use super::*;

pub struct PbrBindGroup {
    pub ao_texture: Texture,
    pub albedo_texture: Texture,
    pub emissive_texture: Texture,
    pub metal_roughness_texture: Texture,
    pub normal_texture: Texture,
}

pub struct TransformBindGroup {
    pub model_matrix: nalgebra::Matrix4<f32>,
    pub view_proj_matrix: nalgebra::Matrix4<f32>,
    pub camera_world_position: nalgebra::Vector3<f32>,
}

pub struct LightingBindGroup {
    pub position: nalgebra::Vector3<f32>,
    pub _padding: u32,
}

pub struct Material {
    pub render_pipeline: wgpu::RenderPipeline,
    pub params_bind_group: wgpu::BindGroup,
    pub transform_bind_group: wgpu::BindGroup,
    pub transform_bind_group_buffer: wgpu::Buffer,
    pub lighting_bind_group: wgpu::BindGroup,
    pub lighting_bind_group_buffer: wgpu::Buffer,
}

impl Material {
    pub fn new(
        device: &wgpu::Device,
        swap_chain_desc: &wgpu::SwapChainDescriptor,
        params: &PbrBindGroup,
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

        // Init bind groups.

        // Transform buffers.
        let transform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<TransformBindGroup>() as u64,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let transform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
                label: Some("transform_bind_group_layout"),
            });

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &transform_bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &transform_buffer,
                    // FYI: you can share a single buffer between bindings.
                    range: 0..std::mem::size_of::<TransformBindGroup>() as wgpu::BufferAddress,
                },
            }],
            label: Some("transform_bind_group"),
        });

        // Material bind group.
        let params_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Float,
                            multisampled: false,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Float,
                            multisampled: false,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Float,
                            multisampled: false,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Float,
                            multisampled: false,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 8,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Float,
                            multisampled: false,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 9,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                ],
                label: None,
            });

        let params_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &params_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&params.ao_texture.view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&params.ao_texture.sampler),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&params.albedo_texture.view),
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&params.albedo_texture.sampler),
                },
                wgpu::Binding {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&params.emissive_texture.view),
                },
                wgpu::Binding {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&params.emissive_texture.sampler),
                },
                wgpu::Binding {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(
                        &params.metal_roughness_texture.view,
                    ),
                },
                wgpu::Binding {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(
                        &params.metal_roughness_texture.sampler,
                    ),
                },
                wgpu::Binding {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(&params.normal_texture.view),
                },
                wgpu::Binding {
                    binding: 9,
                    resource: wgpu::BindingResource::Sampler(&params.normal_texture.sampler),
                },
            ],
            label: Some("params_bind_group"),
        });

        // Lighting bind group.
        let lighting_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<LightingBindGroup>() as u64,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let lighting_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
                label: Some("lighting_bind_group_layout"),
            });

        let lighting_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &lighting_bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &lighting_buffer,
                    // FYI: you can share a single buffer between bindings.
                    range: 0..std::mem::size_of::<LightingBindGroup>() as wgpu::BufferAddress,
                },
            }],
            label: Some("lighting_bind_group"),
        });

        // Init pipeline.
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[
                    &transform_bind_group_layout,
                    &lighting_bind_group_layout,
                    &params_bind_group_layout,
                ],
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
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttributeDescriptor {
                            // Position
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float3,
                        },
                        wgpu::VertexAttributeDescriptor {
                            // Normal
                            offset: (std::mem::size_of::<f32>() * 3) as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float3,
                        },
                        wgpu::VertexAttributeDescriptor {
                            // Tangent
                            offset: (std::mem::size_of::<f32>() * 6) as wgpu::BufferAddress,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float4,
                        },
                        wgpu::VertexAttributeDescriptor {
                            // Tex Coord
                            offset: (std::mem::size_of::<f32>() * 10) as wgpu::BufferAddress,
                            shader_location: 3,
                            format: wgpu::VertexFormat::Float2,
                        },
                    ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            render_pipeline: render_pipeline,
            params_bind_group: params_bind_group,
            transform_bind_group: transform_bind_group,
            transform_bind_group_buffer: transform_buffer,
            lighting_bind_group: lighting_bind_group,
            lighting_bind_group_buffer: lighting_buffer,
        }
    }
}
