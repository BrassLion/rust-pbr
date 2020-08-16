
use super::*;
use specs::prelude::*;

pub struct RenderSystem;

fn update_transform_buffer(device: &wgpu::Device, transform_buffer: &wgpu::Buffer, encoder: &mut wgpu::CommandEncoder, transform_data: &TransformBindGroup) {
    // TODO: Replace this with a function.
    let transform_data_bytes = unsafe {
        let len = std::mem::size_of_val(transform_data);
        let ptr = (transform_data as *const _) as *const u8;
        std::slice::from_raw_parts(ptr, len)
    };
    
    let staging_buffer =
        device.create_buffer_with_data(transform_data_bytes, wgpu::BufferUsage::COPY_SRC);

    encoder.copy_buffer_to_buffer(
        &staging_buffer,
        0,
        &transform_buffer,
        0,
        std::mem::size_of::<TransformBindGroup>() as wgpu::BufferAddress,
    );
}

impl<'a> System<'a> for RenderSystem {

    type SystemData = (WriteExpect<'a, RenderState>,
        ReadExpect<'a, Camera>,
        ReadExpect<'a, Texture>,
        ReadStorage<'a, Pose>,
        ReadStorage<'a, Mesh>,
        ReadStorage<'a, Material>);

    fn setup(&mut self, world: &mut World) {
        
        // Create depth texture.
        let depth_tex;

        {
            let render_state: WriteExpect<RenderState> = world.system_data();

            depth_tex = Texture::new_depth_texture(&render_state.device, &render_state.swap_chain_desc);
        }

        world.insert(depth_tex);
    }

    fn run(&mut self, data: Self::SystemData) {
        
        let (mut render_state,
            camera, 
            depth_texture,
            pose,
            mesh,
            material) = data;

        // Start new command buffer.
        let frame = render_state
                    .swap_chain
                    .get_next_texture()
                    .expect("Timeout getting texture");

        let mut encoder = render_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
            
        // Render all meshes.
        let mut transform_data = TransformBindGroup {
            model_matrix: nalgebra::Matrix4::identity(),
            view_proj_matrix: camera.proj_matrix.as_matrix() * camera.view_matrix.to_homogeneous()
        };

        for (pose, mesh, material) in (&pose, &mesh, &material).join() {
            
            // Upload mesh transforms.
            transform_data.model_matrix = pose.model_matrix.to_homogeneous();

            update_transform_buffer(&render_state.device, &material.transform_bind_group_buffer, &mut encoder, &transform_data);

            // Draw mesh.
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::BLACK,
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &depth_texture.view,
                    depth_load_op: wgpu::LoadOp::Clear,
                    depth_store_op: wgpu::StoreOp::Store,
                    clear_depth: 1.0,
                    stencil_load_op: wgpu::LoadOp::Clear,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_stencil: 0,
                }),
            });

            render_pass.set_pipeline(&material.render_pipeline);
            render_pass.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
            render_pass.set_bind_group(0, &material.transform_bind_group, &[]);
            render_pass.set_bind_group(1, &material.params_bind_group, &[]);

            match &mesh.index_buffer {
                Some(index_buffer) => {
                    render_pass.set_index_buffer(&index_buffer, 0, 0);
                    render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
                }
                None => {
                    render_pass.draw(0..mesh.num_vertices, 0..1);
                }
            }
        }

        // Submit command buffer to the render queue.
        let command_buffer = encoder.finish();

        render_state.queue.submit(&[command_buffer]);
    }
}