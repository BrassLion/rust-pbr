use super::*;
use specs::prelude::*;

pub struct Renderable {
    meshes: Vec<Mesh>,
    pub material: Box<dyn MaterialBase + Send + Sync>,
}

impl Component for Renderable {
    type Storage = VecStorage<Self>;
}

impl Renderable {
    pub fn render<'a>(
        &'a self,
        render_state: &RenderState,
        render_pass_desc: &wgpu::RenderPassDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        transform_data: &TransformBindGroup,
        lighting_data: &LightingBindGroup,
    ) {
        let mut render_pass = self.material.begin_render_pass(
            &render_state.device,
            encoder,
            render_pass_desc,
            transform_data,
            lighting_data,
        );

        for mesh in self.meshes.iter() {
            mesh.draw(&mut render_pass);
        }
    }

    pub fn new(meshes: Vec<Mesh>, material: Box<dyn MaterialBase + Send + Sync>) -> Self {
        Self { meshes, material }
    }

    pub fn new_from_single_mesh(mesh: Mesh, material: Box<dyn MaterialBase + Send + Sync>) -> Self {
        let mut meshes = Vec::new();

        meshes.push(mesh);

        Self::new(meshes, material)
    }

    pub fn new_from_glb<'a>(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        queue: &wgpu::Queue,
        glb_data: &[u8],
        irradiance_texture: &Texture,
    ) -> Self {
        let (gltf, buffers, images) = gltf::import_slice(glb_data.as_ref()).unwrap();

        let mut meshes = Vec::new();
        let mut textures = Vec::new();

        for mesh in gltf.meshes() {
            meshes.push(Renderable::create_mesh(&device, &mesh, &buffers));
        }
        let mat = gltf.materials().next().unwrap();

        let pbr_params = PbrBindGroup {
            ao_property: match mat.occlusion_texture() {
                Some(gltf_texture) => {
                    println!("ao");
                    textures.push(Renderable::create_texture(
                        &device,
                        &queue,
                        &images[gltf_texture.texture().index()],
                    ));
                    MaterialProperty {
                        texture_id: Some(textures.len() - 1),
                        factor: None,
                    }
                }
                None => MaterialProperty {
                    texture_id: None,
                    factor: Some([1.0, 1.0, 1.0, 1.0]),
                },
            },
            albedo_property: match mat.pbr_metallic_roughness().base_color_texture() {
                Some(gltf_texture) => {
                    println!("albedo");
                    textures.push(Renderable::create_texture(
                        &device,
                        &queue,
                        &images[gltf_texture.texture().index()],
                    ));
                    MaterialProperty {
                        texture_id: Some(textures.len() - 1),
                        factor: None,
                    }
                }
                None => MaterialProperty {
                    texture_id: None,
                    factor: Some(mat.pbr_metallic_roughness().base_color_factor()),
                },
            },
            emissive_property: match mat.emissive_texture() {
                Some(gltf_texture) => {
                    println!("emissive");
                    textures.push(Renderable::create_texture(
                        &device,
                        &queue,
                        &images[gltf_texture.texture().index()],
                    ));
                    MaterialProperty {
                        texture_id: Some(textures.len() - 1),
                        factor: None,
                    }
                }
                None => MaterialProperty {
                    texture_id: None,
                    factor: Some([
                        mat.emissive_factor()[0],
                        mat.emissive_factor()[1],
                        mat.emissive_factor()[2],
                        1.0,
                    ]),
                },
            },
            metal_roughness_property: match mat
                .pbr_metallic_roughness()
                .metallic_roughness_texture()
            {
                Some(gltf_texture) => {
                    println!("metal_rough");
                    textures.push(Renderable::create_texture(
                        &device,
                        &queue,
                        &images[gltf_texture.texture().index()],
                    ));
                    MaterialProperty {
                        texture_id: Some(textures.len() - 1),
                        factor: None,
                    }
                }
                None => MaterialProperty {
                    texture_id: None,
                    factor: Some([
                        0.0,
                        mat.pbr_metallic_roughness().metallic_factor(),
                        mat.pbr_metallic_roughness().roughness_factor(),
                        0.0,
                    ]),
                },
            },
            normal_property: match mat.normal_texture() {
                Some(gltf_texture) => {
                    println!("normal");
                    textures.push(Renderable::create_texture(
                        &device,
                        &queue,
                        &images[gltf_texture.texture().index()],
                    ));
                    MaterialProperty {
                        texture_id: Some(textures.len() - 1),
                        factor: None,
                    }
                }
                None => MaterialProperty {
                    texture_id: None,
                    factor: Some([0.0; 4]),
                },
            },
            irradiance_texture,
            textures,
        };

        let material = Box::new(PbrMaterial::new(&device, &sc_desc, &pbr_params));

        Renderable::new(meshes, material)
    }

    fn create_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &gltf::image::Data,
    ) -> Texture {
        match image.format {
            gltf::image::Format::R8G8B8 => {
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
                    wgpu::TextureFormat::Rgba8UnormSrgb,
                )
            }
            gltf::image::Format::R8G8B8A8 => Texture::new_texture(
                &device,
                &queue,
                image.width,
                image.height,
                image.pixels.as_ref(),
                wgpu::TextureFormat::Rgba8UnormSrgb,
            ),
            _ => panic!("Unimplemented tex type"),
        }
    }

    fn create_mesh(
        device: &wgpu::Device,
        gltf_mesh: &gltf::Mesh,
        buffers: &Vec<gltf::buffer::Data>,
    ) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for primitive in gltf_mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let pos_iter = reader.read_positions().unwrap();
            let norm_iter = reader.read_normals().unwrap();
            let tex_coord_iter = reader.read_tex_coords(0).unwrap().into_f32();

            // Read vertices and indices.
            for (vert_pos, vert_norm, vert_tex_coord) in izip!(pos_iter, norm_iter, tex_coord_iter)
            {
                vertices.push(Vertex {
                    position: vert_pos,
                    normal: vert_norm,
                    tangent: [0.0; 4], // Calculated later.
                    tex_coord: vert_tex_coord,
                });
            }

            // Read indices.
            if let Some(iter) = reader.read_indices() {
                for vertex_index in iter.into_u32() {
                    indices.push(vertex_index);
                }
            }

            // Calculate tangents.

            let mut index_iterator = indices.iter();

            for vertex_id in 0..vertices.len() {
                // Find first occurence of vertex.
                let vertex_id_indice_pos =
                    index_iterator.position(|x| *x == vertex_id as u32).unwrap();

                let tri_idx = vertex_id_indice_pos / 3;

                // Get all vertices in triangle.
                let verts_in_tri = [
                    vertices[indices[tri_idx * 3 + 0] as usize],
                    vertices[indices[tri_idx * 3 + 1] as usize],
                    vertices[indices[tri_idx * 3 + 2] as usize],
                ];

                let v1: nalgebra::Vector3<f32> = verts_in_tri[0].position.into();
                let v2: nalgebra::Vector3<f32> = verts_in_tri[1].position.into();
                let v3: nalgebra::Vector3<f32> = verts_in_tri[2].position.into();

                let uv1: nalgebra::Vector2<f32> = verts_in_tri[0].tex_coord.into();
                let uv2: nalgebra::Vector2<f32> = verts_in_tri[1].tex_coord.into();
                let uv3: nalgebra::Vector2<f32> = verts_in_tri[2].tex_coord.into();

                let e1 = v2 - v1;
                let e2 = v3 - v1;

                let delta_uv1 = uv2 - uv1;
                let delta_uv2 = uv3 - uv1;

                let f = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

                vertices[vertex_id].tangent[0] = f * (delta_uv2.y * e1.x - delta_uv1.y * e2.x);
                vertices[vertex_id].tangent[1] = f * (delta_uv2.y * e1.y - delta_uv1.y * e2.y);
                vertices[vertex_id].tangent[2] = f * (delta_uv2.y * e1.z - delta_uv1.y * e2.z);
                vertices[vertex_id].tangent[3] = 1.0;
            }
        }

        Mesh::new(device, vertices.as_slice(), Some(indices.as_slice()))
    }
}
