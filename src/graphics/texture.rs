
pub struct Texture
{
    texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn new_texture(device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32, rgba_data: &[u8]) -> Self {

        // Create texture.
        let size = wgpu::Extent3d {
            width: width,
            height: height,
            depth: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        // Upload data to texture.
        let buffer = device.create_buffer_with_data(
            &rgba_data,
            wgpu::BufferUsage::COPY_SRC,
        );

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("texture_buffer_copy_encoder"),
        });
        
        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &buffer,
                offset: 0,
                bytes_per_row: 4 * width,
                rows_per_image: height,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            size,
        );
        
        queue.submit(&[encoder.finish()]);

        let view = texture.create_default_view();

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
            texture,
            view,
            sampler,
        }
    }

    pub fn new_depth_texture(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        let size = wgpu::Extent3d { 
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: None,
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT
                | wgpu::TextureUsage::SAMPLED 
                | wgpu::TextureUsage::COPY_SRC,
        };

        let texture = device.create_texture(&desc);

        let view = texture.create_default_view();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::LessEqual,
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}