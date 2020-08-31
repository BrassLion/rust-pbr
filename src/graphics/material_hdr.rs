use super::*;

pub struct HdrCvtMaterial {
    pub render_pipeline: wgpu::RenderPipeline,
    pub transform_bind_group: wgpu::BindGroup,
    pub transform_bind_group_buffer: wgpu::Buffer,
    pub cvt_bind_group: wgpu::BindGroup,
}

pub struct HdrConvolveMaterial {
    pub render_pipeline: wgpu::RenderPipeline,
    pub transform_bind_group: wgpu::BindGroup,
    pub transform_bind_group_buffer: wgpu::Buffer,
    pub convolve_bind_group: wgpu::BindGroup,
}

pub struct HdrTransformBindGroup {
    pub proj_matrix: nalgebra::Matrix4<f32>,
    pub view_matrix: nalgebra::Matrix4<f32>,
}

pub struct HdrCvtBindGroup {
    pub equirectangular_texture: Texture,
}

pub struct HdrConvolveBindGroup {
    pub environment_texture: Texture,
}

impl HdrCvtMaterial {
    pub fn new(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        params: &HdrCvtBindGroup,
    ) -> Self {
        // Init bind groups.
        // Transform buffers.
        let transform_bind_group_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<HdrTransformBindGroup>() as u64,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let transform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
                label: Some("transform_bind_group_layout"),
            });

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &transform_bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &transform_bind_group_buffer,
                    range: 0..std::mem::size_of::<HdrTransformBindGroup>() as wgpu::BufferAddress,
                },
            }],
            label: Some("transform_bind_group"),
        });

        // Hdr conversion bind group.
        let cvt_bind_group_layout =
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
                ],
                label: None,
            });

        let cvt_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &cvt_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &params.equirectangular_texture.view,
                    ),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(
                        &params.equirectangular_texture.sampler,
                    ),
                },
            ],
            label: None,
        });

        // Init pipeline.
        let vertex_state_desc = wgpu::VertexStateDescriptor {
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
        };

        // Build pipeline.

        let colour_states = [wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Rgba16Float,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }];

        let render_pipeline = material_base::build_render_pipeline(
            device,
            sc_desc,
            include_str!("./shaders/hdr.vert"),
            include_str!("./shaders/hdr_cvt.frag"),
            &[&transform_bind_group_layout, &cvt_bind_group_layout],
            vertex_state_desc,
            &colour_states,
            None,
        );

        Self {
            render_pipeline,
            transform_bind_group,
            transform_bind_group_buffer,
            cvt_bind_group,
        }
    }
}

impl HdrConvolveMaterial {
    pub fn new(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        params: &HdrConvolveBindGroup,
    ) -> Self {
        // Init bind groups.
        // Transform buffers.
        let transform_bind_group_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<HdrTransformBindGroup>() as u64,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let transform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
                label: Some("transform_bind_group_layout"),
            });

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &transform_bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &transform_bind_group_buffer,
                    range: 0..std::mem::size_of::<HdrTransformBindGroup>() as wgpu::BufferAddress,
                },
            }],
            label: Some("transform_bind_group"),
        });

        // Hdr convolution bind group.
        let convolve_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            dimension: wgpu::TextureViewDimension::Cube,
                            component_type: wgpu::TextureComponentType::Float,
                            multisampled: false,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                ],
                label: None,
            });

        let convolve_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &convolve_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&params.environment_texture.view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&params.environment_texture.sampler),
                },
            ],
            label: None,
        });

        // Init pipeline.
        let vertex_state_desc = wgpu::VertexStateDescriptor {
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
        };

        // Build pipeline.

        let colour_states = [wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Rgba16Float,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }];

        let render_pipeline = material_base::build_render_pipeline(
            device,
            sc_desc,
            include_str!("./shaders/hdr.vert"),
            include_str!("./shaders/hdr_convolve.frag"),
            &[&transform_bind_group_layout, &convolve_bind_group_layout],
            vertex_state_desc,
            &colour_states,
            None,
        );

        Self {
            render_pipeline,
            transform_bind_group,
            transform_bind_group_buffer,
            convolve_bind_group,
        }
    }
}
