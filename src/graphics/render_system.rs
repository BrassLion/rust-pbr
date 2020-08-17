use super::*;
use specs::prelude::*;

pub struct RenderSystem;

impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        WriteExpect<'a, RenderState>,
        ReadExpect<'a, Camera>,
        ReadExpect<'a, Texture>,
        ReadStorage<'a, Light>,
        ReadStorage<'a, Pose>,
        ReadStorage<'a, Renderable>,
    );

    fn setup(&mut self, world: &mut World) {
        // Create depth texture.
        let depth_tex;

        {
            let render_state: WriteExpect<RenderState> = world.system_data();

            depth_tex =
                Texture::new_depth_texture(&render_state.device, &render_state.swap_chain_desc);
        }

        world.insert(depth_tex);
    }

    fn run(&mut self, data: Self::SystemData) {
        let (mut render_state, camera, depth_texture, light, pose, renderable) = data;

        // Start new command buffer.
        let frame = render_state
            .swap_chain
            .get_next_texture()
            .expect("Timeout getting texture");

        let mut encoder =
            render_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // Clear frame.
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

        // Upload lighting data.
        let mut light_positions = Vec::new();

        for (pose, _) in (&pose, &light).join() {
            light_positions.push(pose.model_matrix.isometry.translation.vector);
        }

        for (pose, renderable) in (&pose, &renderable).join() {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Load,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::BLACK,
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &depth_texture.view,
                    depth_load_op: wgpu::LoadOp::Load,
                    depth_store_op: wgpu::StoreOp::Store,
                    clear_depth: 1.0,
                    stencil_load_op: wgpu::LoadOp::Clear,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_stencil: 0,
                }),
            };

            renderable.render(
                &render_state,
                &render_pass_desc,
                &mut encoder,
                &camera,
                &light_positions,
                &pose,
            );
        }

        // Submit command buffer to the render queue.
        let command_buffer = encoder.finish();

        render_state.queue.submit(&[command_buffer]);
    }
}
