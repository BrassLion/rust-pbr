#[macro_use]
extern crate itertools;

mod graphics;

use specs::prelude::*;

struct ExampleRenderLoop {
    world: World,
    dispatcher: Dispatcher<'static, 'static>,
}

struct RotateLightSystem;

impl<'a> System<'a> for RotateLightSystem {
    type SystemData = (
        WriteStorage<'a, graphics::Pose>,
        WriteStorage<'a, graphics::Light>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut pose, light) = data;

        for (pose, _) in (&mut pose, &light).join() {
            pose.model_matrix.append_rotation_wrt_point_mut(
                &nalgebra::UnitQuaternion::new(nalgebra::Vector3::new(0.0, 0.0, 0.01)),
                &nalgebra::Point3::new(0.0, 0.0, 0.0),
            )
        }
    }
}

impl graphics::RenderLoopEvent for ExampleRenderLoop {
    fn init(window: &winit::window::Window) -> Self {
        // Init rendering state.
        let render_state = futures::executor::block_on(graphics::RenderState::new(&window));

        // Create camera.
        let camera_position = nalgebra::Point3::new(1.0, 0.0, 1.0);
        let camera_target = nalgebra::Point3::new(0.0, 0.0, 0.0);
        let camera_up = nalgebra::Vector3::y_axis();

        let camera = graphics::Camera::new(
            &camera_position,
            &camera_target,
            &camera_up,
            1920.0 / 1080.0,
            std::f32::consts::PI / 180.0 * 100.0,
            0.1,
            1000.0,
        );

        // Create render system.
        let mut dispatcher = DispatcherBuilder::new()
            .with(RotateLightSystem, "rot_system", &[])
            .with(graphics::RenderSystem, "render_system", &["rot_system"])
            .build();

        // Create world.
        let mut world = World::new();

        world.register::<graphics::Renderable>();
        world.register::<graphics::Pose>();
        world.register::<graphics::Light>();

        // Add model to world.
        let helmet_data = include_bytes!("../res/DamagedHelmet.glb");

        let hdr_data = include_bytes!("../res/venice_sunset_2k.hdr");

        let skybox = graphics::Skybox::new(
            &render_state.device,
            &render_state.swap_chain_desc,
            &render_state.queue,
            hdr_data,
        );

        world
            .create_entity()
            .with(graphics::Renderable::new_from_glb(
                &render_state.device,
                &render_state.swap_chain_desc,
                &render_state.queue,
                helmet_data,
            ))
            .with(graphics::Pose {
                model_matrix: nalgebra::Similarity3::from_parts(
                    nalgebra::Translation3::identity(),
                    nalgebra::UnitQuaternion::from_euler_angles(
                        std::f32::consts::FRAC_PI_2,
                        0.0,
                        0.0,
                    ),
                    1.0,
                ),
            })
            .build();

        world
            .create_entity()
            .with(skybox)
            .with(graphics::Pose {
                model_matrix: nalgebra::Similarity3::identity(),
            })
            .build();

        world
            .create_entity()
            // .with(graphics::Renderable::new_from_glb(&render_state.device,
            //     &render_state.swap_chain_desc,
            //     &render_state.queue,
            //     include_bytes!("../res/BoxTextured.glb")))
            .with(graphics::Pose {
                model_matrix: nalgebra::Similarity3::from_parts(
                    nalgebra::Translation3::new(5.0, 0.0, 0.0),
                    nalgebra::UnitQuaternion::identity(),
                    1.0,
                ),
            })
            .with(graphics::Light {})
            .build();

        // Pass render state into ECS as last step.
        world.insert(render_state);

        world.insert(camera);

        dispatcher.setup(&mut world);

        Self { world, dispatcher }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        let mut render_state: WriteExpect<graphics::RenderState> = self.world.system_data();

        render_state.resize(new_size);
    }

    fn handle_event(&mut self, window: &winit::window::Window, event: &winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::CursorMoved { .. }
            | winit::event::WindowEvent::MouseInput { .. }
            | winit::event::WindowEvent::MouseWheel { .. } => {
                let mut camera: WriteExpect<graphics::Camera> = self.world.system_data();

                camera.handle_event(window, event);
            }
            _ => {}
        };
    }

    fn render(&mut self) {
        self.dispatcher.dispatch(&mut self.world);
    }
}

fn main() {
    graphics::run::<ExampleRenderLoop>();
}
