use super::*;

pub struct SkyboxBindGroup {
    pub environment_texture: Texture,
}

pub struct SkyboxMaterial {
    pub render_pipeline: wgpu::RenderPipeline,

    pub transform_bind_group: wgpu::BindGroup,
    pub transform_uniform_buffer: wgpu::Buffer,

    pub params_bind_group: wgpu::BindGroup,
}

impl SkyboxMaterial {
    pub fn new(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        params: &SkyboxBindGroup,
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

        let params_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &params_bind_group_layout,
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
            label: Some("params_bind_group"),
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
            format: sc_desc.format,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }];

        let depth_state = Some(wgpu::DepthStencilStateDescriptor {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
            stencil_read_mask: 0,
            stencil_write_mask: 0,
        });

        let render_pipeline = material_base::build_render_pipeline(
            device,
            &sc_desc,
            include_str!("shaders/skybox.vert"),
            include_str!("shaders/skybox.frag"),
            &[&transform_bind_group_layout, &params_bind_group_layout],
            vertex_state_desc,
            &colour_states,
            depth_state,
        );

        Self {
            render_pipeline,
            transform_bind_group,
            transform_uniform_buffer,
            params_bind_group,
        }
    }
}

impl MaterialBase for SkyboxMaterial {
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
            &self.transform_uniform_buffer,
            encoder,
            transform_data,
        );

        let mut render_pass = encoder.begin_render_pass(rp_desc);

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.transform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.params_bind_group, &[]);

        render_pass
    }
}
