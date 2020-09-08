use super::*;

pub struct PbrBindGroup<'a> {
    pub textures: Vec<Texture>,

    pub ao_property: MaterialProperty,
    pub albedo_property: MaterialProperty,
    pub emissive_property: MaterialProperty,
    pub metal_roughness_property: MaterialProperty,
    pub normal_property: MaterialProperty,

    pub irradiance_map: &'a Texture,
    pub prefiltered_environment_map: &'a Texture,
    pub brdf_lut: &'a Texture,
}

#[derive(Copy, Clone)]
pub struct MaterialProperty {
    pub factor: Option<[f32; 4]>,
    pub texture_id: Option<usize>,
}

pub struct PbrMaterial {
    pub render_pipeline: wgpu::RenderPipeline,
    pub transform_bind_group: wgpu::BindGroup,
    pub lighting_bind_group: wgpu::BindGroup,
    pub pbr_factor_bind_group: wgpu::BindGroup,
    pub pbr_texture_bind_group: wgpu::BindGroup,

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
        let mut pbr_factor_values = Vec::new();
        let mut pbr_texture_binding_entries = Vec::new();
        let mut pbr_texture_bindings = Vec::new();
        let mut pbr_defines = "".to_owned();

        // Add constant texture bindings.
        pbr_texture_binding_entries.extend_from_slice(&[
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
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    dimension: wgpu::TextureViewDimension::Cube,
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
        ]);

        pbr_texture_bindings.extend_from_slice(&[
            wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&params.irradiance_map.view),
            },
            wgpu::Binding {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&params.irradiance_map.sampler),
            },
            wgpu::Binding {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(
                    &params.prefiltered_environment_map.view,
                ),
            },
            wgpu::Binding {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(
                    &params.prefiltered_environment_map.sampler,
                ),
            },
            wgpu::Binding {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&params.brdf_lut.view),
            },
            wgpu::Binding {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(&params.brdf_lut.sampler),
            },
        ]);

        let pbr_properties = [
            ("AO", params.ao_property),
            ("ALBEDO", params.albedo_property),
            ("EMISSIVE", params.emissive_property),
            ("METAL_ROUGHNESS", params.metal_roughness_property),
        ];

        for (prop_name, property) in pbr_properties.iter() {
            match property.texture_id {
                None => pbr_factor_values.push(property.factor.unwrap()),
                Some(texture_id) => {
                    pbr_texture_binding_entries.push(wgpu::BindGroupLayoutEntry {
                        binding: pbr_texture_binding_entries.len() as u32,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Float,
                            multisampled: false,
                        },
                    });
                    pbr_texture_binding_entries.push(wgpu::BindGroupLayoutEntry {
                        binding: pbr_texture_binding_entries.len() as u32,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    });

                    pbr_texture_bindings.push(wgpu::Binding {
                        binding: pbr_texture_bindings.len() as u32,
                        resource: wgpu::BindingResource::TextureView(
                            &params.textures[texture_id].view,
                        ),
                    });
                    pbr_texture_bindings.push(wgpu::Binding {
                        binding: pbr_texture_bindings.len() as u32,
                        resource: wgpu::BindingResource::Sampler(
                            &params.textures[texture_id].sampler,
                        ),
                    });

                    pbr_defines = format!(
                        "{}#define {}_TEXTURE_BINDING {}\n",
                        pbr_defines,
                        prop_name,
                        pbr_texture_bindings.len() - 2
                    );
                }
            }
        }

        println!("{}", pbr_defines);

        // Push a dumm
        if pbr_factor_values.len() == 0 {
            pbr_factor_values.push([0.0; 4]);
        }

        let pbr_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: pbr_texture_binding_entries.as_slice(),
                label: None,
            });

        let pbr_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pbr_texture_bind_group_layout,
            bindings: pbr_texture_bindings.as_slice(),
            label: None,
        });

        let pbr_factor_data_bytes = unsafe {
            let len = std::mem::size_of_val(pbr_factor_values.as_slice());
            let ptr = (pbr_factor_values.as_ptr() as *const _) as *const u8;
            std::slice::from_raw_parts(ptr, len)
        };

        let pbr_factor_uniform_buffer =
            device.create_buffer_with_data(pbr_factor_data_bytes, wgpu::BufferUsage::UNIFORM);

        let pbr_factor_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
            });

        let pbr_factor_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pbr_factor_bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &pbr_factor_uniform_buffer,
                    // FYI: you can share a single buffer between bindings.
                    range: 0..pbr_factor_data_bytes.len() as wgpu::BufferAddress,
                },
            }],
            label: Some("transform_bind_group"),
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
            &format!(
                "#version 450\n\n{}\n{}",
                pbr_defines,
                include_str!("shaders/pbr.frag")
            ),
            &[
                &transform_bind_group_layout,
                &lighting_bind_group_layout,
                &pbr_factor_bind_group_layout,
                &pbr_texture_bind_group_layout,
            ],
            vertex_state_desc,
            &colour_states,
            depth_state,
        );

        Self {
            render_pipeline,
            transform_bind_group,
            lighting_bind_group,
            pbr_factor_bind_group,
            pbr_texture_bind_group,
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
        render_pass.set_bind_group(2, &self.pbr_factor_bind_group, &[]);
        render_pass.set_bind_group(3, &self.pbr_texture_bind_group, &[]);

        render_pass
    }
}
