use super::*;
use specs::prelude::*;

pub struct Renderable {
    mesh: Mesh,
    material: Material,
}

impl Component for Renderable {
    type Storage = VecStorage<Self>;
}

fn update_uniform_buffer<T>(
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

impl Renderable {
    pub fn render<'a>(
        &'a self,
        render_state: &RenderState,
        render_pass_desc: &wgpu::RenderPassDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        camera: &Camera,
        light_positions: &Vec<nalgebra::Vector3<f32>>,
        pose: &Pose,
    ) {
        let mut transform_data = TransformBindGroup {
            model_matrix: pose.model_matrix.to_homogeneous(),
            view_proj_matrix: camera.proj_matrix.as_matrix() * camera.view_matrix.to_homogeneous(),
        };

        let lighting_data = LightingBindGroup {
            position: light_positions[0],
        };

        // Upload mesh transforms.
        transform_data.model_matrix = pose.model_matrix.to_homogeneous();

        update_uniform_buffer(
            &render_state.device,
            &self.material.transform_bind_group_buffer,
            encoder,
            &transform_data,
        );
        update_uniform_buffer(
            &render_state.device,
            &self.material.lighting_bind_group_buffer,
            encoder,
            &lighting_data,
        );

        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

        render_pass.set_pipeline(&self.material.render_pipeline);
        render_pass.set_vertex_buffer(0, &self.mesh.vertex_buffer, 0, 0);
        render_pass.set_bind_group(0, &self.material.transform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.material.params_bind_group, &[]);
        render_pass.set_bind_group(2, &self.material.lighting_bind_group, &[]);

        match &self.mesh.index_buffer {
            Some(index_buffer) => {
                render_pass.set_index_buffer(&index_buffer, 0, 0);
                render_pass.draw_indexed(0..self.mesh.num_indices, 0, 0..1);
            }
            None => {
                render_pass.draw(0..self.mesh.num_vertices, 0..1);
            }
        }
    }

    pub fn new_from_glb<'a>(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        queue: &wgpu::Queue,
        glb_data: &[u8],
    ) -> Self {
        let (gltf, buffers, images) = gltf::import_slice(glb_data.as_ref()).unwrap();

        let mesh = Renderable::create_mesh(&device, &gltf, &buffers);

        let pbr_params = PbrBindGroup {
            ambient_texture: Renderable::create_texture(&device, &queue, &images[0]),
        };

        let material = Material::new(&device, &sc_desc, &pbr_params);

        Self { mesh, material }
    }

    fn create_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &gltf::image::Data,
    ) -> Texture {
        // Convert RGB to RGBA.
        let mut rgba_data = vec![0; (image.width * image.height * 4) as usize];

        for i in 0..(image.width * image.height) as usize {
            rgba_data[i * 4 + 0] = image.pixels[i * 3 + 0];
            rgba_data[i * 4 + 1] = image.pixels[i * 3 + 1];
            rgba_data[i * 4 + 2] = image.pixels[i * 3 + 2];
            rgba_data[i * 4 + 3] = 255;
        }

        Texture::new_texture(
            &device,
            &queue,
            image.width,
            image.height,
            rgba_data.as_ref(),
        )
    }

    fn create_mesh(
        device: &wgpu::Device,
        gltf: &gltf::Document,
        buffers: &Vec<gltf::buffer::Data>,
    ) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for mesh in gltf.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let pos_iter = reader.read_positions().unwrap();
                let norm_iter = reader.read_normals().unwrap();
                let tex_coord_iter = reader.read_tex_coords(0).unwrap().into_f32();

                // Read vertices and indices.
                for (vert_pos, vert_norm, vert_tex_coord) in
                    izip!(pos_iter, norm_iter, tex_coord_iter)
                {
                    vertices.push(Vertex {
                        position: vert_pos,
                        normal: vert_norm,
                        tex_coord: vert_tex_coord,
                    });
                }

                // Read indices.
                if let Some(iter) = reader.read_indices() {
                    for vertex_index in iter.into_u32() {
                        indices.push(vertex_index);
                    }
                }
            }
        }

        Mesh::new(device, vertices.as_slice(), Some(indices.as_slice()))
    }
}
