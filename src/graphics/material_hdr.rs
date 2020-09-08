use super::*;

pub struct HdrCvtMaterial {
    pub render_pipeline: wgpu::RenderPipeline,
    pub transform_bind_group: wgpu::BindGroup,
    pub transform_bind_group_buffer: wgpu::Buffer,
    pub cvt_bind_group: wgpu::BindGroup,
}

pub struct HdrConvolveDiffuseMaterial {
    pub render_pipeline: wgpu::RenderPipeline,
    pub transform_bind_group: wgpu::BindGroup,
    pub transform_bind_group_buffer: wgpu::Buffer,
    pub convolve_bind_group: wgpu::BindGroup,
}

pub struct HdrConvolveSpecularMaterial {
    pub render_pipeline: wgpu::RenderPipeline,
    pub transform_bind_group: wgpu::BindGroup,
    pub transform_bind_group_buffer: wgpu::Buffer,
    pub convolve_bind_group: wgpu::BindGroup,
    pub roughness_bind_group: wgpu::BindGroup,
    pub roughness_bind_group_buffer: wgpu::Buffer,
}

pub struct HdrTransformBindGroup {
    pub proj_matrix: nalgebra::Matrix4<f32>,
    pub view_matrix: nalgebra::Matrix4<f32>,
}

pub struct HdrCvtBindGroup<'a> {
    pub equirectangular_texture: &'a Texture,
}

pub struct HdrConvolveDiffuseBindGroup<'a> {
    pub environment_texture: &'a Texture,
}

pub struct HdrConvolveSpecularBindGroup<'a> {
    pub environment_texture: &'a Texture,
    pub roughness: f32,
}

impl HdrCvtMaterial {
    pub fn new(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        params: &HdrCvtBindGroup,
    ) -> Self {
        // Init bind groups.
        // Transform buffers.
        let (transform_bind_group_buffer, transform_bind_group, transform_bind_group_layout) =
            material_base::create_uniform_buffer::<TransformBindGroup>(
                device,
                wgpu::ShaderStage::VERTEX,
            );

        // Hdr conversion bind group.
        let (cvt_bind_group_layout, cvt_bind_group) = material_base::create_texture_bind_group(
            device,
            wgpu::ShaderStage::FRAGMENT,
            &[&params.equirectangular_texture],
        );

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

impl HdrConvolveDiffuseMaterial {
    pub fn new(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        params: &HdrConvolveDiffuseBindGroup,
    ) -> Self {
        // Init bind groups.
        // Transform buffers.
        let (transform_bind_group_buffer, transform_bind_group, transform_bind_group_layout) =
            material_base::create_uniform_buffer::<TransformBindGroup>(
                device,
                wgpu::ShaderStage::VERTEX,
            );

        // Hdr conversion bind group.
        let (convolve_bind_group_layout, convolve_bind_group) =
            material_base::create_texture_bind_group(
                device,
                wgpu::ShaderStage::FRAGMENT,
                &[&params.environment_texture],
            );

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
            include_str!("./shaders/hdr_convolve_diffuse.frag"),
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

impl HdrConvolveSpecularMaterial {
    pub fn new(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        params: &HdrConvolveSpecularBindGroup,
    ) -> Self {
        // Init bind groups.
        let (transform_bind_group_buffer, transform_bind_group, transform_bind_group_layout) =
            material_base::create_uniform_buffer::<TransformBindGroup>(
                device,
                wgpu::ShaderStage::VERTEX,
            );

        // Hdr conversion bind group.
        let (convolve_bind_group_layout, convolve_bind_group) =
            material_base::create_texture_bind_group(
                device,
                wgpu::ShaderStage::FRAGMENT,
                &[&params.environment_texture],
            );

        let (roughness_bind_group_buffer, roughness_bind_group, roughness_bind_group_layout) =
            material_base::create_uniform_buffer::<f32>(device, wgpu::ShaderStage::FRAGMENT);

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
            include_str!("./shaders/hdr_convolve_specular.frag"),
            &[
                &transform_bind_group_layout,
                &convolve_bind_group_layout,
                &roughness_bind_group_layout,
            ],
            vertex_state_desc,
            &colour_states,
            None,
        );

        Self {
            render_pipeline,
            transform_bind_group_buffer,
            transform_bind_group,
            convolve_bind_group,
            roughness_bind_group,
            roughness_bind_group_buffer,
        }
    }
}
