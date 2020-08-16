
use super::*;
use specs::prelude::*;

pub struct RenderSystem;

impl<'a> System<'a> for RenderSystem {

    type SystemData = (WriteExpect<'a, RenderState>,
        WriteExpect<'a, Camera>,
        ReadExpect<'a, Texture>,
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
            mut camera, 
            depth_texture,
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

        // Update the scene camera transform.
        camera.update(&render_state.device, &mut encoder);

        {
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
            
            // Render all meshes.
            for (mesh, material) in (&mesh, &material).join() {
                
                render_pass.set_pipeline(&material.render_pipeline);
                render_pass.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
                render_pass.set_bind_group(0, &camera.uniform_bind_group, &[]);
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
        }

        // Submit command buffer to the render queue.
        let command_buffer = encoder.finish();

        render_state.queue.submit(&[command_buffer]);
    }
}