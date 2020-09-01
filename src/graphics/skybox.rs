use super::*;
use nalgebra::*;

pub struct Skybox {
    pub environment_texture: Texture,
    pub irradiance_texture: Texture,
}

impl Skybox {
    pub fn new(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        queue: &wgpu::Queue,
        hdr_data: &[u8],
    ) -> (Skybox, Renderable) {
        // Decode HDR data.
        let reader = image::hdr::HdrDecoder::new(hdr_data);
        let decoder = reader.unwrap();

        let width = decoder.metadata().width;
        let height = decoder.metadata().height;
        let pixels = decoder.read_image_hdr().unwrap();

        // Add alpha data.
        let mut pixel_data = Vec::new();

        for pixel in pixels {
            pixel_data.push(pixel[0]);
            pixel_data.push(pixel[1]);
            pixel_data.push(pixel[2]);
            pixel_data.push(1.0);
        }

        // Create HDR equirectangular texture.
        let pixel_data_bytes = unsafe {
            let len = pixel_data.len() * std::mem::size_of::<f32>();
            let ptr = pixel_data.as_ptr() as *const u8;
            std::slice::from_raw_parts(ptr, len)
        };

        let hdr_texture = Texture::new_texture(
            device,
            queue,
            width,
            height,
            pixel_data_bytes,
            wgpu::TextureFormat::Rgba32Float,
        );

        // Create HDR material.
        let hdr_material_params = HdrCvtBindGroup {
            equirectangular_texture: hdr_texture,
        };

        let hdr_material = Box::new(HdrCvtMaterial::new(
            device,
            &wgpu::SwapChainDescriptor {
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                width: 512,
                height: 512,
                present_mode: wgpu::PresentMode::Immediate,
            },
            &hdr_material_params,
        ));

        // Create unit cube.
        let cube_vertices: [[f32; 3]; 8] = [
            // front
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
            // back
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
        ];

        let cube_elements: [u32; 36] = [
            // front
            0, 1, 2, 2, 3, 0, // right
            1, 5, 6, 6, 2, 1, // back
            7, 6, 5, 5, 4, 7, // left
            4, 0, 3, 3, 7, 4, // bottom
            4, 5, 1, 1, 0, 4, // top
            3, 2, 6, 6, 7, 3,
        ];

        let cube_vertex_array = cube_vertices
            .iter()
            .map(|&x| Vertex {
                position: x,
                normal: [0.0, 0.0, 0.0],
                tangent: [0.0, 0.0, 0.0, 0.0],
                tex_coord: [0.0, 0.0],
            })
            .collect::<Vec<Vertex>>();

        let cube_mesh = Mesh::new(device, cube_vertex_array.as_slice(), Some(&cube_elements));

        // Convert to cubemap
        let proj = Perspective3::new(1.0, std::f32::consts::PI / 180.0 * 90.0, 0.1, 10.0);

        let views = [
            nalgebra::Similarity3::look_at_rh(
                &Point3::new(0.0, 0.0, 0.0),
                &Point3::new(1.0, 0.0, 0.0),
                &Vector3::new(0.0, -1.0, 0.0),
                1.0,
            ),
            nalgebra::Similarity3::look_at_rh(
                &Point3::new(0.0, 0.0, 0.0),
                &Point3::new(-1.0, 0.0, 0.0),
                &Vector3::new(0.0, -1.0, 0.0),
                1.0,
            ),
            nalgebra::Similarity3::look_at_rh(
                &Point3::new(0.0, 0.0, 0.0),
                &Point3::new(0.0, -1.0, 0.0),
                &Vector3::new(0.0, 0.0, -1.0),
                1.0,
            ),
            nalgebra::Similarity3::look_at_rh(
                &Point3::new(0.0, 0.0, 0.0),
                &Point3::new(0.0, 1.0, 0.0),
                &Vector3::new(0.0, 0.0, 1.0),
                1.0,
            ),
            nalgebra::Similarity3::look_at_rh(
                &Point3::new(0.0, 0.0, 0.0),
                &Point3::new(0.0, 0.0, 1.0),
                &Vector3::new(0.0, -1.0, 0.0),
                1.0,
            ),
            nalgebra::Similarity3::look_at_rh(
                &Point3::new(0.0, 0.0, 0.0),
                &Point3::new(0.0, 0.0, -1.0),
                &Vector3::new(0.0, -1.0, 0.0),
                1.0,
            ),
        ];

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("HDR convert"),
        });

        let mut cubemap_faces = Vec::new();

        // Render every cube face.
        for i in 0..6 {
            let cubemap_face = Texture::new_framebuffer_texture(
                device,
                512,
                512,
                wgpu::TextureFormat::Rgba16Float,
            );

            let transforms = HdrTransformBindGroup {
                proj_matrix: proj.to_homogeneous(),
                view_matrix: views[i].to_homogeneous(),
            };

            material_base::update_uniform_buffer(
                device,
                &hdr_material.transform_bind_group_buffer,
                &mut encoder,
                &transforms,
            );

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &cubemap_face.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color::BLACK,
                    }],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&hdr_material.render_pipeline);
                render_pass.set_bind_group(0, &hdr_material.transform_bind_group, &[]);
                render_pass.set_bind_group(1, &hdr_material.cvt_bind_group, &[]);

                cube_mesh.draw(&mut render_pass);
            }

            cubemap_faces.push(cubemap_face);
        }

        let cmd_buffer = encoder.finish();

        queue.submit(&[cmd_buffer]);

        // Create environment cubemap.
        let environment_texture = Texture::new_cubemap_texture(
            device,
            queue,
            512,
            512,
            cubemap_faces.as_slice(),
            wgpu::TextureFormat::Rgba16Float,
        );

        // Convolve the environment map.
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("HDR convolve"),
        });

        let convolve_params = HdrConvolveBindGroup {
            environment_texture,
        };

        let convolve_material = HdrConvolveMaterial::new(device, sc_desc, &convolve_params);

        let mut cubemap_faces = Vec::new();

        for i in 0..6 {
            let cubemap_face =
                Texture::new_framebuffer_texture(device, 32, 32, wgpu::TextureFormat::Rgba16Float);

            let transforms = HdrTransformBindGroup {
                proj_matrix: proj.to_homogeneous(),
                view_matrix: views[i].to_homogeneous(),
            };

            material_base::update_uniform_buffer(
                device,
                &convolve_material.transform_bind_group_buffer,
                &mut encoder,
                &transforms,
            );

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &cubemap_face.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color::BLACK,
                    }],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&convolve_material.render_pipeline);
                render_pass.set_bind_group(0, &convolve_material.transform_bind_group, &[]);
                render_pass.set_bind_group(1, &convolve_material.convolve_bind_group, &[]);

                cube_mesh.draw(&mut render_pass);
            }

            cubemap_faces.push(cubemap_face);
        }

        let cmd_buffer = encoder.finish();

        queue.submit(&[cmd_buffer]);

        let irradiance_texture = Texture::new_cubemap_texture(
            device,
            queue,
            32,
            32,
            cubemap_faces.as_slice(),
            wgpu::TextureFormat::Rgba16Float,
        );

        let skybox_params = SkyboxBindGroup {
            environment_texture: convolve_params.environment_texture,
        };

        let material = Box::new(SkyboxMaterial::new(device, sc_desc, &skybox_params));
        let skybox = Skybox {
            environment_texture: skybox_params.environment_texture,
            irradiance_texture,
        };

        (
            skybox,
            Renderable::new_from_single_mesh(cube_mesh, material),
        )
    }
}
