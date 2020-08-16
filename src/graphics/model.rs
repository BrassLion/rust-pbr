
use super::*;
use specs::prelude::*;

pub struct ModelLoader;

impl ModelLoader {
    pub fn add_glb_to_world(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor, queue: &wgpu::Queue, glb_data: &[u8], world: &mut World) {
        
        let (gltf, buffers, images) = gltf::import_slice(glb_data.as_ref()).unwrap();

        let mesh = ModelLoader::create_mesh(&device, &gltf, &buffers);
        
        let pbr_params = PbrBindGroup {
            ambient_texture: ModelLoader::create_texture(&device, &queue, &images[0]),
        };        

        let material = Material::new(&device, &sc_desc, &pbr_params);

        world.register::<Pose>();
        world.register::<Mesh>();
        world.register::<Material>();

        world.create_entity()
            .with(Pose{
                model_matrix: nalgebra::Similarity3::identity(),
            })
            .with(mesh)
            .with(material)
            .build();
    }

    fn create_texture(device: &wgpu::Device, queue: &wgpu::Queue, image: &gltf::image::Data) -> Texture {
        // Convert RGB to RGBA.
        let mut rgba_data = vec![0; (image.width * image.height * 4) as usize];

        for i in 0..(image.width * image.height) as usize
        {
            rgba_data[i * 4 + 0] = image.pixels[i * 3 + 0];
            rgba_data[i * 4 + 1] = image.pixels[i * 3 + 1];
            rgba_data[i * 4 + 2] = image.pixels[i * 3 + 2];
            rgba_data[i * 4 + 3] = 255;
        }

        Texture::new_texture(&device, &queue, image.width, image.height, rgba_data.as_ref())
    }

    fn create_mesh(device: &wgpu::Device, gltf: &gltf::Document, buffers: &Vec<gltf::buffer::Data>) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for mesh in gltf.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let pos_iter = reader.read_positions().unwrap();
                let norm_iter = reader.read_normals().unwrap();
                let tex_coord_iter = reader.read_tex_coords(0).unwrap().into_f32();
                
                // Read vertices and indices.
                for (vert_pos, vert_norm, vert_tex_coord) in izip!(pos_iter, norm_iter, tex_coord_iter) {
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