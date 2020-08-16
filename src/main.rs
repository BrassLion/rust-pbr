#[macro_use] extern crate itertools;

mod graphics;

use specs::prelude::*;

struct ExampleRenderLoop {
    world: World,
    dispatcher: Dispatcher<'static, 'static>,
}

struct RotationSystem;

impl<'a> System<'a> for RotationSystem {
    
    type SystemData = WriteStorage<'a, graphics::Pose>;

    fn run(&mut self, mut poses: Self::SystemData) {

        for pose in (&mut poses).join() {
            pose.model_matrix.append_rotation_wrt_center_mut(&nalgebra::UnitQuaternion::new(nalgebra::Vector3::new(0.0, 0.0, 0.01)))
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
            &render_state.device,
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
            .with(RotationSystem, "rot_system", &[])
            .with(graphics::RenderSystem, "render_system", &["rot_system"]).build();

        // Create world.
        let mut world = World::new();

        // Add model to world.
        let model_data = include_bytes!("../res/DamagedHelmet.glb");

        graphics::ModelLoader::add_glb_to_world(&render_state.device, &render_state.swap_chain_desc, &render_state.queue, model_data, &mut world);

        // Pass render state into ECS as last step.
        world.insert(render_state);

        world.insert(camera);

        dispatcher.setup(&mut world);

        Self {
            world,
            dispatcher,
        }
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
