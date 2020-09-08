pub struct Texture {
    _texture: wgpu::Texture,
    pub dimension: wgpu::TextureViewDimension,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn new_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        rgba_data: &[u8],
        image_format: wgpu::TextureFormat,
    ) -> Self {
        // Create texture.
        let size = wgpu::Extent3d {
            width: width,
            height: height,
            depth: 1,
        };
        let _texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: image_format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        // Upload data to texture.
        let buffer = device.create_buffer_with_data(&rgba_data, wgpu::BufferUsage::COPY_SRC);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("texture_buffer_copy_encoder"),
        });

        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &buffer,
                offset: 0,
                bytes_per_row: (rgba_data.len() / height as usize) as u32,
                rows_per_image: height,
            },
            wgpu::TextureCopyView {
                texture: &_texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            size,
        );

        queue.submit(&[encoder.finish()]);

        let view = _texture.create_default_view();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::LessEqual,
        });

        Self {
            _texture,
            dimension: wgpu::TextureViewDimension::D2,
            view,
            sampler,
        }
    }

    pub fn new_cubemap_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        face_textures: &[Texture],
        image_format: wgpu::TextureFormat,
        mip_levels: u32,
    ) -> Self {
        // Create texture.
        let size = wgpu::Extent3d {
            width: width,
            height: height,
            depth: 1,
        };
        let _texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: size,
            array_layer_count: 6,
            mip_level_count: mip_levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: image_format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("texture_buffer_copy_encoder"),
        });

        for mip_level in 0..mip_levels as usize {
            for i in 0..6 {
                encoder.copy_texture_to_texture(
                    wgpu::TextureCopyView {
                        texture: &face_textures[mip_level * 6 + i]._texture,
                        mip_level: 0,
                        array_layer: 0,
                        origin: wgpu::Origin3d::ZERO,
                    },
                    wgpu::TextureCopyView {
                        texture: &_texture,
                        mip_level: mip_level as u32,
                        array_layer: i as u32,
                        origin: wgpu::Origin3d::ZERO,
                    },
                    size,
                );
            }
        }

        queue.submit(&[encoder.finish()]);

        let view = _texture.create_view(&wgpu::TextureViewDescriptor {
            format: image_format,
            dimension: wgpu::TextureViewDimension::Cube,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            level_count: mip_levels,
            base_array_layer: 0,
            array_layer_count: 6,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::LessEqual,
        });

        Self {
            _texture,
            dimension: wgpu::TextureViewDimension::Cube,
            view,
            sampler,
        }
    }

    pub fn new_framebuffer_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        image_format: wgpu::TextureFormat,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: width,
            height: height,
            depth: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: None,
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: image_format,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT
                | wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_SRC,
        };

        let _texture = device.create_texture(&desc);

        let view = _texture.create_default_view();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::LessEqual,
        });

        Self {
            _texture,
            dimension: wgpu::TextureViewDimension::D2,
            view,
            sampler,
        }
    }
}
