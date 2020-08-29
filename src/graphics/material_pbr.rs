use super::*;

pub struct PbrBindGroup {
    pub ao_texture: Texture,
    pub albedo_texture: Texture,
    pub emissive_texture: Texture,
    pub metal_roughness_texture: Texture,
    pub normal_texture: Texture,
}

pub struct PbrMaterial {
    pub render_pipeline: wgpu::RenderPipeline,
    pub params_bind_group: wgpu::BindGroup,
    pub transform_bind_group: wgpu::BindGroup,
    pub lighting_bind_group: wgpu::BindGroup,

    pub transform_uniform_buffer: wgpu::Buffer,
    pub lighting_uniform_buffer: wgpu::Buffer,
}

impl PbrMaterial {
    pub fn new(
        device: &wgpu::Device,
        swap_chain_desc: &wgpu::SwapChainDescriptor,
        params: &PbrBindGroup,
    ) -> Self {
        // Init bind groups.

        // Transform buffers.
        let transform_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
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
                    buffer: &transform_uniform_buffer,
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
        let lighting_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
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
                    buffer: &lighting_uniform_buffer,
                    // FYI: you can share a single buffer between bindings.
                    range: 0..std::mem::size_of::<LightingBindGroup>() as wgpu::BufferAddress,
                },
            }],
            label: Some("lighting_bind_group"),
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

        let colour_states = [wgpu::ColorStateDescriptor {
            format: swap_chain_desc.format,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }];

        let depth_state = Some(wgpu::DepthStencilStateDescriptor {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
        });

        let render_pipeline = material_base::build_render_pipeline(
            device,
            &swap_chain_desc,
            include_str!("shaders/pbr.vert"),
            include_str!("shaders/pbr.frag"),
            &[
                &transform_bind_group_layout,
                &lighting_bind_group_layout,
                &params_bind_group_layout,
            ],
            vertex_state_desc,
            &colour_states,
            depth_state,
        );

        Self {
            render_pipeline,
            params_bind_group,
            transform_bind_group,
            lighting_bind_group,
            transform_uniform_buffer,
            lighting_uniform_buffer,
        }
    }
}

impl MaterialBase for PbrMaterial {
    fn begin_render_pass<'a>(
        &'a self,
        device: &wgpu::Device,
        encoder: &'a mut wgpu::CommandEncoder,
        rp_desc: &'a wgpu::RenderPassDescriptor,
        transform_data: &TransformBindGroup,
        lighting_data: &LightingBindGroup,
    ) -> wgpu::RenderPass<'a> {
        material_base::update_uniform_buffer(
            device,
            &self.lighting_uniform_buffer,
            encoder,
            lighting_data,
        );
        material_base::update_uniform_buffer(
            device,
            &self.transform_uniform_buffer,
            encoder,
            transform_data,
        );

        let mut render_pass = encoder.begin_render_pass(rp_desc);

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.transform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.lighting_bind_group, &[]);
        render_pass.set_bind_group(2, &self.params_bind_group, &[]);

        render_pass
    }
}
