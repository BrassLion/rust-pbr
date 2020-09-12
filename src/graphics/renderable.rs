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

    fn import_gltf(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        queue: &wgpu::Queue,
        gltf: &gltf::Document,
        buffers: &Vec<gltf::buffer::Data>,
        images: &Vec<gltf::image::Data>,
        skybox: &Skybox,
    ) -> Self {
        let mut meshes = Vec::new();
        let mut textures = Vec::new();

        for mesh in gltf.meshes() {
            meshes.push(Renderable::create_mesh(&device, &mesh, &buffers));
        }
        let mat = gltf.materials().next().unwrap();

        let pbr_params = PbrBindGroup {
            ao_property: match mat.occlusion_texture() {
                Some(gltf_texture) => {
                    textures.push(Renderable::create_texture(
                        &device,
                        &queue,
                        &images[gltf_texture.texture().index()],
                        wgpu::TextureFormat::Rgba8Unorm,
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
                    textures.push(Renderable::create_texture(
                        &device,
                        &queue,
                        &images[gltf_texture.texture().index()],
                        wgpu::TextureFormat::Rgba8UnormSrgb,
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
                    textures.push(Renderable::create_texture(
                        &device,
                        &queue,
                        &images[gltf_texture.texture().index()],
                        wgpu::TextureFormat::Rgba8UnormSrgb,
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
                    textures.push(Renderable::create_texture(
                        &device,
                        &queue,
                        &images[gltf_texture.texture().index()],
                        wgpu::TextureFormat::Rgba8Unorm,
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
                    textures.push(Renderable::create_texture(
                        &device,
                        &queue,
                        &images[gltf_texture.texture().index()],
                        wgpu::TextureFormat::Rgba8Unorm,
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
            irradiance_map: &skybox.irradiance_map,
            prefiltered_environment_map: &skybox.prefiltered_environment_map,
            brdf_lut: &skybox.brdf_lut,
            textures,
        };

        let material = Box::new(PbrMaterial::new(&device, &sc_desc, &pbr_params));

        Renderable::new(meshes, material)
    }

    pub fn new_from_path(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        queue: &wgpu::Queue,
        path: &std::path::Path,
        skybox: &Skybox,
    ) -> Self {
        let (gltf, buffers, images) = gltf::import(path).unwrap();

        Renderable::import_gltf(device, sc_desc, queue, &gltf, &buffers, &images, skybox)
    }

    pub fn new_from_glb<'a>(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        queue: &wgpu::Queue,
        glb_data: &[u8],
        skybox: &Skybox,
    ) -> Self {
        let (gltf, buffers, images) = gltf::import_slice(glb_data.as_ref()).unwrap();

        Renderable::import_gltf(device, sc_desc, queue, &gltf, &buffers, &images, skybox)
    }

    fn create_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &gltf::image::Data,
        image_format: wgpu::TextureFormat,
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

                Texture::new_texture_from_data(
                    &device,
                    &queue,
                    image.width,
                    image.height,
                    rgba_data.as_ref(),
                    image_format,
                    wgpu::AddressMode::Repeat,
                )
            }
            gltf::image::Format::R8G8B8A8 => Texture::new_texture_from_data(
                &device,
                &queue,
                image.width,
                image.height,
                image.pixels.as_ref(),
                image_format,
                wgpu::AddressMode::Repeat,
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

            let tex_coord_iter: Box<dyn Iterator<Item = [f32; 2]>> = match reader.read_tex_coords(0)
            {
                Some(tex_coords_iter) => Box::new(tex_coords_iter.into_f32()),
                None => Box::new(std::iter::repeat([0.0; 2])),
            };

            let tangent_iter: Box<dyn Iterator<Item = [f32; 4]>> = match reader.read_tangents() {
                Some(tangent_iter) => Box::new(tangent_iter),
                None => Box::new(std::iter::repeat([0.0; 4])),
            };

            for (vert_pos, vert_norm, vert_tex_coord, vert_tangent) in
                izip!(pos_iter, norm_iter, tex_coord_iter, tangent_iter)
            {
                vertices.push(Vertex {
                    position: vert_pos,
                    normal: vert_norm,
                    tangent: vert_tangent,
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
            let mut tangents: Vec<nalgebra::Vector3<f32>> =
                vec![nalgebra::Vector3::zeros(); vertices.len()];
            let mut bitangents: Vec<nalgebra::Vector3<f32>> =
                vec![nalgebra::Vector3::zeros(); vertices.len()];

            for tri_ids in indices.chunks(3) {
                let i0 = tri_ids[0] as usize;
                let i1 = tri_ids[1] as usize;
                let i2 = tri_ids[2] as usize;

                let p0: nalgebra::Vector3<f32> = vertices[i0].position.into();
                let p1: nalgebra::Vector3<f32> = vertices[i1].position.into();
                let p2: nalgebra::Vector3<f32> = vertices[i2].position.into();

                let w0: nalgebra::Vector2<f32> = vertices[i0].tex_coord.into();
                let w1: nalgebra::Vector2<f32> = vertices[i1].tex_coord.into();
                let w2: nalgebra::Vector2<f32> = vertices[i2].tex_coord.into();

                let e1 = p1 - p0;
                let e2 = p2 - p0;

                let x1 = w1.x - w0.x;
                let x2 = w2.x - w0.x;

                let y1 = w1.y - w0.y;
                let y2 = w2.y - w0.y;

                let r = 1.0 / (x1 * y2 - x2 * y1);

                let t = (e1 * y2 - e2 * y1) * r;
                let b = (e2 * x1 - e1 * x2) * r;

                tangents[i0] += t;
                tangents[i1] += t;
                tangents[i2] += t;

                bitangents[i0] += b;
                bitangents[i1] += b;
                bitangents[i2] += b;
            }

            for (i, vertex) in vertices.iter_mut().enumerate() {
                let t = tangents[i];
                let b = bitangents[i];
                let n: nalgebra::Vector3<f32> = vertex.normal.into();

                let tangent = (t - n.dot(&t) * n).normalize();
                let handedness = if n.dot(&t.cross(&b)) > 0.0 { 1.0 } else { -1.0 };

                vertex.tangent = [tangent.x, tangent.y, tangent.z, handedness];
            }
        }

        Mesh::new(device, vertices.as_slice(), Some(indices.as_slice()))
    }
}
